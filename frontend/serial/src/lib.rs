#![feature(never_type)]

use std::io::{BufRead, Read};
use std::thread;
use std::time::Duration;
use glam::Vec3;
use serialport::{SerialPort, SerialPortType};

pub fn listen<F: FnMut(Frame) -> anyhow::Result<()>>(data_callback: F) -> anyhow::Result<!> {
    let port =
        serialport::available_ports()?
            .into_iter()
            .find(|port| {
                match &port.port_type {
                    SerialPortType::UsbPort(info) => {
                        if info.vid == 9025 && info.pid == 67 {
                            true
                        } else {
                            false
                        }
                    }
                    _ => {
                        false
                    }
                }
            });

    if let Some(port) = port {
        println!("Selected port {}", port.port_name);
        listen_to_port(&port.port_name, data_callback)
    } else {
        panic!("No suitable serial port found")
    }

}

pub fn listen_to_port<F: FnMut(Frame) -> anyhow::Result<()>>(port: &str, mut data_callback: F) -> anyhow::Result<!> {
    let mut port = serialport::new(port, 57_600)
        .open_native().expect("Failed to open port");

    let mut buf = [0; 200];
    let mut last_end = 0;

    loop {
        let btr = port.bytes_to_read()? as usize;

        if btr > 0 {
            let buf_end = usize::min(btr + last_end, buf.len());
            let read = port.read(&mut buf[last_end..buf_end])?;
            let available = read + last_end;

            let (frames, removed) = find_frames(&buf[..available]);
            buf.copy_within(available - removed..available, 0);


            for frame in frames.iter() {
                if let Some(frame) = decode_frame(frame.trim()) {
                    (data_callback)(frame)?;
                } else {
                    eprintln!("Dropped frame: {}", frame);
                }
            }

            last_end = removed;
        } else {
            thread::sleep(Duration::from_millis(1))
        }
    }
}

const NEW_LINE : u8 = 0x0A;
const CHAR_A : u8 = 0x41;

fn find_frames(buffer: &[u8]) -> (Vec<String>, usize) {
    let start = buffer.iter().position(|&byte| byte == CHAR_A);
    let end = buffer.iter().rposition(|&byte| byte == NEW_LINE);

    if let (Some(start), Some(end)) = (start, end) {
        if end > start {
            let frames: Vec<String> = (&buffer[start..end]).lines().flatten().collect();
            return (frames, buffer.len() - end);
        }
    }

    (vec![], buffer.len())
}

fn decode_frame(frame: &str) -> Option<Frame> {
    let mut parts = frame.split(" ");

    let mut vals = parts.next()?[1..].split(',').map(|it| it.parse::<i16>());
    let accel_x = vals.next()?.ok()?;
    let accel_y = vals.next()?.ok()?;
    let accel_z = vals.next()?.ok()?;

    let mut vals = parts.next()?[1..].split(',').map(|it| it.parse::<i16>());
    let gyro_x = vals.next()?.ok()?;
    let gyro_y = vals.next()?.ok()?;
    let gyro_z = vals.next()?.ok()?;

    let mut vals = parts.next()?[1..].split(',').map(|it| it.parse::<i16>());
    let mag_x = vals.next()?.ok()?;
    let mag_y = vals.next()?.ok()?;
    let mag_z = vals.next()?.ok()?;

    let pressure = parts.next()?[1..].parse::<u16>().ok()?;

    let collection_ms = parts.next()?[1..].parse::<u64>().ok()?;
    let total_ms = parts.next()?[1..].parse::<u64>().ok()?;

    if parts.count() > 0 {
        return None;
    }

    Some(raw_to_frame(accel_x, accel_y, accel_z, gyro_x, gyro_y, gyro_z, mag_x, mag_y, mag_z, pressure, collection_ms, total_ms))
}

const G_M : f32 = 9.80665;
const ACCEL_GAIN: f32 = 0.122 / 1000.0 * G_M;
const GYRO_GAIN: f32 = 70.0 / 1000.0;
const MAG_GAIN: f32 = 1.0 / 3421.0;

const G_X_OFFSET: f32 = 0.0;
const G_Y_OFFSET: f32 = 0.0;
const G_Z_OFFSET: f32 = 0.0;

const A_X_OFFSET: f32 = 0.0;
const A_Y_OFFSET: f32 = 0.0;
const A_Z_OFFSET: f32 = 0.0;

//gyro sample mean 30000, x: 1.400736, y: -4.4032674, z: -1.1412126
//gyro sample mean 518000, x: 1.367721, y: -4.466497, z: -1.0681778
//gyro sample mean 549000, x: 1.3885181, y: -4.5077543, z: -0.9722978

/*const G_X_OFFSET: f32 = 1.33794096;
const G_Y_OFFSET: f32 = -4.30960008;
const G_Z_OFFSET: f32 = -1.02645828;

const A_X_OFFSET: f32 = 0.0;
const A_Y_OFFSET: f32 = 0.0;
const A_Z_OFFSET: f32 = 0.0;*/

fn raw_to_frame(accel_x: i16, accel_y: i16, accel_z: i16, gyro_x: i16, gyro_y: i16, gyro_z: i16, mag_x: i16, mag_y: i16, mag_z: i16, pressure: u16, collection_ms: u64, total_ms: u64) -> Frame {
    let accel_x = accel_x as f32 * ACCEL_GAIN - A_X_OFFSET;
    let accel_y = accel_y as f32 * ACCEL_GAIN - A_Y_OFFSET;
    let accel_z = accel_z as f32 * ACCEL_GAIN - A_Z_OFFSET;

    let gyro_x = gyro_x as f32 * GYRO_GAIN - G_X_OFFSET;
    let gyro_y = gyro_y as f32 * GYRO_GAIN - G_Y_OFFSET;
    let gyro_z = gyro_z as f32 * GYRO_GAIN - G_Z_OFFSET;

    let mag_x = mag_x as f32 * MAG_GAIN;
    let mag_y = mag_y as f32 * MAG_GAIN;
    let mag_z = mag_z as f32 * MAG_GAIN;

    let pressure = (pressure as f32 / 1023.0 - 0.5) / 4.0 * 100.0;

    Frame {
        acceleration: Vec3::new(accel_x, accel_y, accel_z),
        gyro: Vec3::new(gyro_x, gyro_y, gyro_z),
        mag: Vec3::new(mag_x, mag_y, mag_z),
        pressure,
        collection_duration: Duration::from_millis(collection_ms),
        total_duration: Duration::from_millis(total_ms)
    }
}

#[derive(Debug)]
pub struct Frame {
    pub acceleration: Vec3,
    pub gyro: Vec3,
    pub mag: Vec3,
    pub pressure: f32,

    pub collection_duration: Duration,
    pub total_duration: Duration
}