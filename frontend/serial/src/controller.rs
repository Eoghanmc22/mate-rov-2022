use common::controller::{DownstreamMessage, UpstreamMessage};
use std::time::{Duration, Instant};
use mio_serial::{ClearBuffer, SerialPort, SerialPortInfo, SerialPortType};
use std::{io, thread};
use std::io::{Read, Write};
use std::sync::atomic::Ordering;
use anyhow::{bail, Context};
use crossbeam::channel::Receiver;
use mio::{Events, Interest, Poll, Token};
use mio_serial::{SerialPortBuilderExt, SerialStream};

fn get_port() -> anyhow::Result<Option<SerialPortInfo>> {
    Ok(mio_serial::available_ports()?
        .into_iter()
        .find(|port| {
            match &port.port_type {
                SerialPortType::UsbPort(info) => { info.vid == 0x2341 && info.pid == 0x42 }
                _ => { false }
            }
        }))
}

pub fn listen<F: FnMut(UpstreamMessage) -> anyhow::Result<()> + Send + 'static>(data_callback: F, commands: Option<Receiver<DownstreamMessage>>) -> anyhow::Result<!> {
    if let Some(port) = get_port()? {
        println!("Selected port {}", port.port_name);
        listen_to_port(&port.port_name, data_callback, commands)
    } else {
        bail!("No suitable serial port found")
    }

}

const SERIAL_TOKEN: Token = Token(0);

pub fn listen_to_port<F: FnMut(UpstreamMessage) -> anyhow::Result<()> + Send + 'static>(port: &str, mut data_callback: F, commands: Option<Receiver<DownstreamMessage>>) -> anyhow::Result<!> {
    let mut poll = Poll::new().context("could not create poll")?;
    let mut events = Events::with_capacity(10);

    let mut port = mio_serial::new(port, common::BAUD_RATE_CTRL).open_native_async().context("could not open serial stream")?;

    // todo try commenting this out on win?
    port.clear(ClearBuffer::All).context("could not clear port")?;

    poll.registry()
        .register(&mut port, SERIAL_TOKEN, Interest::READABLE | Interest::WRITABLE)
        .context("could not register port")?;

    let mut buf_read = [0; 4098];
    let mut last_end = 0;
    let mut buf_write = [0; 4098];
    let mut buf_partial = [0; 4098];
    let mut partial_written = 0;
    let mut writeable = false;
    let mut last_write = Instant::now();
    let mut allow_writes = false;

    loop {
        // Fixme better way to wake up this thread?
        poll.poll(&mut events, Some(MIN_WRITE_DELAY)).context("could not poll")?;

        for event in &events {
            match event.token() {
                SERIAL_TOKEN => {
                    println!("event: r: {}, w: {}", event.is_readable(), event.is_writable());
                    if event.is_readable() {
                        do_read(&mut buf_read, &mut last_end, &mut data_callback, &mut port, &mut allow_writes).context("Read error")?;
                    }
                    if let Some(ref commands) = commands {
                        if event.is_writable() {
                            if allow_writes {
                                writeable = do_write(&mut buf_write, &mut buf_partial, &mut partial_written, commands, &mut port, &mut last_write).context("Write error")?;
                            } else {
                                writeable = true;
                            }
                        }
                    }
                },
                _ => {}
            }
        }

        if let Some(ref commands) = commands {
            if writeable && allow_writes {
                writeable = do_write(&mut buf_write, &mut buf_partial, &mut partial_written, commands, &mut port, &mut last_write).context("Write error")?;
            }
        }
    }
}

fn do_read<F: FnMut(UpstreamMessage) -> anyhow::Result<()>>(buffer: &mut [u8], last_end: &mut usize, data_callback: &mut F, port: &mut SerialStream, allow_writes: &mut bool) -> anyhow::Result<()> {
    loop {
        if buffer[*last_end..].len() == 0 {
            bail!("Read buffer full")
        }

        match port.read(&mut buffer[*last_end..]) {
            Ok(0) => {
                bail!("Remote device was disconnected");
            }
            Ok(read) => {
                println!("read: {}", read);
                let available = read + *last_end;
                let frames = buffer[..available]
                    .split_inclusive_mut(common::end_of_frame);

                let mut removed = 0;
                for frame in frames {
                    if common::end_of_frame(frame.last().unwrap()) {
                        match common::read(frame) {
                            Ok(message) => {
                                (data_callback)(message)?;

                                *allow_writes = true;
                            }
                            Err(com_error) => {
                                println!("read error: {:?}", com_error);
                            }
                        }
                    } else {
                        removed = frame.len();
                        break;
                    }
                }

                buffer.copy_within(available - removed..available, 0);
                *last_end = removed;
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
                continue;
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                break;
            }
            Err(e) => {
                return Err(e).context("Io error");
            }
        }
    }

    Ok(())
}

const MIN_WRITE_DELAY: Duration = Duration::from_millis(2);
const MAX_COMMANDS: usize = 2;

fn do_write(buffer: &mut [u8], buf_partial: &mut [u8], partial_written: &mut usize, command_stream: &Receiver<DownstreamMessage>, port: &mut SerialStream, last_write: &mut Instant) -> anyhow::Result<bool> {
    if *partial_written > 0 {
        let mut buffer = &buf_partial[..*partial_written];
        while !buffer.is_empty() {
            match port.write(buffer) {
                Ok(0) => {
                    bail!("Failed to write buffer");
                }
                Ok(n) => {
                    buffer = &buffer[n..];
                }
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
                    continue;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    let remaining = buffer.len();
                    buf_partial.copy_within(*partial_written-remaining..*partial_written, 0);
                    *partial_written = remaining;
                    return Ok(false);
                }
                Err(e) => {
                    return Err(e).context("Io error");
                }
            }
        }
        *partial_written = 0;
    }

    if last_write.elapsed() > MIN_WRITE_DELAY {
        for command in command_stream.try_iter().take(MAX_COMMANDS) {
            if let Ok(mut buffer) = common::write(&command, buffer) {
                while !buffer.is_empty() {
                    match port.write(buffer) {
                        Ok(0) => {
                            return Err(io::Error::new(
                                io::ErrorKind::WriteZero,
                                "failed to write whole buffer",
                            )).context("Write zero");
                        }
                        Ok(n) => {
                            buffer = &mut buffer[n..];
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {
                            continue;
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            let additional = buffer.len();
                            assert!(additional < buf_partial.len() - *partial_written, "partial write buffer is full");
                            buf_partial[*partial_written..*partial_written+additional].copy_from_slice(buffer);
                            *partial_written += additional;
                            return Ok(false);
                        }
                        Err(e) => {
                            return Err(e).context("Io error");
                        }
                    }
                }
            }
        }

        *last_write = Instant::now();
    }

    Ok(true)
}
