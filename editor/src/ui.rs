// in logical pixels, btw
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum WidgetEvent {
    Hover {
        bounds: (f32, f32, f32, f32),
        mouse: (f32, f32),
    },
    MouseMove {
        bounds: (f32, f32, f32, f32),
        mouse: (f32, f32),
    },
    Unhover,

    MouseDown {
        bounds: (f32, f32, f32, f32),
        mouse: (f32, f32),
        right_click: bool,
        shift: bool,
        alt: bool,
        meta_or_ctrl: bool,
    },
    Press {
        double: bool,
        bounds: (f32, f32, f32, f32),
        mouse: (f32, f32),
        right_click: bool,
        shift: bool,
        alt: bool,
        meta_or_ctrl: bool,
    },
    Release,
    MouseUp,
}

impl WidgetEvent {
    pub fn child_relative(&self, child_bounds: (f32, f32, f32, f32)) -> WidgetEvent {
        match self {
            Self::Hover { mouse, .. } => Self::Hover {
                bounds: child_bounds,
                mouse: relative_mouse(child_bounds, *mouse),
            },
            Self::MouseMove { mouse, .. } => Self::MouseMove {
                bounds: child_bounds,
                mouse: relative_mouse(child_bounds, *mouse),
            },
            Self::MouseDown {
                mouse,
                right_click,
                shift,
                alt,
                meta_or_ctrl,
                ..
            } => Self::MouseDown {
                bounds: child_bounds,
                mouse: relative_mouse(child_bounds, *mouse),
                right_click: *right_click,
                shift: *shift,
                alt: *alt,
                meta_or_ctrl: *meta_or_ctrl,
            },
            Self::Press {
                mouse,
                double,
                right_click,
                shift,
                alt,
                meta_or_ctrl,
                ..
            } => Self::Press {
                bounds: child_bounds,
                mouse: relative_mouse(child_bounds, *mouse),
                double: *double,
                right_click: *right_click,
                shift: *shift,
                alt: *alt,
                meta_or_ctrl: *meta_or_ctrl,
            },
            e => e.clone(),
        }
    }
}

fn relative_mouse(bounds: (f32, f32, f32, f32), mouse: (f32, f32)) -> (f32, f32) {
    (mouse.0 - bounds.0, mouse.1 - bounds.1)
}
