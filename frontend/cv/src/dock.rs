use opencv::prelude::*;
use common::controller::VelocityData;
use crate::OpenCvHandler;

pub struct AutoDock;

impl OpenCvHandler for AutoDock {
    fn handle_frame(&mut self, _frame: &Mat) -> anyhow::Result<(VelocityData, String)> {
        Ok((VelocityData {
            forwards_left: 1.0,
            forwards_right: 1.0,
            strafing: 0.0,
            vertical: 0.0
        }, "Docking".to_owned()))
    }
}
