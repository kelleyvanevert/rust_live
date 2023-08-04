use palette::{FromColor, Hsla, IntoColor, Lcha, Srgba};
// use std::time::Instant;

use crate::{render::WidgetTexture, ui::WidgetEvent, widget::Widget};

pub struct ColorSwatchWidget {
    hovering: bool,
    // t0: Instant,
    colors: Vec<[u8; 4]>,
}

impl ColorSwatchWidget {
    #[allow(unused)]
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
    fn kind(&self) -> &'static str {
        "color"
    }

    fn column_width(&self) -> usize {
        5
    }

    fn event(&mut self, event: WidgetEvent) -> bool {
        match event {
            WidgetEvent::Hover { .. } => self.hovering = true,
            WidgetEvent::Unhover => self.hovering = false,
            _ => {}
        }

        false
    }

    fn draw(&self, frame: &mut WidgetTexture) {
        // let t = Instant::elapsed(&self.t0);
        // t.as_secs();

        if self.hovering {
            frame.clear(&[0, 0, 0, 0xff]);
        } else {
            for y in 0..frame.height() {
                for x in 0..frame.width() {
                    let rgba = self.colors[((x as f32 / frame.width() as f32)
                        * (self.colors.len() as f32))
                        .floor() as usize];

                    frame.set_pixel(x, y, &rgba);
                }
            }
        }
    }
}
