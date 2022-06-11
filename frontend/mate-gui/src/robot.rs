use std::thread;
use std::time::Duration;
use bevy::prelude::*;
use crossbeam::channel::{bounded, Receiver, Sender};
use common::controller::{DownstreamMessage, VelocityData};
use sensor_fusion::state;
use sensor_fusion::state::{MotorState, RobotState};
use crate::{AutoVelo, JoyVelo, ui, utils};

pub struct RobotPlugin;

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(serial_monitor)
            .add_event::<DataEvent>()
            .add_event::<StateEvent>()
            .add_system(handler_data)
            .add_system(handler_state)
            .add_system(update_displays_imu)
            .add_system(update_displays_controller)
            .add_system(send_velocity)
            .add_system(reset_handler)
            .add_system(estop_handler)
            .add_system(estop_display)
        ;
    }
}
pub struct DataEvent(pub RobotState);
pub struct StateEvent(pub MotorState);
pub struct Serial(Receiver<RobotState>, Receiver<MotorState>, pub Sender<SerialNotification>, pub Sender<DownstreamMessage>);

#[derive(Component)]
pub struct ResetButton;

#[derive(Component)]
pub struct EStopButton;
#[derive(Component)]
pub struct EStopText;

#[derive(Component)]
pub enum RobotData {
    AccelerationX,
    AccelerationY,
    AccelerationZ,

    VelocityX,
    VelocityY,
    VelocityZ,

    PositionX,
    PositionY,
    PositionZ,

    GyroVelocityX,
    GyroVelocityY,
    GyroVelocityZ,

    GyroAngleX,
    GyroAngleY,
    GyroAngleZ,

    MagX,
    MagY,
    MagZ,

    Pressure
}

#[derive(Component)]
pub enum ControllerData {
    SpeedSpForwardsLeft,
    SpeedSpForwardsRight,
    SpeedSpStrafing,
    SpeedSpVertical,

    AveragePing,
    LastPing,
}

fn serial_monitor(mut commands: Commands) {
    let (tx_data, rx_data) = bounded::<RobotState>(15);
    let (tx_state, rx_state) = bounded::<MotorState>(15);
    let (tx_notification, rx_notification) = bounded::<SerialNotification>(15);
    let (tx_command, rx_command) = bounded::<DownstreamMessage>(15);

    {
        thread::Builder::new()
            .name("IMU Serial Monitor".to_owned())
            .spawn(move || utils::error_boundary(|| communication::listen_to_imu(tx_data.clone(), rx_notification.clone())))
            .unwrap();
    }

    {
        thread::Builder::new()
            .name("Controller Serial Monitor".to_owned())
            .spawn(move || utils::error_boundary(|| communication::listen_to_controller(tx_state.clone(), rx_command.clone())))
            .unwrap();
    }

    {
        let tx_command = tx_command.clone();

        thread::Builder::new()
            .name("Controller Pinger".to_owned())
            .spawn(move || utils::error_boundary(|| loop {
                tx_command.send(DownstreamMessage::Ping)?;
                state::increment_ping();
                thread::sleep(Duration::from_millis(100));
            }))
            .unwrap();
    }

    commands.insert_resource(Serial(rx_data, rx_state, tx_notification, tx_command));
}

fn handler_data(mut ev_data: EventWriter<DataEvent>, serial: Res<Serial>) {
    for data in serial.0.try_iter().last().into_iter() {
        ev_data.send(DataEvent(data));
    }
}

fn handler_state(mut ev_state: EventWriter<StateEvent>, serial: Res<Serial>) {
    for state in serial.1.try_iter().last().into_iter() {
        ev_state.send(StateEvent(state));
    }
}

fn reset_handler(query: Query<&Interaction, (With<ResetButton>, Changed<Interaction>)>, serial: Res<Serial>) {
    for interaction in query.iter() {
        if let Interaction::Clicked = interaction {
            let _ = serial.2.try_send(SerialNotification::ResetState);
        }
    }
}

fn estop_handler(query: Query<&Interaction, (With<EStopButton>, Changed<Interaction>)>, serial: Res<Serial>) {
    for interaction in query.iter() {
        if let Interaction::Clicked = interaction {
            let _ = serial.3.try_send(DownstreamMessage::EmergencyStop);
        }
    }
}

fn estop_display(mut query: Query<&mut Text, With<EStopText>>, mut ev_state: EventReader<StateEvent>) {
    for StateEvent(state) in ev_state.iter() {
        for mut text in query.iter_mut() {
            for section in text.sections.iter_mut() {
                if state.emergency_stop {
                    section.style.color = ui::EMERGENCY_STOP_ACTIVE;
                } else {
                    section.style.color = Color::WHITE;
                }
            }
        }
    }
}

