use glam::*;
use crate::state::RobotState;

//https://stanford.edu/class/ee267/lectures/lecture10.pdf
pub fn integrate_gyro(state: &mut RobotState, duration: f32) {
    let gyro_velocity = vec3(state.gyro_velocity.x.to_radians(), state.gyro_velocity.y.to_radians(), state.gyro_velocity.z.to_radians());
    let v = gyro_velocity.normalize_or_zero();
    if 1.0 - v.length() < 0.01 {
        let update = Quat::from_axis_angle(v, gyro_velocity.length() * duration);
        state.angle = state.angle * update;
    }
}

//https://stanford.edu/class/ee267/lectures/lecture10.pdf
pub fn tilt_correction(state: &mut RobotState, a_a: f32) {
    let world_acceleration = state.angle * state.acceleration;
    let v = world_acceleration.normalize_or_zero();
    let v2 = v.cross(vec3(0.0, 0.0, 1.0)).normalize_or_zero();
    if 1.0 - v2.length() < 0.01 {
        let tilt_correction = Quat::from_axis_angle(v2, v.z.acos() * (1.0 - a_a));
        state.angle = tilt_correction * state.angle;
    }
    state.acceleration = state.angle * state.acceleration;
}

pub fn yaw_correction(state: &mut RobotState, a_m: f32) {
    //state.mag = state.angle * state.mag;
}

const GRAVITY : f32 = 9.80665;
pub fn subtract_gravity(state: &mut RobotState) {
    state.acceleration -= vec3(0.0, 0.0, GRAVITY);
}

pub fn integrate_acceleration(state: &mut RobotState, duration: f32) {
    state.velocity += state.acceleration * duration;
    state.position += state.velocity * duration;
}
