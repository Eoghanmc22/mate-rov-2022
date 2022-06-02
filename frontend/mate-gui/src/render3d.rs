use bevy::core_pipeline;
use bevy::core_pipeline::{AlphaMask3d, Opaque3d, Transparent3d};
use bevy::prelude::*;
use bevy::render::camera::{ActiveCamera, CameraTypePlugin, RenderTarget};
use bevy::render::{render_graph, RenderApp, RenderStage};
use bevy::render::render_graph::{NodeRunError, RenderGraph, RenderGraphContext, SlotValue};
use bevy::render::render_phase::RenderPhase;
use bevy::render::renderer::RenderContext;
use bevy::window::{CreateWindow, PresentMode, WindowId};
use glam::vec3;
use crate::DataEvent;

pub struct Render3D;

impl Plugin for Render3D {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app.add_system_to_stage(RenderStage::Extract, extract_second_camera_phases);

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        let display_3d_node = render_graph.add_node("3d display", Display3DDriverNode);
        render_graph
            .add_node_edge(
                core_pipeline::node::MAIN_PASS_DEPENDENCIES,
                display_3d_node,
            )
            .unwrap();
        render_graph
            .add_node_edge(core_pipeline::node::CLEAR_PASS_DRIVER, display_3d_node)
            .unwrap();

        app
            .add_plugin(CameraTypePlugin::<DisplayCamera3d>::default())
            .insert_resource(Msaa { samples: 4 })
            .add_startup_system(setup_3d)
            .add_startup_system(create_new_window)
            .add_system(handle_imu)
        ;
    }
}

struct Display3DDriverNode;
impl render_graph::Node for Display3DDriverNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        if let Some(camera) = world.resource::<ActiveCamera<DisplayCamera3d>>().get() {
            graph.run_sub_graph(
                core_pipeline::draw_3d_graph::NAME,
                vec![SlotValue::Entity(camera)],
            )?;
        }

        Ok(())
    }
}

fn extract_second_camera_phases(
    mut commands: Commands,
    active: Res<ActiveCamera<DisplayCamera3d>>,
) {
    if let Some(entity) = active.get() {
        commands.get_or_spawn(entity).insert_bundle((
            RenderPhase::<Opaque3d>::default(),
            RenderPhase::<AlphaMask3d>::default(),
            RenderPhase::<Transparent3d>::default(),
        ));
    }
}

#[derive(Component, Default)]
struct DisplayCamera3d;

#[derive(Component)]
struct Imu;

fn create_new_window(mut create_window_events: EventWriter<CreateWindow>, mut commands: Commands) {
    let window_id = WindowId::new();

    create_window_events.send(CreateWindow {
        id: window_id,
        descriptor: WindowDescriptor {
            width: 800.,
            height: 600.,
            present_mode: PresentMode::Mailbox,
            title: "3d display window".to_string(),
            ..default()
        },
    });

    let eye = vec3(-2.0, -2.0, 2.0);
    let target = Vec3::ZERO;

    commands.spawn_bundle(PerspectiveCameraBundle {
        camera: Camera {
            target: RenderTarget::Window(window_id),
            ..default()
        },
        transform: Transform::from_translation(eye).looking_at(target, Vec3::Z),
        marker: DisplayCamera3d,
        ..PerspectiveCameraBundle::new()
    });
}

fn setup_3d(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // box
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(2.0, 3.5, 0.1))),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    }).insert(Imu);


    // x
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 0.1, 0.1))),
        material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
        ..default()
    });
    // x+
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(0.1, 0.1, 0.1))),
        transform: Transform::from_xyz(0.45, 0.0, 0.1),
        material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
        ..default()
    });
    // y
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(0.1, 1.0, 0.1))),
        material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
        ..default()
    });
    // y+
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(0.1, 0.1, 0.1))),
        transform: Transform::from_xyz(0.0, 0.45, 0.1),
        material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
        ..default()
    });
    // z
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Box::new(0.1, 0.1, 1.0))),
        material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
        ..default()
    });

    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 4.0, 8.0),
        ..default()
    });
}

fn handle_imu(mut query: Query<&mut Transform, With<Imu>>, mut ev_data: EventReader<DataEvent>) {
    for DataEvent(state) in ev_data.iter() {
        for mut transform in query.iter_mut() {
            if (1.0 - state.angle.length()).abs() > 0.01 {
                println!("bad quat");
            }
            transform.rotation = state.angle;
        }
    }
}