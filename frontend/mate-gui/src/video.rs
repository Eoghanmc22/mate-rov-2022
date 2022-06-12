use std::{mem, thread};
use std::sync::Arc;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureFormat};
use crossbeam::channel::{bounded, unbounded, Receiver, Sender};
use common::controller::VelocityData;
use cv::OpenCvHandler;
use crate::{CameraSelectionPanel, create_text, Serial, utils};
use crate::video::camera::{CameraEvent, StreamEvent};

pub struct VideoPlugin;

impl Plugin for VideoPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(stream_video)
            .insert_resource(AutoVelo(VelocityData::default()))
            .add_event::<AutoMsgEvent>()
            .add_system(display_addition)
            .add_system(select_camera)
            .add_system(stream_reader)
            .add_system(camera_reader)
            .add_system(update_displays_auto)
            .add_system(task_handler)
        ;
    }
}

struct Stream(Receiver<Image>, Sender<Image>, Sender<StreamEvent>, Receiver<CameraEvent>);
struct ImageHandle(Handle<Image>);
pub struct AutoVelo(pub VelocityData);
pub struct AutoMsgEvent(pub String);

#[derive(Component)]
pub struct CameraDisplay;


#[derive(Component)]
pub struct GoalDisplay;

#[derive(Component)]
pub struct OpenCvTaskButton(pub Box<dyn Fn() -> Option<Box<dyn OpenCvHandler + Send + Sync>> + Send + Sync>);

#[derive(Component)]
pub struct CameraSelector(i32);

fn stream_video(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let (tx_data, rx_data) = bounded::<Image>(1);
    let (tx_recycle, rx_recycle) = bounded::<Image>(1);
    let (tx_stream_event, rx_stream_event) = bounded::<StreamEvent>(1);
    let (tx_camera_event, rx_camera_event) = unbounded::<CameraEvent>();
    thread::Builder::new()
        .name("Video Provider".to_owned())
        .spawn(move || utils::error_boundary(|| camera::produce_stream(&tx_data, &rx_recycle, &rx_stream_event, tx_camera_event.clone())))
        .unwrap();

    let image = Image::default();
    tx_recycle.send(image.clone()).unwrap();
    let image_handle = images.add(image);
    commands.insert_resource(ImageHandle(image_handle));
    commands.insert_resource(Stream(rx_data, tx_recycle, tx_stream_event, rx_camera_event));

    //tx_stream_event.send(StreamEvent::OpenCamera { camera: 0 }).unwrap();
    //tx_stream_event.send(StreamEvent::OpenNamed { file: "test.mkv".to_owned() }).unwrap();
}

fn select_camera(query: Query<(&CameraSelector, &Interaction), Changed<Interaction>>, stream: Res<Stream>) {
    for (camera_selector, interaction) in query.iter() {
        if let Interaction::Clicked = interaction {
            let camera = camera_selector.0;
            stream.2.send(StreamEvent::OpenCamera { camera }).unwrap();
        }
    }
}

fn display_addition(mut query: Query<&mut UiImage, Added<CameraDisplay>>, image_handler: Res<ImageHandle>) {
    for mut image in query.iter_mut() {
        image.0 = image_handler.0.clone();
    }
}

fn stream_reader(stream: Res<Stream>, mut images: ResMut<Assets<Image>>, image_handle: Res<ImageHandle>) {
    let image_asset = images.get_mut(image_handle.0.clone()).unwrap();
    for image in stream.0.try_iter() {
        let image = mem::replace(image_asset, image);
        stream.1.send(image).unwrap();
    }
}

