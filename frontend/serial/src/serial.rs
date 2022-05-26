use serialport::{SerialPort, SerialPortInfo, SerialPortType};
use std::io::{ErrorKind, Read, Write};
use std::thread;
use std::time::Duration;
use anyhow::bail;
use crossbeam::channel::Receiver;
use crate::commands::Command;
use crate::frame;
use crate::frame::IMUFrame;

pub fn get_port() -> anyhow::Result<Option<SerialPortInfo>> {
    Ok(serialport::available_ports()?
        .into_iter()
        .find(|port| {
            println!("{:?}", port);
            match &port.port_type {
                SerialPortType::UsbPort(info) => { info.vid == 9025 && info.pid == 66 }
                _ => { false }
            }
        }))
}

pub fn listen<F: FnMut(IMUFrame) -> anyhow::Result<()>>(data_callback: F, commands: Option<&Receiver<Command>>) -> anyhow::Result<!> {
    if let Some(port) = get_port()? {
        println!("Selected port {}", port.port_name);
        listen_to_port(&port.port_name, data_callback, commands)
    } else {
        bail!("No suitable serial port found")
    }

}

pub fn listen_to_port<F: FnMut(IMUFrame) -> anyhow::Result<()>>(port: &str, mut data_callback: F, commands: Option<&Receiver<Command>>) -> anyhow::Result<!> {
    let mut port = serialport::new(port, common::BAUD_RATE_PC)
        .open_native().expect("Failed to open port");

    let mut buf = [0; 200];
    let mut last_end = 0;

    loop {
        let btr = port.bytes_to_read()? as usize;

        if let Some(commands) = commands {
            for command in commands.try_iter() {
                let buf = command.to_command_string();
                print!("sent: {}", buf);
                port.write(buf.as_bytes())?;
            }
        }

        if btr > 0 {
            let buf_end = usize::min(btr + last_end, buf.len());
            let read = match port.read(&mut buf[last_end..buf_end]) {
                Ok(read) => { read }
                Err(err) => {
                    if err.kind() == ErrorKind::TimedOut {
                        continue;
                    } else {
                        return Err(err.into());
                    }
                }
            };
            let available = read + last_end;

            let (frames, removed) = common::find_frames(&buf[..available]);

            for frame in frames {
                frame::process_frame(&frame[1..], &mut data_callback)?;
            }
            
            buf.copy_within(available - removed..available, 0);

            last_end = removed;
        } else {
            thread::sleep(Duration::from_millis(1))
        }
    }
}
