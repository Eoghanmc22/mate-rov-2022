use glam::Vec3;
use std::time::Duration;

#[derive(Debug)]
pub struct IMUFrame {
    pub acceleration: Vec3,
    pub gyro: Vec3,
    pub mag: Vec3,
    pub pressure: f32,

    pub collection_duration: Duration,
    pub total_duration: Duration
}

pub fn decode_imu_frame(frame: &str) -> Option<IMUFrame> {
    let mut parts = frame.split(" ");

    let mut vals = parts.next()?.get(1..)?.split(',').map(|it| it.parse::<i16>());
    let accel_x = vals.next()?.ok()?;
    let accel_y = vals.next()?.ok()?;
    let accel_z = vals.next()?.ok()?;

    let mut vals = parts.next()?.get(1..)?.split(',').map(|it| it.parse::<i16>());
    let gyro_x = vals.next()?.ok()?;
    let gyro_y = vals.next()?.ok()?;
    let gyro_z = vals.next()?.ok()?;

    let mut vals = parts.next()?.get(1..)?.split(',').map(|it| it.parse::<i16>());
    let mag_x = vals.next()?.ok()?;
    let mag_y = vals.next()?.ok()?;
    let mag_z = vals.next()?.ok()?;

    let pressure = parts.next()?.get(1..)?.parse::<u16>().ok()?;

    let collection_ms = parts.next()?.get(1..)?.parse::<u64>().ok()?;
    let total_ms = parts.next()?.get(1..)?.parse::<u64>().ok()?;

    if parts.count() > 0 {
        return None;
    }

    Some(raw_to_frame(accel_x, accel_y, accel_z, gyro_x, gyro_y, gyro_z, mag_x, mag_y, mag_z, pressure, collection_ms, total_ms))
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

fn raw_to_frame(accel_x: i16, accel_y: i16, accel_z: i16, gyro_x: i16, gyro_y: i16, gyro_z: i16, mag_x: i16, mag_y: i16, mag_z: i16, pressure: u16, collection_ms: u64, total_ms: u64) -> IMUFrame {
    let accel_x = accel_x as f32 * ACCEL_GAIN - A_X_OFFSET;
    let accel_y = accel_y as f32 * ACCEL_GAIN - A_Y_OFFSET;
    let accel_z = accel_z as f32 * ACCEL_GAIN - A_Z_OFFSET;

    let gyro_x = gyro_x as f32 * GYRO_GAIN - G_X_OFFSET;
    let gyro_y = gyro_y as f32 * GYRO_GAIN - G_Y_OFFSET;
    let gyro_z = gyro_z as f32 * GYRO_GAIN - G_Z_OFFSET;

    let mag_x = mag_x as f32 * MAG_GAIN;
    let mag_y = mag_y as f32 * MAG_GAIN;
    let mag_z = mag_z as f32 * MAG_GAIN;

    let pressure = (pressure as f32 / 1023.0 * 5.0 - 0.5) / 4.0 * 100.0;

    IMUFrame {
        acceleration: Vec3::new(accel_x, accel_y, accel_z),
        gyro: Vec3::new(gyro_x, gyro_y, gyro_z),
        mag: Vec3::new(mag_x, mag_y, mag_z),
        pressure,
        collection_duration: Duration::from_millis(collection_ms),
        total_duration: Duration::from_millis(total_ms)
    }
}
