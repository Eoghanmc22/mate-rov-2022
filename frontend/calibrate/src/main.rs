#![feature(never_type)]

use sensor_fusion::state;
use sensor_fusion::state::RobotState;
use serial::frame::IMUFrame;

fn main() -> anyhow::Result<!> {
    let mut gyro_data = (0.0, 0.0, 0.0);
    let mut local_accel = (0.0, 0.0, 0.0);
    let mut world_accel = (0.0, 0.0, 0.0);

    let mut state = RobotState {
        first_read: true,
        ..Default::default()
    };
    let mut counter = 0;

    serial::serial::listen(move |frame| {
        counter += 1;

        calibrate_gyro(&frame, &mut gyro_data, counter);

        state::update_state(&frame, &mut state);

        //calibrate_local_accel(&frame, &mut local_accel, counter);
        Ok(())
    }, None)
}

fn calibrate_gyro(frame: &IMUFrame, data: &mut (f32, f32, f32), counter: usize) {
    let g = frame.gyro;

    data.0 += g.x;
    data.1 += g.y;
    data.2 += g.z;

    if counter % 1000 == 0 {
        let (x, y, z) = *data;
        let count = counter as f32;

        println!("gyro sample mean {}, x: {}, y: {}, z: {}", counter, x / count, y / count, z / count);
        println!();
    }
}

fn calibrate_local_accel(frame: &IMUFrame, local: &mut (f32, f32, f32), counter: usize) {
    let local_accel = frame.acceleration;

    local.0 += local_accel.x;
    local.1 += local_accel.y;
    local.2 += local_accel.z;

    if counter % 1000 == 0 {
        let (x, y, z) = *local;
        let count = counter as f32;

        println!("local sample mean {}, x: {}, y: {}, z: {}", counter, x / count, y / count, z / count);
        println!();
    }
}

fn calibrate_world_accel(state: &RobotState, world: &mut (f32, f32, f32), counter: usize) {
    let world_accel = state.acceleration;

    world.0 += world_accel.x;
    world.1 += world_accel.y;
    world.2 += world_accel.z;

    if counter % 1000 == 0 {
        let (x, y, z) = *world;
        let count = counter as f32;

        println!("world sample mean {}, x: {}, y: {}, z: {}", counter, x / count, y / count, z / count);
        println!();
    }
}