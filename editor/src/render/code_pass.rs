use live_editor_state::{EditorState, Pos};
use wgpu::TextureView;
use wgpu_text::{
    glyph_brush::{
        ab_glyph::FontRef, FontId, HorizontalAlign, Layout, OwnedText, Section, Text, VerticalAlign,
    },
    BrushBuilder, TextBrush,
};

use crate::highlight::{syntax_highlight, CodeToken};

use super::system::SystemData;

const CODE_COLOR: [f32; 4] = [0.02, 0.02, 0.02, 1.];
const KW_COLOR: [f32; 4] = [0.02, 0.02, 0.02, 1.];

pub struct CodePass<'a> {
    char_size: (f32, f32),
    regular_font_id: FontId,
    bold_font_id: FontId,
    code_font_size: f32,

    title_brush: TextBrush<FontRef<'a>>,
    code_brush: TextBrush<FontRef<'a>>,
}

impl<'a> CodePass<'a> {
    pub fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let roboto_slab: &[u8] = include_bytes!("../../res/fonts/RobotoSlab-Bold.ttf");

        let title_brush = BrushBuilder::using_font_bytes(roboto_slab).unwrap().build(
            &device,
            config.width,
            config.height,
            config.format,
        );

        let fira_code_bold_font =
            FontRef::try_from_slice(include_bytes!("../../res/fonts/FiraCode-Bold.ttf")).unwrap();

        let fira_code_retina_font =
            FontRef::try_from_slice(include_bytes!("../../res/fonts/FiraCode-Retina.ttf")).unwrap();

        let code_font_size = 50.0;

        let mut code_brush =
            BrushBuilder::using_fonts(vec![fira_code_retina_font.clone(), fira_code_bold_font])
                .build(&device, config.width, config.height, config.format);

        let regular_font_id = FontId(0);
        let bold_font_id = FontId(1);

        let tmp_section = Section::default().add_text(Text::new("x").with_scale(code_font_size));

        let x_bounds = code_brush.glyph_bounds(tmp_section).unwrap();

        let char_size = (x_bounds.width(), x_bounds.height());

        Self {
            char_size,
            regular_font_id,
            bold_font_id,
            code_font_size,
            title_brush,
            code_brush,
        }
    }

    pub fn char_size(&self) -> (f32, f32) {
        self.char_size
    }

    pub fn resize(&mut self, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration) {
        self.title_brush
            .resize_view(config.width as f32, config.height as f32, &queue);

        self.code_brush
            .resize_view(config.width as f32, config.height as f32, &queue);
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        system: &SystemData,
        view: &TextureView,
        editor_state: &EditorState,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Vec<(usize, (f32, f32, f32, f32))> {
        let sf = system.scale_factor;

        let mut widget_instances = vec![];

        let title_section = Section::default()
            .add_text(
                Text::new("Some title here")
                    .with_scale(100.0)
                    .with_color([0.01, 0.01, 0.01, 1.0]),
            )
            .with_layout(
                Layout::default()
                    .v_align(VerticalAlign::Top)
                    .h_align(HorizontalAlign::Left),
            )
            // .with_bounds((config.width as f32 - 200.0, config.height as f32))
            .with_screen_position((100.0, 100.0))
            .to_owned();

        let mut code_section = Section::default()
            .with_layout(
                Layout::default()
                    .v_align(VerticalAlign::Top)
                    .h_align(HorizontalAlign::Left),
            )
            .with_screen_position((100.0, 260.0))
            .to_owned();

        let mk_widget_space = |width: usize| {
            OwnedText::new((0..width).map(|_| ' ').collect::<String>())
                .with_font_id(self.bold_font_id)
                .with_scale(self.code_font_size)
                .with_color(KW_COLOR)
        };

        let mk_keyword = |text: String| {
            OwnedText::new(text)
                .with_font_id(self.bold_font_id)
                .with_scale(self.code_font_size)
                .with_color(KW_COLOR)
        };

        let mk_regular = |text: String| {
            OwnedText::new(text)
                .with_font_id(self.regular_font_id)
                .with_scale(self.code_font_size)
                .with_color(CODE_COLOR)
        };

        for (row, line) in syntax_highlight(editor_state.linedata()) {
            for token in line {
                match token {
                    CodeToken::Keyword { text, .. } => code_section.text.push(mk_keyword(text)),
                    CodeToken::Text { text, .. } => code_section.text.push(mk_regular(text)),
                    CodeToken::Widget { col, width, id } => {
                        code_section.text.push(mk_widget_space(width));

                        let (x_start, y) = system.pos_to_px(Pos {
                            row: row as i32,
                            col: col as i32,
                        });

                        let (x_end, _) = system.pos_to_px(Pos {
                            row: row as i32,
                            col: (col + width) as i32,
                        });

                        widget_instances.push((
                            id,
                            (
                                x_start,
                                y + 4.0 / sf,
                                x_end,
                                y + system.char_size.1 / sf - 4.0 / sf,
                            ),
                        ));
                    }
                }
            }

            code_section.text.push(mk_regular("\n".into()));
        }

        self.title_brush
            .queue(&device, &queue, vec![&title_section])
            .unwrap();

        self.code_brush
            .queue(&device, &queue, vec![&code_section])
            .unwrap();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Code render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,

                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        self.title_brush.draw(&mut render_pass);
        self.code_brush.draw(&mut render_pass);

        widget_instances
    }
}