fn update_displays_imu(mut query: Query<(&mut Text, &RobotData)>, mut ev_data: EventReader<DataEvent>) {
    for DataEvent(state) in ev_data.iter() {
        for (mut text, data) in query.iter_mut() {
            if text.sections.len() == 1 {
                let mut new_section = text.sections[0].clone();
                new_section.value = String::new();
                text.sections.push(new_section);
            }
            if text.sections.len() == 2 {
                match data {
                    RobotData::AccelerationX => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.acceleration.x);
                    }
                    RobotData::AccelerationY => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.acceleration.y);
                    }
                    RobotData::AccelerationZ => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.acceleration.z);
                    }

                    RobotData::VelocityX => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.velocity.x);
                    }
                    RobotData::VelocityY => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.velocity.y);
                    }
                    RobotData::VelocityZ => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.velocity.z);
                    }

                    RobotData::PositionX => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.position.x);
                    }
                    RobotData::PositionY => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.position.y);
                    }
                    RobotData::PositionZ => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.position.z);
                    }

                    RobotData::GyroVelocityX => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.gyro_velocity.x);
                    }
                    RobotData::GyroVelocityY => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.gyro_velocity.y);
                    }
                    RobotData::GyroVelocityZ => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.gyro_velocity.z);
                    }

                    RobotData::GyroAngleX => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.gyro_angle.x);
                    }
                    RobotData::GyroAngleY => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.gyro_angle.y);
                    }
                    RobotData::GyroAngleZ => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.gyro_angle.z);
                    }

                    RobotData::MagX => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.mag.x);
                    }
                    RobotData::MagY => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.mag.y);
                    }
                    RobotData::MagZ => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.mag.z);
                    }

                    RobotData::Pressure => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.pressure);
                    }
                }
            }
        }
    }
}

fn update_displays_controller(mut query: Query<(&mut Text, &ControllerData)>, mut ev_data: EventReader<StateEvent>) {
    for StateEvent(state) in ev_data.iter() {
        for (mut text, data) in query.iter_mut() {
            if text.sections.len() == 1 {
                let mut new_section = text.sections[0].clone();
                new_section.value = String::new();
                text.sections.push(new_section);
            }
            if text.sections.len() == 2 {
                match data {
                    ControllerData::SpeedSpForwardsLeft => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.total_velocity.forwards_left);
                    }
                    ControllerData::SpeedSpForwardsRight => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.total_velocity.forwards_right);
                    }
                    ControllerData::SpeedSpStrafing => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.total_velocity.strafing);
                    }
                    ControllerData::SpeedSpVertical => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.2}", state.total_velocity.vertical);
                    }
                    ControllerData::AveragePing => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.4} us", state.average_ping / 1000.0);
                    }
                    ControllerData::LastPing => {
                        let section = &mut text.sections[1];
                        section.value = format!("{:.4} us", state.last_ping / 1000.0);
                    }
                }
            }
        }
    }
}

pub fn send_velocity(serial: Res<Serial>, joystick: Option<Res<JoyVelo>>, auto: Option<Res<AutoVelo>>) {
    let mut forwards_left = 0.0;
    let mut forwards_right = 0.0;
    let mut strafing = 0.0;
    let mut vertical = 0.0;

    if let Some(joy_velo) = joystick {
        forwards_left += joy_velo.0.forwards_left;
        forwards_right += joy_velo.0.forwards_right;
        strafing += joy_velo.0.strafing;
        vertical += joy_velo.0.vertical;
    }

    if let Some(auto) = auto {
        forwards_left += auto.0.forwards_left;
        forwards_right += auto.0.forwards_right;
        strafing += auto.0.strafing;
        vertical += auto.0.vertical;
    }

    let update = VelocityData {
        forwards_left,
        forwards_right,
        strafing,
        vertical
    }.clamp();

    let _ = serial.3.try_send(DownstreamMessage::VelocityUpdate(update));
}

pub enum SerialNotification {
    ResetState
}

mod communication {
    use sensor_fusion::state;
    use sensor_fusion::state::MotorState;
    use super::*;

    pub(super) fn listen_to_imu(tx_data: Sender<RobotState>, rx_notification: Receiver<SerialNotification>) -> anyhow::Result<!> {
        let mut state = RobotState::default();
        state.reset();

        serial::imu::listen(move |frame, makeup| {
            for command in rx_notification.try_iter() {
                match command {
                    SerialNotification::ResetState => {
                        state.reset();
                    }
                }
            }

            state::update_state(&frame, &mut state, makeup);

            tx_data.send(state.clone()).unwrap();

            Ok(())
        })
    }

    pub(super) fn listen_to_controller(tx_state: Sender<MotorState>, rx_command: Receiver<DownstreamMessage>) -> anyhow::Result<!> {
        let mut state = MotorState::default();

        serial::controller::listen(move |message| {
            state::handle_message(&message, &mut state);

            tx_state.send(state.clone()).unwrap();

            Ok(())
        }, Some(rx_command))
    }
}