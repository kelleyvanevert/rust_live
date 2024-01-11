use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

#[derive(Bundle)]
pub struct Square {
    pub coords: SquareCoords,
    pub mesh: MaterialMesh2dBundle<ColorMaterial>,
}

#[derive(Component)]
pub struct SquareCoords(pub Rect, pub usize);

// pub fn handle_moving(
//     holding: Res<Holding>,
//     mut q_square: Query<(&mut Transform, &mut SquareCoordinates, &mut Square)>,
//     q_mouse_pos: Query<&MousePosWorld>,
//     q_click: Query<&Click>,
// ) {
//     if !holding.0 {
//         return;
//     }

//     let (mut transform, mut square_coordinates, _square) = q_square.single_mut();
//     let mouse = q_mouse_pos.single();
//     let click = q_click.single();

//     transform.translation = Vec3::new(mouse.x + click.offset.x, mouse.y + click.offset.y, 0.0);
//     square_coordinates.0 = Vec3::new(mouse.x + click.offset.x, mouse.y + click.offset.y, 0.0);
// }
