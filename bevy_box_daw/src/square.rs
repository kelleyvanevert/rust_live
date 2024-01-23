use bevy::{
    math::vec2,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::NumDrags;

// #[derive(Bundle)]
// pub struct DialogNode {
//     pub info: DialogInfo,
//     pub mesh: MaterialMesh2dBundle<ColorMaterial>,
// }

#[derive(Component)]
pub struct DialogInfo {
    pub pos: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub z: i32,
}

impl DialogInfo {
    pub fn contains(&self, pos: Vec2) -> bool {
        Rect::from_corners(self.pos, self.pos + self.size).contains(pos)
    }
}

fn add_dialog_node(commands: &mut Commands, info: DialogInfo) {
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Px(info.size.x),
            height: Val::Px(info.size.y),
            position_type: PositionType::Absolute,
            left: Val::Px(info.pos.x),
            top: Val::Px(info.pos.y),
            ..default()
        },
        background_color: BackgroundColor(info.color),
        z_index: ZIndex::Local(info.z),
        ..default()
    });
}

pub fn add_first_boxes(mut commands: Commands, mut drags: ResMut<NumDrags>) {
    add_dialog_node(
        &mut commands,
        DialogInfo {
            pos: vec2(10.0, 10.0),
            size: vec2(200.0, 200.0),
            color: Color::BLUE,
            z: 0,
        },
    );

    add_dialog_node(
        &mut commands,
        DialogInfo {
            pos: vec2(260.0, 100.0),
            size: vec2(100.0, 100.0),
            color: Color::BLACK,
            z: 1,
        },
    );

    drags.0 = 2;
}

pub fn update_dialog_node_meshes(
    mut dragging: Query<
        (&DialogInfo, &mut Style, &mut ZIndex, &mut BackgroundColor),
        Changed<DialogInfo>,
    >,
) {
    for (info, mut style, mut z, mut bg_color) in &mut dragging {
        // *mesh_handle = meshes
        //     .add(Mesh::from(shape::Box::from_corners(
        //         Vec3::splat(0.0),
        //         info.size.extend(0.0),
        //     )))
        //     .into();

        // transform.translation = info.pos.extend(info.z as f32);

        style.width = Val::Px(info.size.x);
        style.height = Val::Px(info.size.y);

        style.left = Val::Px(info.pos.x);
        style.top = Val::Px(info.pos.y);

        bg_color.0 = info.color;

        *z = ZIndex::Local(info.z);
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
