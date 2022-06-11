use opencv::core::Mat;
use common::controller::VelocityData;
use sensor_fusion::state::{MotorState, RobotState};
use crate::OpenCvHandler;

pub struct LineFollower;

/*impl OpenCvHandler for LineFollower {
    fn handle_frame(frame: &Mat, robot: &RobotState, motor: &MotorState) -> anyhow::Result<VelocityData> {
        todo!()
    }
}*/