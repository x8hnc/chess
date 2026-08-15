use crate::{
    board::square::{Move, MoveResult},
    game::chess::Chess,
};

pub mod terminal;
mod ui_helper;

pub struct Ui {
    chess: Chess,
}

impl Ui {
    pub fn new() -> Self {
        Self {
            chess: Chess::new(),
        }
    }

    pub fn start(mut self, play_first: bool) {
        let console = std::io::stdin();
        let mut user_input = String::new();

        ui_helper::clear_screen();
        ui_helper::print_current_turn(self.chess.board().turn());

        let mut bot_move = !play_first;

        loop {
            ui_helper::print_board(&self.chess.board().to_string(), bot_move);
            let move_result = if !bot_move {
                user_input.clear();
                console.read_line(&mut user_input).unwrap();
                user_input = String::from(user_input.trim());

                if user_input.to_ascii_lowercase() == "draw" {
                    ui_helper::print_board(&self.chess.board().to_string(), bot_move);
                    ui_helper::print_message("Draw.");
                    break;
                } else if user_input.to_ascii_lowercase() == "resign" {
                    ui_helper::print_board(&self.chess.board().to_string(), bot_move);
                    ui_helper::print_message(&format!("{} won.", !self.chess.board().turn()));
                    break;
                }

                let Ok(movement) = Move::from_uci(&user_input) else {
                    ui_helper::print_message("Invalid notation.");
                    continue;
                };

                self.chess.make_move(movement)
            } else {
                let (bot_move, think_time) = self.chess.search();
                ui_helper::print_message(&format!("Bot moved: {}, in {} seconds.", bot_move.to_string(), think_time.as_secs_f64())[..]);

                self.chess.make_move(bot_move)
            };

            match move_result {
                MoveResult::Ok => {
                    if !bot_move {
                        ui_helper::clear_message();
                    }
                    bot_move = !bot_move;
                }
                MoveResult::Illegal => {
                    ui_helper::print_message("Move is not legal.");
                    continue;
                }
                MoveResult::Draw => {
                    ui_helper::print_message("Draw.");
                    break;
                }
                MoveResult::CheckMate(color) => {
                    ui_helper::print_board(&self.chess.board().to_string(), bot_move);
                    ui_helper::print_message(&format!("{} won.", color));
                    break;
                }
            }

            ui_helper::print_current_turn(self.chess.board().turn());
        }
    }
}
