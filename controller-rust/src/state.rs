use common::controller::{DownstreamMessage, VelocityData};

#[derive(Default)]
pub struct State {
    // forwards_left, forwards_right, strafing, up
    motor_sp_pc: VelocityData,
    motor_sp_joystick: VelocityData,
}

impl State {
    pub fn update(&mut self, message: DownstreamMessage) {
        match message {
            DownstreamMessage::VelocityDataMessage(velocity) => {
                self.motor_sp_pc = velocity
            }
            _ => {}
        }
    }
}