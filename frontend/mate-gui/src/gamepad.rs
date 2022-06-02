use bevy::prelude::*;
use common::controller::DownstreamMessage;
use crate::Serial;

pub struct GamepadPlugin;

impl Plugin for GamepadPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(gamepad_connections)
            .add_system(gamepad_input)
        ;
    }
}

struct CurrentGamepad(Gamepad);

fn gamepad_connections(
    mut commands: Commands,
    current_gamepad: Option<Res<CurrentGamepad>>,
    mut gamepad_evr: EventReader<GamepadEvent>,
) {
    for GamepadEvent(id, kind) in gamepad_evr.iter() {
        match kind {
            GamepadEventType::Connected => {
                println!("New gamepad connected with ID: {:?}", id);

                if current_gamepad.is_none() {
                    commands.insert_resource(CurrentGamepad(*id));
                }
            }
            GamepadEventType::Disconnected => {
                println!("Lost gamepad connection with ID: {:?}", id);

                if let Some(CurrentGamepad(old_id)) = current_gamepad.as_deref() {
                    if old_id == id {
                        commands.remove_resource::<CurrentGamepad>();
                    }
                }
            }
            _ => {}
        }
    }
}

fn gamepad_input(
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    current_gamepad: Option<Res<CurrentGamepad>>,
    serial: Res<Serial>
) {
    let gamepad = if let Some(gp) = current_gamepad {
        gp.0
    } else {
        return;
    };

    let axis_lx = GamepadAxis(gamepad, GamepadAxisType::LeftStickX);
    let axis_ly = GamepadAxis(gamepad, GamepadAxisType::LeftStickY);
    let axis_rx = GamepadAxis(gamepad, GamepadAxisType::RightStickX);
    let axis_ry = GamepadAxis(gamepad, GamepadAxisType::RightStickY);

    if let (Some(lx), Some(ly), Some(rx), Some(ry)) = (axes.get(axis_lx), axes.get(axis_ly), axes.get(axis_rx), axes.get(axis_ry)) {
        let velocity = common::joystick_math(lx, ly, rx, ry);
        serial.3.send(DownstreamMessage::VelocityUpdate(velocity)).unwrap();
    }

    // TODO maybe handle buttons
}
