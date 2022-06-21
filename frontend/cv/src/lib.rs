pub mod line_follower;
pub mod take_image;
pub mod dock;
pub mod mosaic;

use opencv::prelude::*;
use sensor_fusion::state::{MotorState, RobotState};
use common::controller::VelocityData;

pub trait OpenCvHandler {
    fn handle_frame(&mut self, frame: &Mat, /*robot: &RobotState, motor: &MotorState*/) -> anyhow::Result<(VelocityData, String)>;
}
