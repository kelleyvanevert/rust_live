// use bevy_mouse_tracking_plugin::mouse_pos::InitMouseTracking;
// use bevy_mouse_tracking_plugin::MousePos;

// use crate::mouse::Click;
// use crate::square::{Square, SquareCoordinates};
use bevy::{prelude::*, time::Stopwatch};

use crate::mouse::MousePos;

// const SQUARE_X: f32 = 0.0;
// const SQUARE_Y: f32 = 0.0;
// const SQUARE_SIZE: Vec3 = Vec3::new(200.0, 200.0, 0.0);

// #[derive(Component)]
// pub struct MainCamera;

// #[derive(Resource)]
// pub struct Counter {
//     pub count: usize,
// }

// pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
//     commands
//         .spawn((Camera2dBundle::default(), MainCamera))
//         .add(InitMouseTracking);

//     commands.spawn((
//         SpriteBundle {
//             transform: Transform {
//                 translation: Vec3::new(SQUARE_X, SQUARE_Y, 0.0),
//                 scale: SQUARE_SIZE,
//                 ..Default::default()
//             },
//             sprite: Sprite {
//                 color: Color::rgb(0.3, 0.3, 0.7),
//                 ..default()
//             },
//             ..default()
//         },
//         Square { size: SQUARE_SIZE },
//         SquareCoordinates(Vec3::new(SQUARE_X, SQUARE_Y, 0.0)),
//         Click {
//             offset: Vec2::ZERO,
//             time: Stopwatch::new(),
//         },
//     ));

//     commands.spawn(
//         TextBundle::from_sections([
//             TextSection::new(
//                 "Clicks\n",
//                 TextStyle {
//                     font: asset_server.load("fonts/FiraSans-Bold.ttf"),
//                     font_size: 32.0,
//                     color: Color::rgb(0.5, 0.5, 1.0),
//                 },
//             ),
//             TextSection::from_style(TextStyle {
//                 font: asset_server.load("fonts/FiraSans-Medium.ttf"),
//                 font_size: 32.0,
//                 color: Color::rgb(0.5, 0.5, 1.0),
//             }),
//         ])
//         .with_style(Style {
//             position_type: PositionType::Absolute,
//             left: Val::Px(5.0),
//             top: Val::Px(5.0),
//             display: Display::Flex,
//             flex_direction: FlexDirection::Column,
//             align_items: AlignItems::Center,
//             ..default()
//         }),
//     );
// }

// pub fn update_counter_text(counter: Res<Counter>, mut query: Query<&mut Text>) {
//     let mut text = query.single_mut();
//     text.sections[1].value = counter.count.to_string();
// }

// pub fn screen_to_world_system(
//     window: Query<&Window>,
//     buttons: Res<Input<MouseButton>>,
//     q_camera: Query<&Transform, With<Camera>>,
//     mouse: Res<MousePos>,
// ) {
//     let camera_transform = q_camera.single();
//     let mouse = mouse.0;

//     let position = Vec2::new(mouse.x, mouse.y);

//     if buttons.just_pressed(MouseButton::Right) {
//         println!(
//             "{}",
//             screen_to_world(position, camera_transform, window.single())
//         );
//     }
// }
