use crate::{
    board::movement::{Move, MoveResult},
    game::chess::Chess,
};

pub mod terminal;
mod tui_helper;

pub struct Tui {
    chess: Chess,
    white_on_bottom: bool,
    play_white: bool,
}

impl Tui {
    pub fn new(depth: usize, threads: usize, white_on_bottom: bool, play_white: bool) -> Self {
        Self {
            chess: Chess::new(depth, threads),
            white_on_bottom,
            play_white,
        }
    }

    pub fn _from_fen(
        fen: &str,
        depth: usize,
        threads: usize,
        white_on_bottom: bool,
        play_white: bool,
    ) -> Result<Self, String> {
        Ok(Self {
            chess: Chess::_from_fen(fen, depth, threads)?,
            white_on_bottom,
            play_white,
        })
    }

    pub fn start(mut self) {
        let console = std::io::stdin();
        let mut user_input = String::new();

        tui_helper::clear_screen();
        tui_helper::print_current_turn(self.chess.board().turn());

        let mut bot_move = !self.play_white;

        loop {
            tui_helper::print_board(
                &self.chess.board().to_string(self.white_on_bottom),
                bot_move,
            );
            let move_result = if !bot_move {
                user_input.clear();
                console.read_line(&mut user_input).unwrap();
                user_input = String::from(user_input.trim());

                if user_input.eq_ignore_ascii_case("reset") {
                    self.chess.reset();
                    tui_helper::clear_message();
                    continue;
                }

                let Ok(movement) = Move::from_uci(&user_input) else {
                    tui_helper::print_message("Invalid notation.");
                    continue;
                };

                self.chess.make_move(movement)
            } else {
                let (bot_move, think_time, best_eval) = self.chess.search();
                tui_helper::print_message(
                    &format!(
                        "Bot moved: {}, in {} seconds. Evaluation: {}.",
                        bot_move,
                        think_time.as_secs_f64(),
                        best_eval
                    )[..],
                );

                self.chess.make_move(bot_move)
            };

            match move_result {
                MoveResult::Ok => {
                    if !bot_move {
                        tui_helper::clear_message();
                    }
                    bot_move = !bot_move;
                }
                MoveResult::Illegal => {
                    tui_helper::print_message("Move is not legal.");
                    continue;
                }
                MoveResult::Draw => {
                    tui_helper::print_message("Draw.");
                    break;
                }
                MoveResult::CheckMate => {
                    tui_helper::print_board(
                        &self.chess.board().to_string(self.white_on_bottom),
                        bot_move,
                    );
                    tui_helper::print_message(&format!("{} won.", self.chess.turn()));
                    break;
                }
            }

            tui_helper::print_current_turn(self.chess.board().turn());
        }
    }
}
