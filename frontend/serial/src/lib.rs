use std::error::Error;
use std::io::{BufRead, Read};
use std::time::Duration;
use common::vec::Vec3d;

pub fn listen_to_port<F: FnMut(Frame)>(port: &str, data_callback: &mut F) -> Result<(), Box<dyn Error>> {
    let mut port = serialport::new(port, 57_600)
        .open_native().expect("Failed to open port");

    let mut buf = [0; 200];
    let mut last_end = 0;

    // first read seems to get junk data
    // look into why / if this is needed
    let _ = port.read(&mut buf)?;

    loop {
        let read = port.read(&mut buf[last_end..])?;
        let available = read + last_end;

        let (frames, removed) = find_frames(&buf[..available]);
        buf.copy_within(available - removed..available, 0);

        for frame in frames.iter() {
            if let Some(frame) = decode_frame(frame) {
                (data_callback)(frame);
            }
        }

        last_end = removed;
    }
}

const NEW_LINE : u8 = 0x0A;
const CHAR_A : u8 = 0x41;

fn find_frames(buffer: &[u8]) -> (Vec<String>, usize) {
    let mut frames = buffer.lines().flatten().collect::<Vec<String>>();

    if buffer[0] != CHAR_A {
        frames.remove(0);
    }

    if buffer[buffer.len() - 1] != NEW_LINE {
        let removed = frames.pop().expect("No frame to remove");
        (frames, removed.len())
    } else {
        (frames, 0)
    }
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

    let collection_ms = parts.next()?[1..].parse::<u64>().ok()?;
    let total_ms = parts.next()?[1..].parse::<u64>().ok()?;

    Some(raw_to_frame(accel_x, accel_y, accel_z, gyro_x, gyro_y, gyro_z, mag_x, mag_y, mag_z, collection_ms, total_ms))
}

// +/- 4g
const ACCEL_FS: f64 = 4.0;
// +/- 500dps
const GYRO_FS: f64 = 500.0;
// +/- 8 gauss
const MAG_FS: f64 = 8.0;

const G_M : f64 = 9.80665;
const ACCEL_GAIN: f64 = ACCEL_FS * 2.0 / 32768.0 * G_M;
const GYRO_GAIN: f64 = GYRO_FS * 2.0 / 32768.0;
const MAG_GAIN: f64 = MAG_FS * 2.0 / 32768.0;


fn raw_to_frame(accel_x: i16, accel_y: i16, accel_z: i16, gyro_x: i16, gyro_y: i16, gyro_z: i16, mag_x: i16, mag_y: i16, mag_z: i16, collection_ms: u64, total_ms: u64) -> Frame {
    let accel_x = accel_x as f64 * ACCEL_GAIN;
    let accel_y = accel_y as f64 * ACCEL_GAIN;
    let accel_z = accel_z as f64 * ACCEL_GAIN;

    let gyro_x = gyro_x as f64 * GYRO_GAIN;
    let gyro_y = gyro_y as f64 * GYRO_GAIN;
    let gyro_z = gyro_z as f64 * GYRO_GAIN;

    let mag_x = mag_x as f64 * MAG_GAIN;
    let mag_y = mag_y as f64 * MAG_GAIN;
    let mag_z = mag_z as f64 * MAG_GAIN;

    Frame {
        acceleration: Vec3d::new(accel_x, accel_y, accel_z),
        gyro: Vec3d::new(gyro_x, gyro_y, gyro_z),
        mag: Vec3d::new(mag_x, mag_y, mag_z),
        collection_duration: Duration::from_millis(collection_ms),
        total_duration: Duration::from_millis(total_ms)
    }
}

#[derive(Debug)]
pub struct Frame {
    pub acceleration: Vec3d,
    pub gyro: Vec3d,
    pub mag: Vec3d,

    pub collection_duration: Duration,
    pub total_duration: Duration
}