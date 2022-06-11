pub mod line_follower;

use opencv::prelude::*;
use sensor_fusion::state::{MotorState, RobotState};
use common::controller::VelocityData;

pub trait OpenCvHandler {
    type Goal: ToString;

    fn handle_frame(frame: &Mat, robot: &RobotState, motor: &MotorState, goal: Self::Goal) -> anyhow::Result<(VelocityData, Self::Goal)>;
}