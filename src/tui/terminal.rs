use crate::board::piece::Color;

const ESC: &str = "\x1B";
pub const END_COLOR: &str = "\x1b[0m";
const WHITE_BACKGROUND: &str = "\x1B[47m";
const BLACK_BACKGROUND: &str = "\x1B[40m";
pub const BLACK_FOREGROUND: &str = "\x1B[38;5;124m";
pub const WHITE_FOREGROUND: &str = "\x1B[38;5;20m";

pub fn clear_screen() {
    print!("{}[H", ESC);
    print!("{}[2J", ESC);
}

pub fn save_cursor() {
    print!("{}[s", ESC);
}

pub fn load_cursor() {
    print!("{}[u", ESC);
}

pub fn move_cursor_down(lines: usize) {
    print!("{}[{}E", ESC, lines);
}

pub fn delete_line() {
    print!("{}[2K", ESC);
}

pub fn move_cursor_to(line: usize) {
    print!("{}[{};1H", ESC, line + 1);
}

pub fn color_square(text: &str, background: Color, foreground: Color) -> String {
    match (background, foreground) {
        (Color::White, Color::White) => {
            format!(
                "{}{}{}{}",
                WHITE_BACKGROUND, WHITE_FOREGROUND, text, END_COLOR
            )
        }
        (Color::White, Color::Black) => {
            format!(
                "{}{}{}{}",
                WHITE_BACKGROUND, BLACK_FOREGROUND, text, END_COLOR
            )
        }
        (Color::Black, Color::White) => {
            format!(
                "{}{}{}{}",
                BLACK_BACKGROUND, WHITE_FOREGROUND, text, END_COLOR
            )
        }
        (Color::Black, Color::Black) => {
            format!(
                "{}{}{}{}",
                BLACK_BACKGROUND, BLACK_FOREGROUND, text, END_COLOR
            )
        }
    }
}
