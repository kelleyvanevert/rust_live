use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

#[derive(Bundle)]
pub struct Square {
    pub coords: SquareCoords,
    pub mesh: MaterialMesh2dBundle<ColorMaterial>,
}

#[derive(Component)]
pub struct SquareCoords {
    pub pos: Vec2,
    pub size: Vec2,
    pub z: usize,
}

impl SquareCoords {
    pub fn contains(&self, pos: Vec2) -> bool {
        Rect::from_corners(self.pos, self.pos + self.size).contains(pos)
    }
}

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
