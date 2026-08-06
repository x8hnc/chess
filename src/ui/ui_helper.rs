use crate::{
    board::piece::Color, ui::terminal::{self, BLACK_FOREGROUND, END_COLOR, WHITE_FOREGROUND}
};

const CURRENT_TURN_LINE: usize = 0;
const BOARD_START_LINE: usize = 1;
const MESSAGE_LINE: usize = BOARD_START_LINE + 12;
const USER_PROMPT: &str = "Your move: ";
const BOT_PROMPT: &str = "Bot move, please wait.";

pub fn clear_screen() {
    terminal::clear_screen()
}

pub fn print_board(board: &str, bot_move: bool) {
    let board_string = board.to_string();

    terminal::move_cursor_to(BOARD_START_LINE);

    for _ in BOARD_START_LINE..MESSAGE_LINE {
        terminal::delete_line();
        terminal::move_cursor_down(1);
    }
    terminal::move_cursor_to(BOARD_START_LINE);

    let prompt = if bot_move {
        BOT_PROMPT
    } else {
        USER_PROMPT
    };

    println!("{}\n{}", board_string, prompt);
}

pub fn print_message(message: &str) {
    terminal::move_cursor_to(MESSAGE_LINE);
    terminal::delete_line();
    println!("{}", message);
}

pub fn clear_message() {
    terminal::save_cursor();
    terminal::move_cursor_to(MESSAGE_LINE);
    terminal::delete_line();
    terminal::load_cursor();
}

pub fn print_current_turn(turn: Color) {
    terminal::move_cursor_to(CURRENT_TURN_LINE);
    terminal::delete_line();
    match turn {
        Color::White => println!("{}{}{}", WHITE_FOREGROUND, turn, END_COLOR),
        Color::Black => println!("{}{}{}", BLACK_FOREGROUND, turn, END_COLOR),
    }
}
