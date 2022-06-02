use serde::{Serialize, Deserialize};
use crate::CommunicationError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DownstreamMessage {
    VelocityUpdate(VelocityData),
    EmergencyStop
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct VelocityData {
    pub forwards_left: f32,
    pub forwards_right: f32,
    pub strafing: f32,
    pub vertical: f32,
}

impl VelocityData {
    pub fn clamp(&self) -> VelocityData {
        fn clamp(num: f32) -> f32 { num.clamp(-1.0, 1.0) }

        VelocityData {
            forwards_left: clamp(self.forwards_left),
            forwards_right: clamp(self.forwards_right),
            strafing: clamp(self.strafing),
            vertical: clamp(self.vertical)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UpstreamMessage<'a> {
    Init,
    Log(&'a str),

    Ack,
    BadO,
    BadP(CommunicationError),

    TotalVelocity(VelocityData),

    EStop(bool)
}