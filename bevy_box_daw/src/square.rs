use bevy::{
    math::vec2,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::Drags;

#[derive(Bundle)]
pub struct DialogNode {
    pub info: DialogInfo,
    pub mesh: MaterialMesh2dBundle<ColorMaterial>,
}

#[derive(Component)]
pub struct DialogInfo {
    pub pos: Vec2,
    pub size: Vec2,
    pub z: usize,
}

impl DialogInfo {
    pub fn contains(&self, pos: Vec2) -> bool {
        Rect::from_corners(self.pos, self.pos + self.size).contains(pos)
    }
}

pub fn add_first_boxes(
    mut commands: Commands,
    mut drags: ResMut<Drags>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let pos = vec2(10.0, 10.0);
    let size = vec2(200.0, 200.0);
    let z = drags.0;
    drags.0 += 1;

    commands.spawn(DialogNode {
        info: DialogInfo {
            pos,
            size,
            z: drags.0,
        },
        mesh: MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Box::from_corners(
                    Vec3::splat(0.0),
                    size.extend(0.0),
                )))
                .into(),
            material: materials.add(ColorMaterial::from(Color::YELLOW)),
            transform: Transform::from_translation(pos.extend(z as f32)),
            ..default()
        },
    });

    let pos = vec2(260.0, 170.0);
    let size = vec2(100.0, 100.0);
    let z = drags.0;
    drags.0 += 1;

    commands.spawn(DialogNode {
        info: DialogInfo {
            pos,
            size,
            z: drags.0,
        },
        mesh: MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Box::from_corners(
                    Vec3::splat(0.0),
                    size.extend(0.0),
                )))
                .into(),
            material: materials.add(ColorMaterial::from(Color::PINK)),
            transform: Transform::from_translation(pos.extend(z as f32)),
            ..default()
        },
    });
}

pub fn update_dialog_node_meshes(
    mut meshes: ResMut<Assets<Mesh>>,
    mut dragging: Query<(&DialogInfo, &mut Transform, &mut Mesh2dHandle), Changed<DialogInfo>>,
) {
    for (info, mut transform, mut mesh_handle) in &mut dragging {
        *mesh_handle = meshes
            .add(Mesh::from(shape::Box::from_corners(
                Vec3::splat(0.0),
                info.size.extend(0.0),
            )))
            .into();

        transform.translation = info.pos.extend(info.z as f32);
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
