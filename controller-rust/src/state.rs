use common::controller::{DownstreamMessage, VelocityData};

#[derive(Default)]
pub struct State {
    // forwards_left, forwards_right, strafing, up
    motor_sp_pc: VelocityData,
    motor_sp_joystick: VelocityData,
}

impl State {
    pub fn update_pc(&mut self, message: DownstreamMessage) {
        match message {
            DownstreamMessage::VelocityDataMessage(velocity) => {
                self.motor_sp_pc = velocity
            }
            _ => {}
        }
    }

    pub fn update_joystick(&mut self, velocity: VelocityData) {
        self.motor_sp_joystick = velocity
    }

    // Maybe add interpolation? prob not necessary tho
    pub fn compute_velocity(&self) -> VelocityData {
        let motor_sp_pc = self.motor_sp_pc.clamp();
        let motor_sp_joystick = self.motor_sp_joystick.clamp();

        VelocityData {
            forwards_left: motor_sp_pc.forwards_left   + motor_sp_joystick.forwards_left,
            forwards_right: motor_sp_pc.forwards_right + motor_sp_joystick.forwards_right,
            strafing: motor_sp_pc.strafing             + motor_sp_joystick.strafing,
            vertical: motor_sp_pc.vertical             + motor_sp_joystick.vertical
        }.clamp()
    }
}