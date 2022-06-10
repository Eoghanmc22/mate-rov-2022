use glam::Vec3;
use std::time::Duration;

#[derive(Debug)]
pub struct IMUFrame {
    pub acceleration: Vec3,
    pub gyro: Vec3,
    pub mag: Option<Vec3>,
    pub pressure: f32,

    pub total_duration: Duration,
}

pub fn decode_imu_frame(mut frame: &[u8]) -> Option<IMUFrame> {
    let full_frame = &*frame;

    let pressure = read_i16(&mut frame)?;

    let accel_x = read_i16(&mut frame)?;
    let accel_y = read_i16(&mut frame)?;
    let accel_z = read_i16(&mut frame)?;

    let gyro_x = read_i16(&mut frame)?;
    let gyro_y = read_i16(&mut frame)?;
    let gyro_z = read_i16(&mut frame)?;

    let mag = if frame.len() > 2 {
        let mag_x = read_i16(&mut frame)?;
        let mag_y = read_i16(&mut frame)?;
        let mag_z = read_i16(&mut frame)?;
        Some((mag_x, mag_y, mag_z))
    } else {
        None
    };

    let total_ms = *frame.get(0)?;

    let check = *frame.get(1)?;
    let actual = full_frame
        .get(..full_frame.len()-frame.len()+1)?
        .iter()
        .fold(0u8, |acc, &it| acc ^ it);

    if frame.len() != 3 || check != actual {
        println!("len: {}, ck: {:b}, asd: {}", frame.len(), check ^ actual, full_frame.len()-frame.len()+2);
        return None;
    }

    Some(raw_to_frame(accel_x, accel_y, accel_z, gyro_x, gyro_y, gyro_z, mag, pressure as u16, total_ms as u64))
}

fn read_i16(buffer: &mut &[u8]) -> Option<i16> {
    if buffer.len() >= 2 {
        let num = i16::from_le_bytes(buffer[..2].try_into().unwrap());
        *buffer = &buffer[2..];

        Some(num)
    } else {
        None
    }
}

const G_M : f32 = 9.80665;
const ACCEL_GAIN: f32 = 0.122 / 1000.0 * G_M;
const GYRO_GAIN: f32 = 70.0 / 1000.0;
const MAG_GAIN: f32 = 1.0 / 3421.0;

//gyro sample mean 30000, x: 1.400736, y: -4.4032674, z: -1.1412126
//gyro sample mean 518000, x: 1.367721, y: -4.466497, z: -1.0681778
//gyro sample mean 549000, x: 1.3885181, y: -4.5077543, z: -0.9722978

const G_X_OFFSET: f32 = 1.373729015;
const G_Y_OFFSET: f32 = -4.421779695;
const G_Z_OFFSET: f32 = -1.05203662;

const A_X_OFFSET: f32 = 0.0;
const A_Y_OFFSET: f32 = 0.0;
const A_Z_OFFSET: f32 = 0.0;

fn raw_to_frame(accel_x: i16, accel_y: i16, accel_z: i16, gyro_x: i16, gyro_y: i16, gyro_z: i16, mag: Option<(i16, i16, i16)>, pressure: u16, total_ms: u64) -> IMUFrame {
    let pressure = (pressure as f32 / 1023.0 * 5.0 - 0.5) / 4.0 * 100.0;

    let accel_x = accel_x as f32 * ACCEL_GAIN - A_X_OFFSET;
    let accel_y = accel_y as f32 * ACCEL_GAIN - A_Y_OFFSET;
    let accel_z = accel_z as f32 * ACCEL_GAIN - A_Z_OFFSET;

    let gyro_x = gyro_x as f32 * GYRO_GAIN - G_X_OFFSET;
    let gyro_y = gyro_y as f32 * GYRO_GAIN - G_Y_OFFSET;
    let gyro_z = gyro_z as f32 * GYRO_GAIN - G_Z_OFFSET;

    if let Some((mag_x, mag_y, mag_z)) = mag {
        let mag_x = mag_x as f32 * MAG_GAIN;
        let mag_y = mag_y as f32 * MAG_GAIN;
        let mag_z = mag_z as f32 * MAG_GAIN;

        IMUFrame {
            acceleration: Vec3::new(accel_x, accel_y, accel_z),
            gyro: Vec3::new(gyro_x, gyro_y, gyro_z),
            mag: Some(Vec3::new(mag_x, mag_y, mag_z)),
            pressure,
            total_duration: Duration::from_millis(total_ms)
        }
    } else {
        IMUFrame {
            acceleration: Vec3::new(accel_x, accel_y, accel_z),
            gyro: Vec3::new(gyro_x, gyro_y, gyro_z),
            mag: None,
            pressure,
            total_duration: Duration::from_millis(total_ms)
        }
    }
}
