#![feature(let_chains)]

use crate::mouse::{InitMyMouseTracking, MyMouseTrackingPlugin};
use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
        touchpad::{TouchpadMagnify, TouchpadRotate},
    },
    math::{vec2, vec3},
    prelude::*,
    window::PrimaryWindow,
};
use mouse::MouseWorldPos;
use square::{add_first_boxes, update_dialog_node_meshes, DialogInfo};

pub mod mouse;
pub mod square;
pub mod util;
pub mod wall;

// const TIMESTEP: f64 = 1. / 60.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(Drags(0))
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (800., 600.).into(),
                    ..default()
                }),
                ..default()
            }),
            MyMouseTrackingPlugin,
        ))
        .add_systems(Startup, (setup, add_first_boxes).chain())
        .add_systems(Update, zoom_control_system)
        .add_systems(Update, update_camera_transform)
        .add_systems(Update, camera_movement)
        // .insert_resource(Time::<Fixed>::from_seconds(TIMESTEP))
        .add_systems(Update, print_mouse_events_system)
        .add_systems(Update, drag_cursor_icon)
        .add_systems(Update, drag_start)
        .add_systems(Update, drag_move)
        .add_systems(Update, drag_end)
        .add_systems(Update, update_dialog_node_meshes)
        // .add_systems(FixedUpdate, |pos: Res<MousePos>| {
        //     info!("mouse at {:?}", pos);
        // })
        //         .add_systems(Update, update_counter_text)
        .add_systems(Update, bevy::window::close_on_esc)
        //         // .add_systems(Update, screen_to_world_system)
        .run();
}

#[derive(Component)]
struct MainCamera;

// #[derive(Resource)]
// struct Zoom(Transform);

fn setup(mut commands: Commands, window: Query<&Window, With<PrimaryWindow>>) {
    let window = window.single();

    // window *

    commands
        .spawn((
            Camera2dBundle {
                transform: Transform::default()
                    .with_scale(vec3(1.0, 1.0, 1.0))
                    .with_translation(vec3(window.width() / 2.0, window.height() / 2.0, 0.0)),
                ..Default::default()
            },
            MainCamera,
        ))
        .add(InitMyMouseTracking);

    // commands.insert_resource(Zoom(Transform::default()));
}

fn zoom_control_system(
    mut touchpad_magnify_events: EventReader<TouchpadMagnify>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    // mut camera_query: Query<(&Camera, &mut OrthographicProjection)>,
    mut transform: Query<(&mut Transform, &OrthographicProjection, With<MainCamera>)>,
    window: Query<&Window, With<PrimaryWindow>>,
    mouse_pos: Res<MouseWorldPos>,
    // mut zoom_transform: ResMut<Zoom>,
) {
    let window = window.single();
    let window_size = Vec2::new(window.width(), window.height());

    let (mut transform, projection, _) = transform.single_mut();

    println!("camera transform: {:?}", transform);

    // // This event will only fire on macOS
    // for event in touchpad_magnify_events.read() {
    //     let old_scale = transform.scale.x.abs();
    //     info!("{old_scale}, {:?}", event);
    //     let new_scale = event.0 * 1.5 * old_scale;

    //     // transform.scale += vec3(new_scale, -new_scale, 0.0);

    //     // transform.translation = vec3(
    //     //     mouse_pos.0.x - (new_scale / old_scale) * (mouse_pos.0.x - transform.translation.x), // - window.width() / 2.0,
    //     //     mouse_pos.0.y - (new_scale / old_scale) * (mouse_pos.0.y - transform.translation.y), // - window.height() / 2.0,
    //     //     0.0,
    //     // );
    // }

    // for event in mouse_wheel_events.read() {
    //     // zoom_transform.0.translation.x -= event.x;
    //     // zoom_transform.0.translation.y += event.y;

    //     info!("{:?}", event);

    //     let proj_size = projection.area.size();
    //     let world_units_per_device_pixel = proj_size / window_size;
    //     let delta_world = vec2(event.x, event.y) * world_units_per_device_pixel;
    //     let proposed_cam_transform = transform.translation - delta_world.extend(0.0);

    //     transform.translation = proposed_cam_transform;
    // }
}

