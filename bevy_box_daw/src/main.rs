#![feature(let_chains)]

use crate::{
    mouse::{InitMyMouseTracking, MousePos, MyMouseTrackingPlugin},
    square::Square,
};
use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseMotion, MouseWheel},
        touchpad::{TouchpadMagnify, TouchpadRotate},
    },
    math::vec3,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};
use square::SquareCoords;

pub mod mouse;
pub mod square;
pub mod util;
pub mod wall;

const TIMESTEP: f64 = 1. / 60.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(Dragging(None))
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
        .insert_resource(Time::<Fixed>::from_seconds(TIMESTEP))
        .add_systems(Update, print_mouse_events_system)
        .add_systems(Update, drag_cursor_icon)
        .add_systems(Update, drag_start)
        .add_systems(Update, drag_move)
        .add_systems(Update, drag_end)
        .add_systems(Update, |mut q: Query<(&mut Transform, &SquareCoords)>| {
            for (mut transform, coords) in &mut q {
                transform.translation.z = coords.1 as f32;
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

fn setup(mut commands: Commands) {
    commands
        .spawn((Camera2dBundle::default(), MainCamera))
        .add(InitMyMouseTracking);
}

fn zoom_control_system(
    input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut OrthographicProjection>,
) {
    // projection.area.

    // projection.scale

    // if input.pressed(KeyCode::Minus) {
    //     projection.scale += 0.2;
    // }

    // if input.pressed(KeyCode::Equals) {
    //     projection.scale -= 0.2;
    // }

    // projection.scale = projection.scale.clamp(0.2, 5.);
}

fn update_camera_transform(
    mut transform: Query<&mut Transform, With<MainCamera>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.single();
    let mut transform = transform.single_mut();

    transform.translation = vec3(window.width() / 2.0, window.height() / 2.0, 0.0);
    transform.scale = vec3(1.0, -1.0, 1.0);
}

fn add_first_boxes(
    mut commands: Commands,
    mut drags: ResMut<Drags>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let rect = Rect::from_corners((100.0, 50.0).into(), (300.0, 250.0).into());

    commands.spawn(Square {
        coords: SquareCoords(rect, drags.0),
        mesh: MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Box::from_corners(
                    rect.min.extend(0.0),
                    rect.max.extend(0.0),
                )))
                .into(),
            material: materials.add(ColorMaterial::from(Color::PINK)),
            ..default()
        },
    });
    drags.0 += 1;

    let rect = Rect::from_corners((260.0, 170.0).into(), (360.0, 270.0).into());

    commands.spawn(Square {
        coords: SquareCoords(rect, drags.0),
        mesh: MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Box::from_corners(
                    rect.min.extend(0.0),
                    rect.max.extend(0.0),
                )))
                .into(),
            material: materials.add(ColorMaterial::from(Color::YELLOW)),
            ..default()
        },
    });
    drags.0 += 1;
}

#[derive(Resource)]
struct Dragging(Option<DragState>);

#[derive(Resource)]
struct Drags(usize);

#[derive(Debug)]
struct DragState {
    entity: Entity,
    down: Vec2,
    drag_no: usize,
    start_rect: Rect,
}

fn drag_cursor_icon(
    dragging: ResMut<Dragging>,
    mouse_pos: Res<MousePos>,
    mut windows: Query<&mut Window>,
    square: Query<&SquareCoords>,
) {
    let is_dragging = dragging.0.is_some();
    let is_hovering = square.iter().any(|coords| coords.0.contains(mouse_pos.0));

    windows.single_mut().cursor.icon = match (is_dragging, is_hovering) {
        (true, _) => CursorIcon::Grabbing,
        (false, true) => CursorIcon::Grab,
        (false, false) => CursorIcon::Default,
    };
}

fn drag_start(
    mut dragging: ResMut<Dragging>,
    mut drags: ResMut<Drags>,
    pos: Res<MousePos>,
    mouse: Res<Input<MouseButton>>,
    square: Query<(Entity, &SquareCoords)>,
) {
    if mouse.just_pressed(MouseButton::Left) && dragging.0.is_none() {
        info!("mouse at {:?}", pos);

        if let Some((entity, coords)) = square
            .iter()
            .filter(|(_, coords)| coords.0.contains(pos.0))
            .max_by_key(|(_, coords)| coords.1)
        {
            dragging.0 = Some(DragState {
                entity,
                down: pos.0,
                drag_no: drags.0,
                start_rect: coords.0,
            });
            drags.0 += 1;
        }
    }
}

fn drag_move(
    mut dragging: ResMut<Dragging>,
    pos: Res<MousePos>,
    mut square: Query<(Entity, &mut Mesh2dHandle, &mut SquareCoords)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if let Some(drag_state) = &mut dragging.0 {
        let (_, mut mesh_handle, mut coords) = square
            .iter_mut()
            .find(|s| s.0 == drag_state.entity)
            .unwrap();

        let d = pos.0 - drag_state.down;
        let mut new_rect = drag_state.start_rect;
        new_rect.min += d;
        new_rect.max += d;

        mesh_handle.0 = meshes.add(Mesh::from(shape::Box::from_corners(
            new_rect.min.extend(0.0),
            new_rect.max.extend(0.0),
        )));

        coords.0 = new_rect;
        coords.1 = drag_state.drag_no;
    }
}

fn drag_end(
    mut dragging: ResMut<Dragging>,
    mouse: Res<Input<MouseButton>>,
    mut square: Query<(Entity, &mut Mesh2dHandle, &SquareCoords)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if let Some(drag_state) = &mut dragging.0
        && mouse.just_released(MouseButton::Left)
    {
        let (_, mut mesh_handle, coords) = square
            .iter_mut()
            .find(|s| s.0 == drag_state.entity)
            .unwrap();

        mesh_handle.0 = meshes.add(Mesh::from(shape::Box::from_corners(
            coords.0.min.extend(0.0),
            coords.0.max.extend(0.0),
        )));

        dragging.0 = None;
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
