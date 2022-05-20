use std::time::Duration;
use glam::*;
use serial::Frame;
use crate::fusion::*;

#[derive(Clone, Debug, Default)]
pub struct RobotState {
    pub acceleration: Vec3,
    pub velocity: Vec3,
    pub position: Vec3,

    pub gyro_velocity: Vec3,
    pub angle: Quat,
    pub gyro_angle: Vec3,

    pub mag: Vec3,

    pub pressure: f32,

    pub collection_duration: Duration,
    pub total_duration: Duration,

    pub first_read: bool
}

pub fn update_state(frame: Frame, state: &mut RobotState, gyro: bool, tilt: bool, yaw: bool, gravity: bool, acceleration: bool) {
    let a_a = 0.95;
    let a_m = 0.95;

    let duration = frame.total_duration.as_secs_f32();

    state.acceleration = frame.acceleration;
    state.gyro_velocity = /*gyro_velocity_high_pass.filter(*/frame.gyro/*, duration)*/;
    state.mag = frame.mag;
    state.pressure = frame.pressure;
    state.collection_duration = frame.collection_duration;
    state.total_duration = frame.total_duration;

    if state.first_read {
        tilt_correction(state, 0.0);
    }

    integrate_gyro(state, duration);
    tilt_correction(state, a_a);
    yaw_correction(state, a_m);
    subtract_gravity(state);
    integrate_acceleration(state, duration);

    let (e_yaw, e_pitch, e_roll) = state.angle.to_euler(EulerRot::ZXY);
    state.gyro_angle = (e_yaw.to_degrees(), e_pitch.to_degrees(), e_roll.to_degrees()).into();

    state.first_read = false;
}