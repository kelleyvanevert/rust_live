use live_editor_state::LineData;
use tao::clipboard::Clipboard as TaoClipboard;

pub struct Clipboard {
    system_clipboard: TaoClipboard,
    copied: Option<Vec<LineData>>,
}

impl Clipboard {
    pub fn new() -> Self {
        Self {
            system_clipboard: TaoClipboard::new(),
            copied: None,
        }
    }

    pub fn read(&self) -> Option<Vec<LineData>> {
        self.copied.clone().or_else(|| {
            self.system_clipboard
                .read_text()
                .map(|str| vec![str.into()])
        })
    }

    pub fn write(&mut self, data: impl AsRef<Vec<LineData>>) {
        let data = data.as_ref().clone();

        self.system_clipboard.write_text(
            data.iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("\n\n"),
        );

        self.copied = Some(data);
    }
}
