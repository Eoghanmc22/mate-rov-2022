use common::controller::{DownstreamMessage, VelocityData};

/// A struct that keeps track of the robots current state
#[derive(Default)]
pub struct State {
    // forwards_left, forwards_right, strafing, up
    motor_sp_pc: VelocityData,
    motor_sp_joystick: VelocityData,

    emergency_stop: bool,

    do_ping: bool,
}

impl State {
    /// Update the state with info from the connected pc
    pub fn update_pc(&mut self, message: DownstreamMessage) {
        match message {
            DownstreamMessage::VelocityUpdate(velocity) => {
                self.motor_sp_pc = velocity;
            }
            DownstreamMessage::EmergencyStop => {
                self.emergency_stop = true;
            }
            DownstreamMessage::Ping => {
                self.do_ping = true;
            }
        }
    }

    /// Update the velocity with info from the physical joysticks
    pub fn update_joystick(&mut self, velocity: VelocityData) {
        self.motor_sp_joystick = velocity;
    }

    /// Update the emergency stop state with info from the physical button
    pub fn update_emergency_stop(&mut self, emergency_stop: bool) {
        self.emergency_stop |= emergency_stop;
    }

    /// Is an emergency stop active
    pub fn emergency_stop(&self) -> bool {
        self.emergency_stop
    }

    /// Do we need to respond to a ping
    pub fn do_ping(&self) -> bool {
        self.do_ping
    }

    /// Clear ping status
    pub fn clear_ping(&mut self) {
        self.do_ping = false;
    }

    // Maybe add interpolation? prob not necessary tho
    /// Compute the motor speed setpoints
    pub fn compute_velocity(&self) -> VelocityData {
        if self.emergency_stop {
            return VelocityData::default();
        }

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