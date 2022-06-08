use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use crate::{CameraDisplay, ControllerData, EStopButton, EStopText, ResetButton};
use crate::robot::RobotData;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup_ui)
            .add_system(button_system)
            .add_system(mouse_scroll)
        ;
    }
}

pub const NORMAL_BUTTON: Color = Color::rgb(0.2, 0.2, 0.2);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
pub const CAMERA_BACKGROUND: Color = Color::rgb(0.3, 0.3, 0.3);
pub const LEFT_PANEL_BACKGROUND: Color = Color::rgb(0.15, 0.15, 0.15);
pub const LEFT_PANEL_BUTTON_BACKGROUND: Color = Color::rgb(0.4, 0.4, 0.4);
pub const EMERGENCY_STOP_ACTIVE: Color = Color::rgb(1.0, 0.0, 0.0);

#[derive(Component)]
pub struct InfoPanel;

#[derive(Component)]
pub struct CameraSelectionPanel;

#[derive(Component, Default)]
struct ScrollingList {
    position: f32,
}

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
        color: LEFT_PANEL_BACKGROUND.into(),
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
            color: Color::NONE.into(),
            ..default()
        }).insert(
            ScrollingList::default()
        ).with_children(|parent| {
            parent.spawn_bundle(NodeBundle {
                style: Style {
                    flex_wrap: FlexWrap::WrapReverse,
                    align_items: AlignItems::FlexStart,
                    align_content: AlignContent::FlexStart,
                    ..default()
                },
                color: Color::NONE.into(),
                ..default()
            }).with_children(|parent| {
                parent.spawn_bundle(
                    create_rect()
                ).with_children(|parent| {
                    parent.spawn_bundle(create_text("Acceleration: ", 20.0, &asset_server));
                    parent.spawn_bundle(create_text("X: ", 15.0, &asset_server)).insert(RobotData::AccelerationX);
                    parent.spawn_bundle(create_text("Y: ", 15.0, &asset_server)).insert(RobotData::AccelerationY);
                    parent.spawn_bundle(create_text("Z: ", 15.0, &asset_server)).insert(RobotData::AccelerationZ);
                });

                parent.spawn_bundle(
                    create_rect()
                ).with_children(|parent| {
                    parent.spawn_bundle(create_text("Velocity: ", 20.0, &asset_server));
                    parent.spawn_bundle(create_text("X: ", 15.0, &asset_server)).insert(RobotData::VelocityX);
                    parent.spawn_bundle(create_text("Y: ", 15.0, &asset_server)).insert(RobotData::VelocityY);
                    parent.spawn_bundle(create_text("Z: ", 15.0, &asset_server)).insert(RobotData::VelocityZ);
                });

                parent.spawn_bundle(
                    create_rect()
                ).with_children(|parent| {
                    parent.spawn_bundle(create_text("Position: ", 20.0, &asset_server));
                    parent.spawn_bundle(create_text("X: ", 15.0, &asset_server)).insert(RobotData::PositionX);
                    parent.spawn_bundle(create_text("Y: ", 15.0, &asset_server)).insert(RobotData::PositionY);
                    parent.spawn_bundle(create_text("Z: ", 15.0, &asset_server)).insert(RobotData::PositionZ);
                });
                parent.spawn_bundle(create_divider());

                parent.spawn_bundle(
                    create_rect()
                ).with_children(|parent| {
                    parent.spawn_bundle(create_text("Angular Velocity: ", 20.0, &asset_server));
                    parent.spawn_bundle(create_text("X: ", 15.0, &asset_server)).insert(RobotData::GyroVelocityX);
                    parent.spawn_bundle(create_text("Y: ", 15.0, &asset_server)).insert(RobotData::GyroVelocityY);
                    parent.spawn_bundle(create_text("Z: ", 15.0, &asset_server)).insert(RobotData::GyroVelocityZ);
                });

                parent.spawn_bundle(
                    create_rect()
                ).with_children(|parent| {
                    parent.spawn_bundle(create_text("Angle: ", 20.0, &asset_server));
                    parent.spawn_bundle(create_text("Yaw: ", 15.0, &asset_server)).insert(RobotData::GyroAngleX);
                    parent.spawn_bundle(create_text("Pitch: ", 15.0, &asset_server)).insert(RobotData::GyroAngleY);
                    parent.spawn_bundle(create_text("Roll: ", 15.0, &asset_server)).insert(RobotData::GyroAngleZ);
                });
                parent.spawn_bundle(create_divider());

                parent.spawn_bundle(
                    create_rect()
                ).with_children(|parent| {
                    parent.spawn_bundle(create_text("Mag: ", 20.0, &asset_server));
                    parent.spawn_bundle(create_text("X: ", 15.0, &asset_server)).insert(RobotData::MagX);
                    parent.spawn_bundle(create_text("Y: ", 15.0, &asset_server)).insert(RobotData::MagY);
                    parent.spawn_bundle(create_text("Z: ", 15.0, &asset_server)).insert(RobotData::MagZ);
                });

                parent.spawn_bundle(
                    create_rect()
                ).with_children(|parent| {
                    parent.spawn_bundle(create_text("Motor Speeds: ", 20.0, &asset_server));
                    parent.spawn_bundle(create_text("Left: ", 15.0, &asset_server)).insert(ControllerData::SpeedSpForwardsLeft);
                    parent.spawn_bundle(create_text("Right: ", 15.0, &asset_server)).insert(ControllerData::SpeedSpForwardsRight);
                    parent.spawn_bundle(create_text("Strafing: ", 15.0, &asset_server)).insert(ControllerData::SpeedSpStrafing);
                    parent.spawn_bundle(create_text("Vertical: ", 15.0, &asset_server)).insert(ControllerData::SpeedSpVertical);
                });

                parent.spawn_bundle(
                    create_rect()
                ).with_children(|parent| {
                    parent.spawn_bundle(create_text("Pressure: ", 20.0, &asset_server));
                    parent.spawn_bundle(create_text("Psi: ", 15.0, &asset_server)).insert(RobotData::Pressure);
                });

                parent.spawn_bundle(
                    create_button()
                ).with_children(|parent| {
                    parent.spawn_bundle(create_text("Reset State", 20.0, &asset_server));
                }).insert(ResetButton);

                parent.spawn_bundle(
                    create_button()
                ).with_children(|parent| {
                    parent.spawn_bundle(create_text("Emergency Stop", 20.0, &asset_server)).insert(EStopText);
                }).insert(EStopButton);
            });
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

fn mouse_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_list: Query<(&mut ScrollingList, &mut Style, &Children, &Node)>,
    node_query: Query<&Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.iter() {
        for (mut scrolling_list, mut style, children, uinode) in query_list.iter_mut() {
            let items_height: f32 = children
                .iter()
                .map(|entity| node_query.get(*entity).unwrap().size.y)
                .sum();

            let panel_height = uinode.size.y;
            let max_scroll = (items_height - panel_height).max(0.0);
            let dy = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * 20.0,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };
            scrolling_list.position += dy;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.0);
            style.position.top = Val::Px(scrolling_list.position);
        }
    }
}

pub fn create_rect() -> impl Bundle {
    NodeBundle {
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