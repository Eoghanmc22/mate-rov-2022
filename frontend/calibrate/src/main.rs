use sensor_fusion::state;
use sensor_fusion::state::RobotState;
use serial::Frame;

fn main() {
    let mut gyro_data = (0.0, 0.0, 0.0);
    let mut local_accel = (0.0, 0.0, 0.0);
    let mut world_accel = (0.0, 0.0, 0.0);
    let mut state = RobotState {
        first_read: true,
        ..Default::default()
    };
    let mut counter = 0;

    serial::listen(move |frame| {
        counter += 1;

        //calibrate_gyro(&frame, &mut gyro_data, counter);

        state::update_state(&frame, &mut state, true, true, true, true, true);

        calibrate_accel(&state, &mut local_accel, &mut world_accel, counter);
        Ok(())
    }).unwrap();
}

fn calibrate_gyro(frame: &Frame, data: &mut (f32, f32, f32), counter: usize) {
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

fn calibrate_accel(state: &RobotState, local: &mut (f32, f32, f32), world: &mut (f32, f32, f32), counter: usize) {
    let world_accel = state.acceleration;
    let local_accel = state.angle.inverse() * state.acceleration;

    world.0 += world_accel.x;
    world.1 += world_accel.y;
    world.2 += world_accel.z;

    local.0 += local_accel.x;
    local.1 += local_accel.y;
    local.2 += local_accel.z;

    if counter % 1000 == 0 {
        let (x_w, y_w, z_w) = *world;
        let (x_l, y_l, z_l) = *local;
        let count = counter as f32;

        println!("world sample mean {}, x: {}, y: {}, z: {}", counter, x_w / count, y_w / count, z_w / count);
        println!("local sample mean {}, x: {}, y: {}, z: {}", counter, x_l / count, y_l / count, z_l / count);
        println!();
        println!();
    }
}