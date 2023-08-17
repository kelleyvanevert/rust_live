use epaint::emath::Align2;
use epaint::text::{FontData, FontDefinitions};
use epaint::textures::TextureOptions;
use epaint::{
    hex_color, pos2, tessellate_shapes, ClippedShape, Color32, FontFamily, FontId, FontImage,
    Fonts, Primitive, Rect, Rgba, Shape, Stroke, TessellationOptions, TextureManager,
};

use crate::kgui::context::KguiContext;

pub fn draw(ctx: &KguiContext) {
    // let s = Shape::text(
    //     &self.fonts,
    //     pos2(300.0, 300.0),
    //     Align2::LEFT_TOP,
    //     "JS",
    //     FontId {
    //         size: 30.0,
    //         family: epaint::FontFamily::Monospace,
    //     },
    //     Color32::BLACK,
    // );

    let shape = Shape::Vec(vec![
        Shape::rect_filled(
            Rect {
                min: pos2(200.0, 200.0),
                max: pos2(300.0, 300.0),
            },
            10.0,
            hex_color!("#E8D44D"),
        ),
        Shape::circle_stroke(pos2(200.0, 200.0), 50.0, Stroke::new(6.0, Color32::BLACK)),
        // s,
    ]);

    ctx.graphics_mut(|graphics| {
        //
        graphics.layer(0).add(shape.visual_bounding_rect(), shape);
    });
}