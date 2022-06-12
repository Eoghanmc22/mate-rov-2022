use opencv::prelude::*;
use common::controller::VelocityData;
use crate::OpenCvHandler;

pub struct AutoUp;

impl OpenCvHandler for AutoUp {
    fn handle_frame(&mut self, _frame: &Mat) -> anyhow::Result<(VelocityData, String)> {
        Ok((VelocityData {
            forwards_left: 0.0,
            forwards_right: 0.0,
            strafing: 0.0,
            vertical: 1.0
        }, "Going up!".to_owned()))
    }
}
