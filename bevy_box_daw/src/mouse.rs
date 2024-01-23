use bevy::{ecs::system::EntityCommand, prelude::*, window::PrimaryWindow};

#[derive(Debug, Resource, Clone, Copy, PartialEq)]
pub struct MousePos(pub Vec2);

#[derive(Debug, Resource, Clone, Copy, PartialEq)]
pub struct MouseWorldPos(pub Vec2);

pub struct MyMouseTrackingPlugin;

impl Plugin for MyMouseTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}

fn update(
    window: Query<Entity, With<PrimaryWindow>>,
    camera: Query<(&GlobalTransform, &Camera)>,
    mut movement: EventReader<CursorMoved>,
    mut pos: ResMut<MousePos>,
    mut world_pos: ResMut<MouseWorldPos>,
) {
    let (camera_transform, camera) = camera.single();
    let window = window.single();

    for event in movement.read() {
        if event.window == window {
            pos.0 = event.position;

            world_pos.0 = camera
                .viewport_to_world_2d(camera_transform, pos.0)
                .unwrap();
        }
    }
}

/// Added to `Camera2dBundle` bundle, hence it's an `EntityCommand` instead of a regular `Command`
pub struct InitMyMouseTracking;

impl EntityCommand for InitMyMouseTracking {
    fn apply(self, _: Entity, world: &mut World) {
        let window = world
            .query::<(&Window, With<PrimaryWindow>)>()
            .single(world)
            .0;

        let pos = window.cursor_position().unwrap_or_default();
        world.insert_resource(MousePos(pos));

        // can't do this yet, because of initialization ordering stuff...
        // ===
        // let (camera_transform, camera) = world.query::<(&GlobalTransform, &Camera)>().single(world);
        world.insert_resource(MouseWorldPos(
            // camera.viewport_to_world_2d(camera_transform, pos).unwrap(),
            pos,
        ));
    }
}

// const HOLD_TIME_THRESHOLD: f32 = 0.3;

// #[derive(Component)]
// pub struct Click {
//     pub offset: Vec2,
//     pub time: Stopwatch,
// }

// #[derive(Resource)]
// pub struct Holding(pub bool);

// pub fn handle_clicking(
//     mut counter: ResMut<Counter>,
//     buttons: Res<Input<MouseButton>>,
//     time: Res<Time>,
//     mut holding: ResMut<Holding>,
//     q_mouse: Query<&MousePosWorld>,
//     mut q_square: Query<(&mut Click, &mut Transform, &SquareCoordinates, &Square)>,
// ) {
//     let (mut click, _, square_coordinates, square) = q_square.single_mut();
//     let mouse = *q_mouse.single();

//     if validate_location(mouse.x, square_coordinates.0.x, square.size.x)
//         && validate_location(mouse.y, square_coordinates.0.y, square.size.y)
//     {
//         if buttons.just_released(MouseButton::Left) && holding.0 {
//             holding.0 = false;
//             return;
//         }

//         if buttons.just_pressed(MouseButton::Left) {
//             click.time.reset();
//         }

//         if buttons.pressed(MouseButton::Left) {
//             click.time.tick(time.delta());

//             if click.time.elapsed_secs() > HOLD_TIME_THRESHOLD && !holding.0 {
//                 click.offset = get_click_offset(square_coordinates.0.truncate(), mouse.truncate());
//                 holding.0 = true;
//             }
//         }
//         if buttons.just_released(MouseButton::Left)
//             && click.time.elapsed_secs() < HOLD_TIME_THRESHOLD
//             && !holding.0
//         {
//             counter.count += 1;
//         }
//     }
// }

// fn validate_location(pos: f32, coord: f32, size: f32) -> bool {
//     let halfs = size / 2.;

//     pos <= coord + halfs && pos >= coord - halfs
// }

// fn get_click_offset(square_coordinates: Vec2, mouse_coordinates: Vec2) -> Vec2 {
//     let x_offset = match mouse_coordinates.x > square_coordinates.x {
//         true => -(square_coordinates.x - mouse_coordinates.x).abs(),
//         false => (square_coordinates.x - mouse_coordinates.x).abs(),
//     };

//     let y_offset = match mouse_coordinates.y > square_coordinates.y {
//         true => -(square_coordinates.y - mouse_coordinates.y).abs(),
//         false => (square_coordinates.y - mouse_coordinates.y).abs(),
//     };

//     Vec2::new(x_offset, y_offset)
// }