fn camera_reader(mut commands: Commands, panel_query: Query<Entity, With<CameraSelectionPanel>>, child_query: Query<Entity, With<CameraSelector>>, stream: Res<Stream>, asset_server: Res<AssetServer>, mut auto_velo: ResMut<AutoVelo>, mut msg_events: EventWriter<AutoMsgEvent>) {
    for event in stream.3.try_iter() {
        match event {
            CameraEvent::AvailableDevices { cameras } => {
                for display in panel_query.iter() {
                    let mut children_new = vec![];

                    for camera in &cameras {
                        children_new.push(
                            commands.spawn_bundle(ButtonBundle {
                                style: Style {
                                    size: Size::new(Val::Px(100.0), Val::Percent(100.0)),
                                    margin: Rect::all(Val::Px(5.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                ..default()
                            }).with_children(|parent| {
                                parent.spawn_bundle(create_text(format!("Camera {}", camera), 20.0, &asset_server));
                            }).insert(CameraSelector(*camera)).id()
                        );
                    }

                    for child in child_query.iter() {
                        commands.entity(child).despawn_recursive();
                    }

                    commands.entity(display).insert_children(0, &children_new);
                }
            }
            CameraEvent::AutonomousUpdate { velocity_data, goal_msg } => {
                auto_velo.0 = velocity_data;
                msg_events.send(AutoMsgEvent(goal_msg))
            }
        }
    }
}

fn update_displays_auto(mut query: Query<&mut Text, With<GoalDisplay>>, mut ev_data: EventReader<AutoMsgEvent>) {
    for AutoMsgEvent(msg) in ev_data.iter() {
        for mut text in query.iter_mut() {
            if text.sections.len() == 1 {
                let mut new_section = text.sections[0].clone();
                new_section.value = String::new();
                text.sections.push(new_section);
            }
            if text.sections.len() == 2 {
                let section = &mut text.sections[1];
                section.value = format!("{}", msg);
            }
        }
    }
}

fn task_handler(query: Query<(&Interaction, &OpenCvTaskButton), Changed<Interaction>>, stream: Res<Stream>) {
    for (interaction, task) in query.iter() {
        if let Interaction::Clicked = interaction {
            let _ = stream.2.try_send(StreamEvent::FrameHandler { processor: (task.0)() });
        }
    }
}

mod camera {
    use std::thread::{Builder, JoinHandle};
    use std::time::{Duration, Instant};
    use opencv::prelude::*;
    use common::controller::VelocityData;
    use cv::OpenCvHandler;
    use super::*;

    pub enum StreamEvent {
        OpenCamera {
            camera: i32
        },
        OpenNamed {
            file: String
        },
        FrameHandler {
            processor: Option<Box<dyn OpenCvHandler + Send + Sync>>
        },
        Close
    }

    pub enum CameraEvent {
        AvailableDevices {
            cameras: Vec<i32>
        },
        AutonomousUpdate {
            velocity_data: VelocityData,
            goal_msg: String
        }
    }

    pub(super) fn produce_stream(tx_data: &Sender<Image>, rx_recycle: &Receiver<Image>, rx_stream_event: &Receiver<StreamEvent>, tx_camera_event: Sender<CameraEvent>) -> anyhow::Result<!> {
        let mut video_capture = None;
        let mut selected_camera = None;
        let mut autodetect_thread_handle: Option<JoinHandle<anyhow::Result<Vec<i32>>>> = None;
        let mut last_cameras = Some(vec![]);

        let mut opencv_processor = None;
        let mut opencv_thread_handle: Option<JoinHandle<anyhow::Result<((VelocityData, String), Box<dyn OpenCvHandler + Send + Sync>)>>> = None;

        let mut mat_1 = Mat::default();
        let mut mat_2 = Mat::default();

        let mut last_camera_check = Instant::now();
        let camera_check_interval = Duration::from_secs(1);

        tx_camera_event.send(CameraEvent::AutonomousUpdate {
            velocity_data: VelocityData::default(),
            goal_msg: "No message".to_owned()
        })?;


        loop {
            for event in rx_stream_event.try_iter() {
                video_capture = None;
                selected_camera = None;

                if let Some(handle) = autodetect_thread_handle.take() {
                    last_cameras = Some(handle.join().unwrap()?);
                }
                if let Some(handle) = opencv_thread_handle.take() {
                    let result = handle.join().unwrap();
                    match result {
                        Ok(((velo, msg), handler)) => {
                            tx_camera_event.send(CameraEvent::AutonomousUpdate {
                                velocity_data: velo,
                                goal_msg: msg
                            })?;
                            opencv_processor = Some(handler);
                        }
                        Err(e) => {
                            tx_camera_event.send(CameraEvent::AutonomousUpdate {
                                velocity_data: VelocityData::default(),
                                goal_msg: "Error".to_owned()
                            })?;
                            println!("OpenCv error: {:?}", e);
                        }
                    }
                }

                match event {
                    StreamEvent::OpenCamera { camera } => {
                        let new_video_capture = opencv::videoio::VideoCapture::new(camera, opencv::videoio::CAP_ANY)?;
                        if new_video_capture.is_opened()? {
                            video_capture = Some(new_video_capture);
                            selected_camera = Some(camera);
                        }
                    }
                    StreamEvent::OpenNamed { ref file } => {
                        let new_video_capture = opencv::videoio::VideoCapture::from_file(file, opencv::videoio::CAP_ANY)?;
                        if new_video_capture.is_opened()? {
                            video_capture = Some(new_video_capture);
                        }
                    }
                    StreamEvent::Close => {}
                    StreamEvent::FrameHandler { processor } => {
                        tx_camera_event.send(CameraEvent::AutonomousUpdate {
                            velocity_data: VelocityData::default(),
                            goal_msg: "No message".to_owned()
                        })?;

                        opencv_processor = processor;
                    }
                };
            }

            if let Some(handle) = autodetect_thread_handle.take() {
                if handle.is_finished() {
                    last_cameras = Some(handle.join().unwrap()?);
                } else {
                    autodetect_thread_handle = Some(handle);
                }
            }

            if let Some(handle) = opencv_thread_handle.take() {
                if handle.is_finished() {
                    match result {
                        Ok(((velo, msg), handler)) => {
                            tx_camera_event.send(CameraEvent::AutonomousUpdate {
                                velocity_data: velo,
                                goal_msg: msg
                            })?;
                            opencv_processor = Some(handler);
                        }
                        Err(e) => {
                            tx_camera_event.send(CameraEvent::AutonomousUpdate {
                                velocity_data: VelocityData::default(),
                                goal_msg: "Error".to_owned()
                            })?;
                            println!("OpenCv error: {:?}", e);
                        }
                    }
                } else {
                    opencv_thread_handle = Some(handle);
                }
            }

            if let Some(mut last_cameras_taken) = last_cameras.take() {
                if last_camera_check.elapsed() > camera_check_interval {
                    let tx_camera_event = tx_camera_event.clone();
                    autodetect_thread_handle = Some(Builder::new()
                        .name("Detect Cameras".to_owned())
                        .spawn(move || {
                            detect_cameras(tx_camera_event, selected_camera, &mut last_cameras_taken)?;
                            Ok(last_cameras_taken)
                        })?);
                    last_camera_check = Instant::now();
                } else {
                    last_cameras = Some(last_cameras_taken);
                }
            }

            if let Some(video_capture) = &mut video_capture {
                if video_capture.read(&mut mat_1)? {
                    let opencv_mat = mat_1.clone();
                    if let Some(mut handler) = opencv_processor.take() {
                        opencv_thread_handle = Some(Builder::new()
                            .name("Opencv Processor".to_owned())
                            .spawn(move || {
                                let result = handler.handle_frame(&opencv_mat);
                                result.map(|it| (it, handler))
                            })?);
                    }

                    opencv::imgproc::cvt_color(&mat_1, &mut mat_2, opencv::imgproc::COLOR_BGR2BGRA, 0)?;
                    let data = mat_2.data_bytes()?;
                    let size = mat_2.size()?;

                    let mut image = rx_recycle.recv()?;
                    image.resize(Extent3d { width: size.width as u32, height: size.height as u32, depth_or_array_layers: 1 });
                    image.data.copy_from_slice(data);
                    image.texture_descriptor.format = TextureFormat::Bgra8UnormSrgb;
                    tx_data.send(image)?;
                } else {
                    // wait until next frame
                    let image = rx_recycle.recv()?;
                    tx_data.send(image)?;
                }
            } else {
                // wait until next frame
                let image = rx_recycle.recv()?;
                tx_data.send(image)?;
            }
        }
    }

    fn detect_cameras(tx_event: Sender<CameraEvent>, selected_camera: Option<i32>, last_cameras: &mut Vec<i32>) -> anyhow::Result<()> {
        let max_non_continuous = 10;
        let mut cameras = vec![];
        let mut missed = 0;

        cameras.extend(selected_camera.iter());

        for camera in 0.. {
            if Some(camera) == selected_camera {
                continue;
            }

            let video_capture = opencv::videoio::VideoCapture::new(camera, opencv::videoio::CAP_ANY)?;
            if video_capture.is_opened()? {
                cameras.push(camera);
                missed = 0;
            } else {
                missed += 1;
            }

            if missed > max_non_continuous {
                break;
            }
        }

        cameras.sort();

        if cameras != *last_cameras {
            tx_event.send(CameraEvent::AvailableDevices { cameras: cameras.clone() })?;
            *last_cameras = cameras;
        }

        Ok(())
    }
}
