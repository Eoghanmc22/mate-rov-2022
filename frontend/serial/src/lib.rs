#![feature(never_type)]

use serialport::{ClearBuffer, SerialPort, SerialPortInfo, SerialPortType};
use std::{io, thread};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use anyhow::bail;
use crossbeam::channel::Receiver;
use common::controller::{DownstreamMessage, UpstreamMessage};

pub fn get_port() -> anyhow::Result<Option<SerialPortInfo>> {
    Ok(serialport::available_ports()?
        .into_iter()
        .find(|port| {
            match &port.port_type {
                SerialPortType::UsbPort(info) => { info.vid == 9025 && info.pid == 66 }
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

pub fn listen_to_port<F: FnMut(UpstreamMessage) -> anyhow::Result<()> + Send + 'static>(port: &str, mut data_callback: F, commands: Option<Receiver<DownstreamMessage>>) -> anyhow::Result<!> {
    let mut port = serialport::new(port, common::BAUD_RATE_CTRL)
        .timeout(Duration::from_millis(1))
        .open_native()
        .expect("Failed to open port");

    port.clear(ClearBuffer::All)?;

    let mut buf_read = [0; 4098];
    let mut buf_write = [0; 4098];
    let mut last_end = 0;

    let should_write = Arc::new(AtomicBool::new(false));

    if let Some(ref commands) = commands {
        if let Ok(mut other) = port.try_clone_native() {
            let reader = {
                let should_write = should_write.clone();

                thread::spawn(move || {
                    loop {
                        do_read(&mut buf_read, &mut last_end, &mut data_callback, &mut port, &should_write)?;
                    }
                })
            };

            let commands = commands.to_owned();
            let writer = thread::spawn(move || {
                let mut last_write = Instant::now();

                loop {
                    if should_write.load(Ordering::Relaxed) {
                        do_write(&mut buf_write, &commands, &mut other, &mut last_write)?;
                    } else {
                        thread::sleep(Duration::from_millis(1));
                    }
                }
            });

            loop {
                if reader.is_finished() {
                    return reader.join().unwrap();
                }

                if writer.is_finished() {
                    return writer.join().unwrap();
                }

                thread::sleep(Duration::from_millis(100));
            }
        }
    }

    // FIXME Why does sync version get locked up on linux?
    let mut last_write = Instant::now();

    loop {
        do_read(&mut buf_read, &mut last_end, &mut data_callback, &mut port, &should_write)?;

        if let Some(ref commands) = commands {
            if should_write.load(Ordering::Relaxed) {
                do_write(&mut buf_write, commands, &mut port, &mut last_write)?;
            }
        }
    }
}

fn do_read<F: FnMut(UpstreamMessage) -> anyhow::Result<()>>(buffer: &mut [u8], last_end: &mut usize, data_callback: &mut F, port: &mut impl SerialPort, should_write: &AtomicBool) -> anyhow::Result<()> {
    match port.read(&mut buffer[*last_end..]) {
        Ok(read) => {
            //println!("read: {}", read);
            let available = read + *last_end;
            let frames = buffer[..available]
                .split_inclusive_mut(common::end_of_frame);

            let mut removed = 0;
            for frame in frames {
                if common::end_of_frame(frame.last().unwrap()) {
                    match common::read(frame) {
                        Ok(message) => {
                            //println!("{:#?}", message);
                            (data_callback)(message)?;

                            should_write.store(true, Ordering::Relaxed);
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
        Err(e) => {
            assert_eq!(e.kind(), io::ErrorKind::TimedOut);
        }
    }

    Ok(())
}

const MIN_WRITE_DELAY: Duration = Duration::from_millis(2);
fn do_write(buffer: &mut [u8], command_stream: &Receiver<DownstreamMessage>, port: &mut impl SerialPort, last_write: &mut Instant) -> anyhow::Result<()> {
    if last_write.elapsed() > MIN_WRITE_DELAY {
        for command in command_stream.try_iter().take(2) {
            if let Ok(buffer) = common::write(&command, buffer) {
                write_all(buffer, port)?;
            }
        }

        *last_write = Instant::now();
    }

    Ok(())
}

fn write_all(mut buf: &[u8], port: &mut impl SerialPort) -> io::Result<()> {
    while !buf.is_empty() {
        match port.write(buf) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "failed to write whole buffer",
                    ));
            }
            Ok(n) => {
                buf = &buf[n..]
            },
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                //println!("TimedOut write_all");
            }
            Err(e) => return Err(e),
        }
    }
    port.flush()?;
    Ok(())
}
