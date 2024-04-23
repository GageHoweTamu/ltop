struct App {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    cursor_position: usize,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
}

fn move_cursor_left(&mut self) {
    let cursor_moved_left = self.cursor_position.saturating_sub(1);
    self.cursor_position = self.clamp_cursor(cursor_moved_left);
}

fn move_cursor_right(&mut self) {
    let cursor_moved_right = self.cursor_position.saturating_add(1);
    self.cursor_position = self.clamp_cursor(cursor_moved_right);
}

fn enter_char(&mut self, new_char: char) {
    self.input.insert(self.cursor_position, new_char);

    self.move_cursor_right();
}

fn delete_char(&mut self) {
    let is_not_cursor_leftmost = self.cursor_position != 0;
    if is_not_cursor_leftmost {
        // Method "remove" is not used on the saved text for deleting the selected char.
        // Reason: Using remove on String works on bytes instead of the chars.
        // Using remove would require special care because of char boundaries.

        let current_index = self.cursor_position;
        let from_left_to_current_index = current_index - 1;

        // Getting all characters before the selected character.
        let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
        // Getting all characters after selected character.
        let after_char_to_delete = self.input.chars().skip(current_index);

        // Put all characters together except the selected one.
        // By leaving the selected one out, it is forgotten and therefore deleted.
        self.input = before_char_to_delete.chain(after_char_to_delete).collect();
        self.move_cursor_left();
    }
}
