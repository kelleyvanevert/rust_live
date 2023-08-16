use egui_wgpu::{winit::Painter, WgpuConfiguration};

fn main() {
    let painter = Painter::new(
        WgpuConfiguration::default(),
        1,
        depth_format,
        support_transparent_backbuffer,
    );

    println!("Hello, world!");
}