fn update_camera_transform(
    mut transform: Query<&mut Transform, With<MainCamera>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.single();
    let mut transform = transform.single_mut();

    // transform.translation = vec3(window.width() / 2.0, window.height() / 2.0, 0.0);
    // transform.scale = vec3(1.0, -1.0, 1.0);
}

fn camera_movement(
    mut transform: Query<(&mut Transform, &OrthographicProjection, With<MainCamera>)>,
    window: Query<&Window, With<PrimaryWindow>>,
    // zoom_transform: Res<Zoom>,
) {
    // let window = window.single();
    // let window_size = Vec2::new(window.width(), window.height());

    // let (mut transform, projection, _) = transform.single_mut();

    // let proj_size = projection.area.size();
    // let world_units_per_device_pixel = proj_size / window_size;

    // *transform.as_mut() = Transform::default()
    //     .with_scale(vec3(1.0, -1.0, 1.0))
    //     .with_translation(vec3(window.width() / 2.0, window.height() / 2.0, 0.0));
    // //* zoom_transform.0;

    // // transform.translation = vec3(window.width() / 2.0, window.height() / 2.0, 0.0);
    // // transform.scale = vec3(1.0, -1.0, 1.0);
}

#[derive(Resource)]
pub struct Drags(pub usize);

#[derive(Debug, Component)]
struct DragState {
    down: Vec2,
    drag_no: usize,
    start_pos: Vec2,
}

fn drag_cursor_icon(
    dragging: Query<&DragState>,
    mouse_pos: Res<MouseWorldPos>,
    mut windows: Query<&mut Window>,
    square: Query<&DialogInfo>,
) {
    let is_dragging = !dragging.is_empty();
    let is_hovering = square.iter().any(|coords| coords.contains(mouse_pos.0));

    windows.single_mut().cursor.icon = match (is_dragging, is_hovering) {
        (true, _) => CursorIcon::Grabbing,
        (false, true) => CursorIcon::Grab,
        (false, false) => CursorIcon::Default,
    };
}

fn drag_start(
    mut commands: Commands,
    dragging: Query<&DragState>,
    mut drags: ResMut<Drags>,
    mouse_pos: Res<MouseWorldPos>,
    mouse: Res<Input<MouseButton>>,
    square: Query<(Entity, &DialogInfo)>,
) {
    if mouse.just_pressed(MouseButton::Left) && dragging.is_empty() {
        if let Some((entity, coords)) = square
            .iter()
            .filter(|&(_, coords)| coords.contains(mouse_pos.0))
            .max_by_key(|&(_, coords)| coords.z)
        {
            commands.get_entity(entity).unwrap().insert(DragState {
                down: mouse_pos.0,
                drag_no: drags.0,
                start_pos: coords.pos,
            });

            drags.0 += 1;
        }
    }
}

fn drag_move(
    mouse_pos: Res<MouseWorldPos>,
    mut dragging: Query<(Entity, &DragState, &mut DialogInfo)>,
) {
    if let Some((_, drag_state, mut coords)) = dragging.get_single_mut().ok() {
        let d = mouse_pos.0 - drag_state.down;

        coords.pos = drag_state.start_pos + d;
        coords.z = drag_state.drag_no;
    }
}

fn drag_end(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    mut dragging: Query<(Entity, &DragState, &DialogInfo)>,
) {
    if let Some((entity, _, coords)) = dragging.get_single_mut().ok()
        && mouse.just_released(MouseButton::Left)
    {
        commands.get_entity(entity).unwrap().remove::<DragState>();
    }
}

/// This system prints out all mouse events as they come in
fn print_mouse_events_system(
    mut mouse_button_input_events: EventReader<MouseButtonInput>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut touchpad_magnify_events: EventReader<TouchpadMagnify>,
    mut touchpad_rotate_events: EventReader<TouchpadRotate>,
) {
    // for event in mouse_button_input_events.read() {
    //     info!("{:?}", event);
    // }

    // for event in mouse_motion_events.read() {
    //     info!("{:?}", event);
    // }

    // for event in cursor_moved_events.read() {
    //     info!("{:?}", event);
    // }

    // for event in mouse_wheel_events.read() {
    //     info!("{:?}", event);
    // }

    // // This event will only fire on macOS
    // for event in touchpad_magnify_events.read() {
    //     info!("{:?}", event);
    // }

    // This event will only fire on macOS
    // for event in touchpad_rotate_events.read() {
    //     info!("{:?}", event);
    // }
}
