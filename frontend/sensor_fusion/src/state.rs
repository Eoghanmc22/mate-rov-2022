use std::time::Duration;
use glam::*;
use common::controller::UpstreamMessage;
use crate::frame;
use crate::frame::{decode_imu_frame, IMUFrame};
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

    first_read: bool,

    frame_buffer: Option<Vec<u8>>
}

impl RobotState {
    pub fn reset(&mut self) {
        *self = Default::default();
        self.first_read = true;
    }
}

pub fn handle_message<F: Fn(&RobotState) -> anyhow::Result<()>>(message: &UpstreamMessage, state: &mut RobotState, imu_notification: F) -> anyhow::Result<()> {
    match message {
        UpstreamMessage::IMUStream(byte) => {
            let mut frame_buffer = state.frame_buffer.take().unwrap_or(vec![]);
            frame_buffer.push(*byte);

            if let Some(start) = frame_buffer.iter().position(|&byte| byte == b'A') {
                if let Some(len) = frame_buffer[start..].iter().rposition(|&byte| byte == b'\n') {
                    for frame in frame_buffer[start..start + len].split(|&byte| byte == b'\n') {
                        if let Ok(frame) = core::str::from_utf8(frame) {
                            if let Some(frame) = decode_imu_frame(frame.trim()) {
                                update_state(&frame, state);
                                (imu_notification)(state)?;
                            } else {
                                println!("invalid frame: {}", frame)
                            }
                        } else {
                            println!("invalid frame")
                        }
                    }

                    frame_buffer.copy_within(start + len.., 0);
                    frame_buffer.truncate(frame_buffer.len() - (start + len));
                } else {
                    frame_buffer.copy_within(start.., 0);
                    frame_buffer.truncate(frame_buffer.len() - start);
                }
            } else {
                frame_buffer.clear();
            }

            state.frame_buffer.replace(frame_buffer);
        }
        UpstreamMessage::Log(msg) => {
            println!("Arduino logged: {}", msg)
        }
        UpstreamMessage::Panic => {
            println!("Arduino panicked")
        }
        UpstreamMessage::Init => {
            println!("Arduino init")
        }
        UpstreamMessage::Ack => {
            //println!("ack")
        }
        UpstreamMessage::Bad => {
            //println!("bad")
        }
    }

    Ok(())
}

pub fn update_state(frame: &IMUFrame, state: &mut RobotState) {
    let a_a = 0.98;
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