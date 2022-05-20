use bevy::prelude::*;
use crate::{CameraDisplay, ResetButton};
use crate::robot::RobotData;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup_ui)
            .add_system(button_system);
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.2, 0.2, 0.2);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const CAMERA_BACKGROUND: Color = Color::rgb(0.3, 0.3, 0.3);
const LEFT_PANEL_BACKGROUND: Color = Color::rgb(0.15, 0.15, 0.15);
const LEFT_PANEL_BUTTON_BACKGROUND: Color = Color::rgb(0.4, 0.4, 0.4);

#[derive(Component)]
pub struct InfoPanel;

#[derive(Component)]
pub struct CameraSelectionPanel;

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());

    // root node
    commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            align_items: AlignItems::FlexStart,
            align_content: AlignContent::SpaceAround,
            ..default()
        },
        ..default()
    }).with_children(|parent| {
        // left panel
        parent.spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(20.0), Val::Percent(100.0)),
                flex_wrap: FlexWrap::WrapReverse,
                align_items: AlignItems::FlexStart,
                align_content: AlignContent::FlexStart,
                ..default()
            },
            color: LEFT_PANEL_BACKGROUND.into(),
            ..default()
        }).with_children(|parent| {
            parent.spawn_bundle(
                create_rect(Val::Px(80.0))
            ).with_children(|parent| {
                parent.spawn_bundle(create_text("Acceleration: ", 20.0, &asset_server));
                parent.spawn_bundle(create_text("X: ", 15.0, &asset_server)).insert(RobotData::AccelerationX);
                parent.spawn_bundle(create_text("Y: ", 15.0, &asset_server)).insert(RobotData::AccelerationY);
                parent.spawn_bundle(create_text("Z: ", 15.0, &asset_server)).insert(RobotData::AccelerationZ);
            });

            parent.spawn_bundle(
                create_rect(Val::Px(80.0))
            ).with_children(|parent| {
                parent.spawn_bundle(create_text("Velocity: ", 20.0, &asset_server));
                parent.spawn_bundle(create_text("X: ", 15.0, &asset_server)).insert(RobotData::VelocityX);
                parent.spawn_bundle(create_text("Y: ", 15.0, &asset_server)).insert(RobotData::VelocityY);
                parent.spawn_bundle(create_text("Z: ", 15.0, &asset_server)).insert(RobotData::VelocityZ);
            });

            parent.spawn_bundle(
                create_rect(Val::Px(80.0))
            ).with_children(|parent| {
                parent.spawn_bundle(create_text("Position: ", 20.0, &asset_server));
                parent.spawn_bundle(create_text("X: ", 15.0, &asset_server)).insert(RobotData::PositionX);
                parent.spawn_bundle(create_text("Y: ", 15.0, &asset_server)).insert(RobotData::PositionY);
                parent.spawn_bundle(create_text("Z: ", 15.0, &asset_server)).insert(RobotData::PositionZ);
            });
            parent.spawn_bundle(create_divider());

            parent.spawn_bundle(
                create_rect(Val::Px(80.0))
            ).with_children(|parent| {
                parent.spawn_bundle(create_text("Angular Velocity: ", 20.0, &asset_server));
                parent.spawn_bundle(create_text("X: ", 15.0, &asset_server)).insert(RobotData::GyroVelocityX);
                parent.spawn_bundle(create_text("Y: ", 15.0, &asset_server)).insert(RobotData::GyroVelocityY);
                parent.spawn_bundle(create_text("Z: ", 15.0, &asset_server)).insert(RobotData::GyroVelocityZ);
            });

            parent.spawn_bundle(
                create_rect(Val::Px(80.0))
            ).with_children(|parent| {
                parent.spawn_bundle(create_text("Angle: ", 20.0, &asset_server));
                parent.spawn_bundle(create_text("Yaw: ", 15.0, &asset_server)).insert(RobotData::GyroAngleX);
                parent.spawn_bundle(create_text("Pitch: ", 15.0, &asset_server)).insert(RobotData::GyroAngleY);
                parent.spawn_bundle(create_text("Roll: ", 15.0, &asset_server)).insert(RobotData::GyroAngleZ);
            });
            parent.spawn_bundle(create_divider());

            parent.spawn_bundle(
                create_rect(Val::Px(80.0))
            ).with_children(|parent| {
                parent.spawn_bundle(create_text("Mag: ", 20.0, &asset_server));
                parent.spawn_bundle(create_text("X: ", 15.0, &asset_server)).insert(RobotData::MagX);
                parent.spawn_bundle(create_text("Y: ", 15.0, &asset_server)).insert(RobotData::MagY);
                parent.spawn_bundle(create_text("Z: ", 15.0, &asset_server)).insert(RobotData::MagZ);
            });

            parent.spawn_bundle(
                create_rect(Val::Px(80.0))
            ).with_children(|parent| {
                parent.spawn_bundle(create_text("Pressure: ", 20.0, &asset_server));
                parent.spawn_bundle(create_text("", 15.0, &asset_server)).insert(RobotData::Pressure);
            });

            parent.spawn_bundle(
                create_button()
            ).with_children(|parent| {
                parent.spawn_bundle(create_text("Reset State", 20.0, &asset_server));
            }).insert(ResetButton);
        });

        // center panel
        parent.spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(80.0), Val::Percent(100.0)),
                flex_wrap: FlexWrap::WrapReverse,
                align_content: AlignContent::FlexStart,
                ..default()
            },
            ..default()
        }).with_children(|parent| {
            // camera selectors
            parent.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(5.0)),
                    flex_wrap: FlexWrap::Wrap,
                    flex_direction: FlexDirection::Column,
                    align_content: AlignContent::FlexStart,
                    overflow: Overflow::Hidden,
                    ..default()
                },
                color: Color::rgb(0.0, 0.0, 0.0).into(),
                ..default()
            }).insert(CameraSelectionPanel);

            parent.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(95.0)),
                    ..default()
                },
                color: CAMERA_BACKGROUND.into(),
                ..default()
            }).with_children(|parent| {
                // camera
                parent.spawn_bundle(ImageBundle {
                    style: Style {
                        max_size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        margin: Rect::all(Val::Auto),
                        ..default()
                    },
                    ..default()
                }).insert(CameraDisplay);
            });
        });
    });
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >
) {
    for (interaction, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

pub fn create_rect(height: Val) -> impl Bundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), height),
            margin: Rect::all(Val::Px(5.0)),
            padding: Rect::all(Val::Px(5.0)),
            flex_wrap: FlexWrap::WrapReverse,
            align_items: AlignItems::FlexStart,
            align_content: AlignContent::FlexStart,
            overflow: Overflow::Hidden,
            ..default()
        },
        color: LEFT_PANEL_BUTTON_BACKGROUND.into(),
        ..default()
    }
}

pub fn create_button() -> impl Bundle {
    ButtonBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Auto),
            margin: Rect::all(Val::Px(5.0)),
            padding: Rect::all(Val::Px(5.0)),
            flex_wrap: FlexWrap::WrapReverse,
            align_items: AlignItems::FlexStart,
            align_content: AlignContent::FlexStart,
            overflow: Overflow::Hidden,
            ..default()
        },
        color: LEFT_PANEL_BUTTON_BACKGROUND.into(),
        ..default()
    }
}

pub fn create_text<S: Into<String>>(string: S, size: f32, asset_server: &AssetServer) -> impl Bundle {
    TextBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Px(size)),
            ..default()
        },
        text: Text::with_section(
            string,
            TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: size,
                color: Color::WHITE,
            },
            Default::default(),
        ),
        ..default()
    }
}

pub fn create_divider() -> impl Bundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Px(2.0)),
            margin: Rect {
                left: Val::Px(15.0),
                right: Val::Px(15.0),
                top: Val::Px(3.0),
                bottom: Val::Px(3.0)
            },
            ..default()
        },
        color: LEFT_PANEL_BUTTON_BACKGROUND.into(),
        ..default()
    }
}