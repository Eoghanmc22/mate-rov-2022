use serde::{Serialize, Deserialize};
use crate::CommunicationError;

#[derive(Serialize, Deserialize, Debug)]
pub enum DownstreamMessage {
    VelocityDataMessage(VelocityData),
    Msg
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct VelocityData {
    pub forwards_left: f32,
    pub forwards_right: f32,
    pub strafing: f32,
    pub up: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UpstreamMessage<'a> {
    Init,
    IMUStream(&'a [u8]),
    Log(&'a str),
    Panic,
    Ack,
    BadO,
    BadP(CommunicationError)
}