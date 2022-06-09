pub mod handlers;

use opencv::prelude::*;
use sensor_fusion::state::{MotorState, RobotState};
use common::controller::VelocityData;

pub trait OpenCvHandler {
    fn handle_frame(frame: &Mat, robot: &RobotState, motor: &MotorState) -> anyhow::Result<VelocityData>;
}