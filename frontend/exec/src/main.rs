mod high_pass;

use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use common::vec::Vec3d;
use crate::high_pass::HighPass3d;

use eframe::{CreationContext, egui};

#[derive(Clone, Debug, Default)]
pub struct State {
    pub acceleration: Vec3d,
    pub velocity: Vec3d,
    pub position: Vec3d,

    pub gyro_rate: Vec3d,
    pub gyro_angle: Vec3d,

    pub mag: Vec3d,

    pub collection_duration: Duration,
    pub total_duration: Duration
}

fn main() {
    let state_atomic = Arc::new(AtomicPtr::new(Box::leak(Box::new(State::default()))));

    {
        let mut state = State::default();
        let state_atomic = state_atomic.clone();
        let mut acceleration_filter = None;
        let mut velocity_filter = HighPass3d::new(Vec3d::uniform(0.0), 0.02);
        let mut gyro_angle_filter = HighPass3d::new(Vec3d::uniform(0.0), 0.02);
        thread::spawn(|| {
            serial::listen_to_port("COM6", &mut move |frame| {
                state.acceleration = frame.acceleration;
                state.gyro_rate = frame.gyro;
                state.mag = frame.mag;
                state.collection_duration = frame.collection_duration;
                state.total_duration = frame.total_duration;

                let duration = frame.total_duration.as_secs_f64();

                if let None = acceleration_filter {
                    acceleration_filter = Some(HighPass3d::new(state.acceleration, 0.02));
                }

                let acceleration_filter = acceleration_filter.as_mut().unwrap();

                state.velocity += acceleration_filter.filter(state.acceleration, duration) * duration;
                state.position += velocity_filter.filter(state.velocity, duration) * duration;

                state.gyro_angle += gyro_angle_filter.filter(state.gyro_rate, duration) * duration;

                let last = state_atomic.swap(Box::leak(Box::new(state.clone())), Ordering::SeqCst);
                let _ = unsafe { Box::from_raw(last) };
            }).unwrap();
        });
    }

    let native_options = eframe::NativeOptions::default();
    let state_atomic = state_atomic.clone();
    eframe::run_native("Test", native_options, Box::new(|cc| Box::new(Application::new(cc, state_atomic))));
}

struct Application {
    state: Arc<AtomicPtr<State>>,
}

impl Application {
    pub fn new(cc: &CreationContext, state: Arc<AtomicPtr<State>>) -> Self {
        Self { state }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let state = unsafe { &*self.state.load(Ordering::SeqCst) };
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Demo App");
            ui.spacing();

            ui.label(format!("accel_x: {:?}", state.acceleration.x));
            ui.label(format!("accel_y: {:?}", state.acceleration.y));
            ui.label(format!("accel_z: {:?}", state.acceleration.z));
            ui.spacing();

            ui.label(format!("velo_x: {:?}", state.velocity.x));
            ui.label(format!("velo_y: {:?}", state.velocity.y));
            ui.label(format!("velo_z: {:?}", state.velocity.z));
            ui.spacing();

            ui.label(format!("pos_x: {:?}", state.position.x));
            ui.label(format!("pos_y: {:?}", state.position.y));
            ui.label(format!("pos_z: {:?}", state.position.z));
            ui.spacing();

            ui.label(format!("gyro_rate_x: {:?}", state.gyro_rate.x));
            ui.label(format!("gyro_rate_y: {:?}", state.gyro_rate.y));
            ui.label(format!("gyro_rate_z: {:?}", state.gyro_rate.z));
            ui.spacing();

            ui.label(format!("gyro_angle_x: {:?}", state.gyro_angle.x));
            ui.label(format!("gyro_angle_y: {:?}", state.gyro_angle.y));
            ui.label(format!("gyro_angle_z: {:?}", state.gyro_angle.z));
            ui.spacing();

            ui.label(format!("mag_x: {:?}", state.mag.x));
            ui.label(format!("mag_y: {:?}", state.mag.y));
            ui.label(format!("mag_z: {:?}", state.mag.z));
            ui.spacing();

            ui.label(format!("collection_duration: {}", state.collection_duration.as_millis()));
            ui.label(format!("total_duration: {:?}", state.total_duration.as_millis()));
        });

        ctx.request_repaint();
    }
}