use std::sync::atomic::{AtomicUsize, Ordering};
use anyhow::bail;
use opencv::*;
use opencv::core::Vector;
use opencv::prelude::*;
use common::controller::VelocityData;
use crate::OpenCvHandler;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct ImageProducer;

impl OpenCvHandler for ImageProducer {
    fn handle_frame(&mut self, frame: &Mat) -> anyhow::Result<(VelocityData, String)> {
        let mat = frame.clone();
        let idx = COUNTER.fetch_add(1, Ordering::AcqRel);
        imgcodecs::imwrite(&format!("img_{}.jpg", idx), &mat, &Vector::default())?;
        bail!("Image created successfully")
    }
}

pub fn reset_counter() {
    COUNTER.store(0, Ordering::Release);
}
