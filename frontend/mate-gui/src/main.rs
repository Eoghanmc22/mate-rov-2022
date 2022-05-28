#![feature(never_type)]

mod ui;
mod video;
mod utils;
mod robot;
mod render3d;

use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use crate::ui::*;
use crate::video::*;
use crate::robot::*;
use crate::render3d::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(Render3D)
        .add_plugin(UiPlugin)
        //.add_plugin(VideoPlugin)
        .add_plugin(RobotPlugin)
        //.add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}
