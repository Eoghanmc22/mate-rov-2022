use serialport::{ClearBuffer, SerialPort, SerialPortInfo, SerialPortType};
use std::io;
use std::io::Read;
use std::time::Duration;
use anyhow::bail;
use sensor_fusion::frame::IMUFrame;
use sensor_fusion::frame;

fn get_port() -> anyhow::Result<Option<SerialPortInfo>> {
    Ok(serialport::available_ports()?
        .into_iter()
        .find(|port| {
            match &port.port_type {
                SerialPortType::UsbPort(info) => { info.vid == 0x2341 && info.pid == 0x43 }
                _ => { false }
            }
        }))
}

pub fn listen<F: FnMut(IMUFrame, u32) -> anyhow::Result<()>>(imu_notification: F) -> anyhow::Result<!> {
    if let Some(port) = get_port()? {
        println!("Selected port {}", port.port_name);
        listen_to_port(&port.port_name, imu_notification)
    } else {
        bail!("No suitable serial port found")
    }

}

pub fn listen_to_port<F: FnMut(IMUFrame, u32) -> anyhow::Result<()>>(port: &str, mut imu_notification: F) -> anyhow::Result<!> {
    let mut port = serialport::new(port, common::BAUD_RATE_FORWARD)
        .timeout(Duration::from_millis(1))
        .open_native()
        .expect("Failed to open port");

    port.clear(ClearBuffer::All)?;

    let mut buffer = [0; 4098];
    let mut last_end = 0;
    let mut makeup = 0;

    loop {
        match port.read(&mut buffer[last_end..]) {
            Ok(read) => {
                let available = read + last_end;
                let frames = buffer[..available].split_inclusive(|&byte| byte == 0x6E);

                let mut removed = 0;
                for frame in frames {
                    if *frame.last().unwrap() == 0x6E {
                        if let Some(frame) = frame::decode_imu_frame(frame) {
                            (imu_notification)(frame, makeup)?;
                            makeup = 0;
                        } else {
                            println!("invalid frame");
                            makeup += 1;
                        }
                    } else {
                        removed = frame.len();
                        break;
                    }
                }

                buffer.copy_within(available - removed..available, 0);
                last_end = removed;
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::TimedOut {
                    Err::<(), _>(e).unwrap();
                }
            }
        }
    }
}
