#![feature(never_type)]

use serialport::{ClearBuffer, SerialPort, SerialPortInfo, SerialPortType};
use std::io::{Read, Write};
use std::{io, thread};
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

pub fn listen<F: FnMut(UpstreamMessage) -> anyhow::Result<()>>(data_callback: F, commands: Option<&Receiver<DownstreamMessage>>) -> anyhow::Result<!> {
    if let Some(port) = get_port()? {
        println!("Selected port {}", port.port_name);
        listen_to_port(&port.port_name, data_callback, commands)
    } else {
        bail!("No suitable serial port found")
    }

}

pub fn listen_to_port<F: FnMut(UpstreamMessage) -> anyhow::Result<()>>(port: &str, mut data_callback: F, commands: Option<&Receiver<DownstreamMessage>>) -> anyhow::Result<!> {
    let mut port = serialport::new(port, common::BAUD_RATE_PC)
        .timeout(Duration::from_millis(10))
        .open()
        .expect("Failed to open port");

    port.clear(ClearBuffer::All)?;

    let mut buf = [0; 1000];
    let mut last_end = 0;

    loop {
        match port.read(&mut buf[last_end..]) {
            Ok(read) => {
                let available = read + last_end;
                let frames = buf[..available]
                    .split_inclusive_mut(|&byte| common::end_of_frame(byte));

                let mut removed = 0;
                for frame in frames {
                    if common::end_of_frame(*frame.last().unwrap()) {
                        if let Ok(message) = common::read(frame) {
                            (data_callback)(message)?;
                        }
                    } else {
                        removed = frame.len();
                        break;
                    }
                }

                buf.copy_within(available - removed..available, 0);
                last_end = removed;
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
            Err(e) => return Err(e.into())
        }

        if let Some(commands) = commands {
            for command in commands.try_iter() {
                let mut out = [0; 250];
                if let Ok(buffer) = common::write(&command, &mut out) {
                    write_all(buffer, &mut port)?;
                }
            }
        }
    }
}

fn write_all(mut buf: &[u8], port: &mut Box<dyn SerialPort>) -> io::Result<()> {
    while !buf.is_empty() {
        match port.write(buf) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "failed to write whole buffer",
                    ));
            }
            Ok(n) => buf = &buf[n..],
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
