pub trait Widget {
    // Decide how big it should be in the code editor (only called once)
    fn column_width(&self) -> usize {
        5
    }

    // Receive events such as: suspend, update how many instances are used, mouse input stuff, etc.
    fn event(&self) {}

    // Draw to pixel frame
    fn draw(&self, _frame: &mut [u8], _width: usize, _height: usize) {
        //
    }

    // When the file is saved in "bundled" mode, this method is called
    fn bundle_resources(&self) {}

    // For debugging, or for "save as text file"
    fn describe(&self) -> String {
        format!("[no description]")
    }
}
