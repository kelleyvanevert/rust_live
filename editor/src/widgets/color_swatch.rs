use palette::{FromColor, Hsla, IntoColor, Lcha, Srgba};
// use std::time::Instant;

use crate::widget::{Widget, WidgetEvent};

pub struct ColorSwatchWidget {
    hovering: bool,
    // t0: Instant,
    colors: Vec<[u8; 4]>,
}

impl ColorSwatchWidget {
    pub fn new() -> Self {
        let my_rgb = Srgba::new(0.8, 0.3, 0.3, 1.0);

        let mut my_lch = Lcha::from_color(my_rgb);
        my_lch.hue += 180.0;

        let mut my_hsl: Hsla = my_lch.into_color();
        my_hsl.lightness *= 0.6;

        let my_rgb: [u8; 4] = my_rgb.into_format().into();
        let my_lch: [u8; 4] = Srgba::from_color(my_lch).into_format().into();
        let my_hsl: [u8; 4] = Srgba::from_color(my_hsl).into_format().into();

        let orangeish = Srgba::new(1.0, 0.6, 0.0, 1.0);
        let blueish = Srgba::new(0.0, 0.2, 1.0, 1.0);

        let orangish: [u8; 4] = orangeish.into_format().into();
        let blueish: [u8; 4] = blueish.into_format().into();

        Self {
            hovering: false,
            // t0: Instant::now(),
            colors: vec![my_rgb, my_lch, my_hsl, orangish, blueish],
        }
    }
}

impl Widget for ColorSwatchWidget {
    fn column_width(&self) -> usize {
        5
    }

    fn event(&mut self, event: WidgetEvent) {
        match event {
            WidgetEvent::Hover => self.hovering = true,
            WidgetEvent::Unhover => self.hovering = false,
        }
    }

    fn draw(&self, frame: &mut [u8], width: usize, height: usize) {
        // let t = Instant::elapsed(&self.t0);
        // t.as_secs();

        if self.hovering {
            for pixel in frame.chunks_exact_mut(4) {
                pixel[0] = 0; // R
                pixel[1] = 0; // G
                pixel[2] = 0; // B
                pixel[3] = 0xff; // A
            }
        } else {
            for y in 0..height {
                for x in 0..width {
                    let [r, g, b, a] = self.colors
                        [((x as f32 / width as f32) * (self.colors.len() as f32)).floor() as usize];

                    frame[(y * width + x) * 4 + 0] = r; // R
                    frame[(y * width + x) * 4 + 1] = g; // G
                    frame[(y * width + x) * 4 + 2] = b; // B
                    frame[(y * width + x) * 4 + 3] = a; // A
                }
            }
        }
    }
}
