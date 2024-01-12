#![feature(let_chains)]

use crate::{
    mouse::{InitMyMouseTracking, MyMouseTrackingPlugin},
    square::Square,
};
use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
        touchpad::{TouchpadMagnify, TouchpadRotate},
    },
    math::{vec2, vec3},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};
use mouse::MouseWorldPos;
use square::SquareCoords;

pub mod mouse;
pub mod square;
pub mod util;
pub mod wall;

const TIMESTEP: f64 = 1. / 60.0;

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
        .add_systems(Update, camera_movement)
        .insert_resource(Time::<Fixed>::from_seconds(TIMESTEP))
        .add_systems(Update, print_mouse_events_system)
        .add_systems(Update, drag_cursor_icon)
        .add_systems(Update, drag_start)
        .add_systems(Update, drag_move)
        .add_systems(Update, drag_end)
        .add_systems(Update, |mut q: Query<(&mut Transform, &SquareCoords)>| {
            for (mut transform, coords) in &mut q {
                transform.translation.z = coords.z as f32;
            }
        })
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

    commands
        .spawn((
            Camera2dBundle {
                transform: Transform::default()
                    .with_scale(vec3(1.0, -1.0, 1.0))
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
    // mut zoom_transform: ResMut<Zoom>,
) {
    let window = window.single();
    let window_size = Vec2::new(window.width(), window.height());

    let (mut transform, projection, _) = transform.single_mut();

    // This event will only fire on macOS
    for event in touchpad_magnify_events.read() {
        info!("{:?}", event);
        let d = event.0 * 1.3;
        transform.scale += vec3(-d, d, 0.0);
    }

    for event in mouse_wheel_events.read() {
        // zoom_transform.0.translation.x -= event.x;
        // zoom_transform.0.translation.y += event.y;

        info!("{:?}", event);

        let proj_size = projection.area.size();
        let world_units_per_device_pixel = proj_size / window_size;
        let delta_world = vec2(event.x, event.y) * world_units_per_device_pixel;
        let proposed_cam_transform = transform.translation - delta_world.extend(0.0);

        transform.translation = proposed_cam_transform;
    }

    // camera_query.single().0.view

    // if input.pressed(KeyCode::Minus) {
    //     projection.scale += 0.2;
    // }

    // if input.pressed(KeyCode::Equals) {
    //     projection.scale -= 0.2;
    // }

    // projection.scale = projection.scale.clamp(0.2, 5.);
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

fn add_first_boxes(
    mut commands: Commands,
    mut drags: ResMut<Drags>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let pos = vec2(100.0, 50.0);
    let size = vec2(200.0, 200.0);

    commands.spawn(Square {
        coords: SquareCoords {
            pos,
            size,
            z: drags.0,
        },
        mesh: MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Box::from_corners(
                    pos.extend(0.0),
                    (pos + size).extend(0.0),
                )))
                .into(),
            material: materials.add(ColorMaterial::from(Color::PINK)),
            ..default()
        },
    });
    drags.0 += 1;

    let pos = vec2(260.0, 170.0);
    let size = vec2(100.0, 100.0);

    commands.spawn(Square {
        coords: SquareCoords {
            pos,
            size,
            z: drags.0,
        },
        mesh: MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Box::from_corners(
                    pos.extend(0.0),
                    (pos + size).extend(0.0),
                )))
                .into(),
            material: materials.add(ColorMaterial::from(Color::YELLOW)),
            ..default()
        },
    });
    drags.0 += 1;
}

#[derive(Resource)]
struct Drags(usize);

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
    square: Query<&SquareCoords>,
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
    square: Query<(Entity, &SquareCoords)>,
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
    mut dragging: Query<(Entity, &DragState, &mut Mesh2dHandle, &mut SquareCoords)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if let Some((_, drag_state, mut mesh_handle, mut coords)) = dragging.get_single_mut().ok() {
        let d = mouse_pos.0 - drag_state.down;

        mesh_handle.0 = meshes.add(Mesh::from(shape::Box::from_corners(
            (drag_state.start_pos + d).extend(0.0),
            (drag_state.start_pos + d + coords.size).extend(0.0),
        )));

        coords.pos = drag_state.start_pos + d;
        coords.z = drag_state.drag_no;
    }
}

fn drag_end(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    mut dragging: Query<(Entity, &DragState, &mut Mesh2dHandle, &SquareCoords)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if let Some((entity, _, mut mesh_handle, coords)) = dragging.get_single_mut().ok()
        && mouse.just_released(MouseButton::Left)
    {
        mesh_handle.0 = meshes.add(Mesh::from(shape::Box::from_corners(
            coords.pos.extend(0.0),
            (coords.pos + coords.size).extend(0.0),
        )));

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
