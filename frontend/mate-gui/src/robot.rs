use std::sync::Arc;
use std::thread;
use bevy::prelude::*;
use crossbeam::channel::{bounded, Receiver, Sender};
use sensor_fusion::state::RobotState;
use serial::commands::Command;
use crate::utils;

pub struct RobotPlugin;

impl Plugin for RobotPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(serial_monitor)
            .add_event::<DataEvent>()
            .add_system(handler_data)
            .add_system(update_displays)
            .add_system(reset_handler)
        ;
    }
}
pub struct DataEvent(pub RobotState);
struct Serial(Receiver<RobotState>, Sender<SerialNotification>, Arc<Sender<Command>>);

#[derive(Component)]
pub struct ResetButton;

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

fn serial_monitor(mut commands: Commands) {
    let (tx_data, rx_data) = bounded::<RobotState>(10);
    let (tx_notification, rx_notification) = bounded::<SerialNotification>(10);
    let (tx_command, rx_command) = bounded::<Command>(10);
    let tx_command = Arc::new(tx_command);
    {
        let tx_command = tx_command.clone();
        thread::Builder::new()
            .name("Serial Monitor".to_owned())
            .spawn(move || utils::error_boundary(|| communication::listen_to_serial(&tx_data, &rx_notification, &rx_command, tx_command.clone())))
            .unwrap();
    }

    commands.insert_resource(Serial(rx_data, tx_notification, tx_command));
}

fn handler_data(mut ev_data: EventWriter<DataEvent>, serial: Res<Serial>) {
    for state in serial.0.try_iter().last().into_iter() {
        ev_data.send(DataEvent(state));
    }
}

fn reset_handler(query: Query<&Interaction, (With<ResetButton>, Changed<Interaction>)>, serial: Res<Serial>) {
    for interaction in query.iter() {
        if let Interaction::Clicked = interaction {
            serial.1.send(SerialNotification::ResetState).unwrap();
        }
    }
}

fn update_displays(mut query: Query<(&mut Text, &RobotData)>, mut ev_data: EventReader<DataEvent>) {
    for DataEvent(state) in ev_data.iter() {
        for (mut text, data) in query.iter_mut() {
            if text.sections.len() == 1 {
                let mut new_section = text.sections[0].clone();
                new_section.value = String::new();
                text.sections.push(new_section);
            } else if text.sections.len() == 2 {
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
                        section.value = format!("Psi: {:.2}", state.pressure);
                    }
                }
            }
        }
    }
}

pub enum SerialNotification {
    ResetState
}

mod communication {
    use glam::vec3;
    use sensor_fusion::state::update_state;
    use serial::commands::Command;
    use super::*;

    pub(super) fn listen_to_serial(tx_data: &Sender<RobotState>, rx_notification: &Receiver<SerialNotification>, rx_command: &Receiver<Command>, tx_command: Arc<Sender<Command>>) -> anyhow::Result<!> {
        let mut state = RobotState {
            first_read: true,
            ..default()
        };

        serial::serial::listen(move |frame| {
            for command in rx_notification.try_iter() {
                match command {
                    SerialNotification::ResetState => {
                        state.velocity = Default::default();
                        state.position = Default::default();

                        state.angle = Default::default();

                        state.first_read = true;
                    }
                }
            }

            tx_command.send(Command::VelocityUpdate(vec3(32.4, 56.7, 21.3), 0.0))?;

            update_state(&frame, &mut state);

            tx_data.send(state.clone())?;

            Ok(())
        }, Some(rx_command))
    }
}