#![feature(never_type)]

mod render3d;
mod ui;
mod video;
mod robot;
mod gamepad;
mod utils;

use bevy::prelude::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use crate::render3d::*;
use crate::ui::*;
use crate::video::*;
use crate::robot::*;
use crate::gamepad::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        //.add_plugin(Render3D)
        .add_plugin(UiPlugin)
        .add_plugin(VideoPlugin)
        .add_plugin(RobotPlugin)
        .add_plugin(GamepadPlugin)
        //.add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .run();
}
