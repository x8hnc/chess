use std::collections::HashMap;

use crate::board::{
    Board,
    movement::{Move, MoveResult},
    piece::{Color, PIECE_TYPES, Piece},
};

#[derive(Clone)]
pub struct Position {
    board: Board,
    board_history: HashMap<u64, u8>,
    undo_stack: Vec<Board>,
}

impl Position {
    pub fn new() -> Self {
        let board = Board::new();
        let mut board_history = HashMap::new();
        board_history.insert(board.hash(), 1);
        Self {
            board,
            undo_stack: Vec::new(),
            board_history,
        }
    }

    pub fn order_moves(&self, moves: &mut [Move]) {
        let mut i = 0;

        for j in 0..moves.len() {
            if self.board.is_attack(moves[j]) {
                moves.swap(i, j);
                i += 1;
            }
        }
    }

    pub fn _from_fen(fen: &str) -> Result<Self, String> {
        Ok(Self {
            board: Board::_from_fen(fen)?,
            undo_stack: vec![],
            board_history: HashMap::new(),
        })
    }

    pub fn turn(&self) -> Color {
        self.board.turn()
    }

    pub fn hash(&self) -> u64 {
        self.board.hash()
    }

    pub fn save(&mut self) {
        self.undo_stack.push(self.board.clone());
    }

    pub fn undo(&mut self) {
        self.board = self.undo_stack.pop().unwrap();
    }

    fn is_endgame(&self) -> bool {
        let white_endgame =
            !self.board.white_pieces().has_queen() || !self.board.white_pieces().has_minor_pieces();
        let black_endgame =
            !self.board.black_pieces().has_queen() || !self.board.black_pieces().has_minor_pieces();

        white_endgame && black_endgame
    }

    pub fn evaluate(&self) -> i32 {
        let (friendly_pieces, enemy_pieces) = match self.board.turn() {
            Color::White => (self.board.white_pieces(), self.board.black_pieces()),
            Color::Black => (self.board.black_pieces(), self.board.white_pieces()),
        };

        let endgame = self.is_endgame();
        let mut score = 0;

        for piece in PIECE_TYPES {
            for square in friendly_pieces.find_piece(piece) {
                score += piece.value(square, self.board.turn(), endgame);
            }

            for square in enemy_pieces.find_piece(piece) {
                score -= piece.value(square, !self.board.turn(), endgame);
            }
        }

        score
    }

    pub fn find_legal_moves(&mut self) -> Vec<Move> {
        let initial_board = self.board.clone();
        let possible_moves = self.board.find_current_possible_moves();
        let mut legal_moves = Vec::new();

        for &movement in possible_moves.iter() {
            let mut test_board = initial_board.clone();

            test_board.make_move(movement);

            if !test_board.is_in_check() {
                legal_moves.push(movement);
            }
        }

        legal_moves
    }

    fn is_stalemate(&mut self) -> bool {
        if self.board.is_in_check() {
            return false;
        }

        self.find_legal_moves().is_empty()
    }

    pub fn is_draw(&mut self) -> bool {
        *self.board_history.get(&self.board.hash()).unwrap_or(&0) >= 3 || self.is_stalemate()
    }

    pub fn is_checkmate(&mut self) -> bool {
        if !self.board.is_in_check() {
            return false;
        }

        self.find_legal_moves().is_empty()
    }

    fn is_legal(&mut self, movement: Move) -> bool {
        let piece = match self.board.turn() {
            Color::White => self.board.white_pieces().get(movement.from()),
            Color::Black => self.board.black_pieces().get(movement.from()),
        };

        let Some(piece) = piece else {
            return false;
        };

        if movement.promotion().is_none() {
            if piece == Piece::Pawn {
                let promotion_row = match self.board.turn() {
                    Color::White => Board::BLACK_PIECE_ROW,
                    Color::Black => Board::WHITE_PIECE_ROW,
                };

                if movement.to().row() == promotion_row {
                    return false;
                }
            }
        } else {
            if piece != Piece::Pawn {
                return false;
            }
        }

        let legal_moves = self.find_legal_moves();

        legal_moves.contains(&movement)
    }

    pub fn make_move(&mut self, movement: Move) -> MoveResult {
        if !self.is_legal(movement) {
            return MoveResult::Illegal;
        }

        let result = self.board.make_move(movement);
        if MoveResult::Illegal == result {
            return result;
        }

        self.board.end_turn();
        *self.board_history.entry(self.board.hash()).or_insert(0) += 1;

        MoveResult::Ok
    }

    pub fn board(&self) -> &Board {
        &self.board
    }
}
