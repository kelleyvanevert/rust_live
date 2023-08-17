use egui::{hex_color, text::LayoutJob, Color32, FontId, TextFormat};

pub fn code_view_ui(ui: &mut egui::Ui, mut code: &str) {
    let theme = CodeTheme::default();

    let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
        let layout_job = highlight(ui.ctx(), &theme, string);
        // layout_job.wrap.max_width = wrap_width; // no wrapping
        ui.fonts(|f| f.layout_job(layout_job))
    };

    ui.add(
        egui::TextEdit::multiline(&mut code)
            .font(egui::TextStyle::Monospace)
            .code_editor()
            .desired_rows(1)
            .lock_focus(true)
            .layouter(&mut layouter),
    );
}

/// Memoized Code highlighting
fn highlight(ctx: &egui::Context, theme: &CodeTheme, code: &str) -> LayoutJob {
    impl egui::util::cache::ComputerMut<(&CodeTheme, &str), LayoutJob> for Highlighter {
        fn compute(&mut self, (theme, code): (&CodeTheme, &str)) -> LayoutJob {
            self.highlight(theme, code)
        }
    }

    type HighlightCache = egui::util::cache::FrameCache<LayoutJob, Highlighter>;

    ctx.memory_mut(|mem| mem.caches.cache::<HighlightCache>().get((theme, code)))
}

#[derive(Clone, Copy, PartialEq, enum_map::Enum)]
enum TokenType {
    Comment,
    Keyword,
    Literal,
    NumericLiteral,
    StringLiteral,
    Punctuation,
    Whitespace,
}

#[derive(Clone, Hash, PartialEq)]
pub struct CodeTheme {
    formats: enum_map::EnumMap<TokenType, egui::TextFormat>,
}

impl Default for CodeTheme {
    fn default() -> Self {
        let regular = FontId {
            size: 18.0,
            family: egui::FontFamily::Monospace,
        };

        let bold = FontId {
            size: 18.0,
            family: egui::FontFamily::Name("Code Bold".into()),
        };

        Self {
            formats: enum_map::enum_map![
                TokenType::Comment => TextFormat::simple(regular.clone(), Color32::GRAY),
                TokenType::Keyword => TextFormat::simple(bold.clone(), hex_color!("#000000")),
                TokenType::Literal => TextFormat::simple(regular.clone(), hex_color!("000000")),
                TokenType::NumericLiteral => TextFormat::simple(bold.clone(), hex_color!("#0038FF")),
                TokenType::StringLiteral => TextFormat::simple(bold.clone(), hex_color!("#DC1A1A")),
                TokenType::Punctuation => TextFormat::simple(regular.clone(), Color32::DARK_GRAY),
                TokenType::Whitespace => TextFormat::simple(regular.clone(), Color32::TRANSPARENT),
            ],
        }
    }
}

#[derive(Default)]
struct Highlighter {}

impl Highlighter {
    fn highlight(&self, theme: &CodeTheme, mut text: &str) -> LayoutJob {
        let mut job = LayoutJob::default();

        while !text.is_empty() {
            if text.starts_with("//") {
                let end = text.find('\n').unwrap_or(text.len());
                job.append(&text[..end], 0.0, theme.formats[TokenType::Comment].clone());
                text = &text[end..];
            } else if text.starts_with('"') {
                let end = text[1..]
                    .find('"')
                    .map(|i| i + 2)
                    .or_else(|| text.find('\n'))
                    .unwrap_or(text.len());
                job.append(
                    &text[..end],
                    0.0,
                    theme.formats[TokenType::StringLiteral].clone(),
                );
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_digit() || c == '.') {
                let end = text[1..]
                    .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == 'm' || c == 's'))
                    .map_or_else(|| text.len(), |i| i + 1);
                let word = &text[..end];
                let tt = TokenType::NumericLiteral;
                job.append(word, 0.0, theme.formats[tt].clone());
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_alphanumeric()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_alphanumeric())
                    .map_or_else(|| text.len(), |i| i + 1);
                let word = &text[..end];
                let tt = if is_keyword(word) {
                    TokenType::Keyword
                } else {
                    TokenType::Literal
                };
                job.append(word, 0.0, theme.formats[tt].clone());
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_whitespace()) {
                let end = text[1..]
                    .find(|c: char| !c.is_ascii_whitespace())
                    .map_or_else(|| text.len(), |i| i + 1);
                job.append(
                    &text[..end],
                    0.0,
                    theme.formats[TokenType::Whitespace].clone(),
                );
                text = &text[end..];
            } else {
                let mut it = text.char_indices();
                it.next();
                let end = it.next().map_or(text.len(), |(idx, _chr)| idx);
                job.append(
                    &text[..end],
                    0.0,
                    theme.formats[TokenType::Punctuation].clone(),
                );
                text = &text[end..];
            }
        }

        job
    }
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    )
}
