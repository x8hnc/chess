use std::fmt::Display;

use crate::{
    board::{
        castle_rights::CastleRights,
        piece::{Color, PIECE_TYPES, Piece},
        piece_set::PieceSet,
        square::{Move, MoveResult, Square},
    },
    ui::terminal,
};

#[derive(Clone)]
pub struct Board {
    white_pieces: PieceSet,
    black_pieces: PieceSet,
    turn: Color,
    white_castle_rights: CastleRights,
    black_castle_rights: CastleRights,
}

impl Board {
    pub const WHITE_PAWN_ROW: i8 = 1;
    pub const BLACK_PAWN_ROW: i8 = 6;
    pub const WHITE_PIECE_ROW: i8 = 0;
    pub const BLACK_PIECE_ROW: i8 = 7;

    pub fn new() -> Self {
        Self {
            white_pieces: PieceSet::new(Color::White),
            black_pieces: PieceSet::new(Color::Black),
            turn: Color::White,
            white_castle_rights: CastleRights::new(),
            black_castle_rights: CastleRights::new(),
        }
    }

    pub fn _from_fen(fen: &str) -> Result<Self, String> {
        let parts: Vec<&str> = fen.split_whitespace().collect();

        if parts.len() != 6 {
            return Err("Invalid FEN".into());
        }

        let mut board = Self {
            white_pieces: PieceSet::_empty(),
            black_pieces: PieceSet::_empty(),
            turn: Color::White,
            white_castle_rights: CastleRights::_none(),
            black_castle_rights: CastleRights::_none(),
        };

        //
        // Piece placement
        //
        let mut row = 7;
        let mut col = 0;

        for ch in parts[0].chars() {
            match ch {
                '/' => {
                    if col != 8 {
                        return Err("Invalid FEN board".into());
                    }

                    row -= 1;
                    col = 0;
                }

                '1'..='8' => {
                    col += ch.to_digit(10).unwrap() as usize;
                }

                _ => {
                    let (piece_set, piece) = match ch {
                        'P' => (&mut board.white_pieces, Piece::Pawn),
                        'N' => (&mut board.white_pieces, Piece::Knight),
                        'B' => (&mut board.white_pieces, Piece::Bishop),
                        'R' => (&mut board.white_pieces, Piece::Rook),
                        'Q' => (&mut board.white_pieces, Piece::Queen),
                        'K' => (&mut board.white_pieces, Piece::King),

                        'p' => (&mut board.black_pieces, Piece::Pawn),
                        'n' => (&mut board.black_pieces, Piece::Knight),
                        'b' => (&mut board.black_pieces, Piece::Bishop),
                        'r' => (&mut board.black_pieces, Piece::Rook),
                        'q' => (&mut board.black_pieces, Piece::Queen),
                        'k' => (&mut board.black_pieces, Piece::King),

                        _ => return Err(format!("Invalid piece '{}'", ch)),
                    };

                    if col >= 8 {
                        return Err("Invalid FEN board".into());
                    }

                    piece_set.add_piece(Square::new(row, col as i8), piece);
                    col += 1;
                }
            }
        }

        //
        // Side to move
        //
        board.turn = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("Invalid side to move".into()),
        };

        //
        // Castling rights
        //
        let mut white = CastleRights::_none();
        let mut black = CastleRights::_none();

        if parts[2] != "-" {
            white = CastleRights::new();
            black = CastleRights::new();

            if !parts[2].contains('K') {
                white.right_rook_moved();
            }

            if !parts[2].contains('Q') {
                white.left_rook_moved();
            }

            if !parts[2].contains('k') {
                black.right_rook_moved();
            }

            if !parts[2].contains('q') {
                black.left_rook_moved();
            }
        }

        board.white_castle_rights = white;
        board.black_castle_rights = black;

        //
        // En passant
        //
        if parts[3] != "-" {
            let bytes = parts[3].as_bytes();

            if bytes.len() != 2 {
                return Err("Invalid en passant square".into());
            }

            let file = (bytes[0] - b'a') as usize;
            let rank = (bytes[1] - b'1') as usize;

            let pos = Square::new(rank as i8, file as i8);

            match board.turn {
                Color::White => board.black_pieces.set_en_passant(pos),
                Color::Black => board.white_pieces.set_en_passant(pos),
            }
        }

        // Halfmove clock (parts[4]) and fullmove number (parts[5])
        // are ignored since your Board doesn't store them.

        Ok(board)
    }

    pub fn make_move(&mut self, movement: Move) -> MoveResult {
        let (friendly_pieces, enemy_pieces, pawn_row, piece_row, pawn_direction, castle_rights) =
            match self.turn {
                Color::White => (
                    &mut self.white_pieces,
                    &mut self.black_pieces,
                    Self::WHITE_PAWN_ROW,
                    Self::WHITE_PIECE_ROW,
                    1,
                    &mut self.white_castle_rights,
                ),
                Color::Black => (
                    &mut self.black_pieces,
                    &mut self.white_pieces,
                    Self::BLACK_PAWN_ROW,
                    Self::BLACK_PIECE_ROW,
                    -1,
                    &mut self.black_castle_rights,
                ),
            };

        let Some(piece) = friendly_pieces.get(movement.from()) else {
            return MoveResult::Illegal;
        };
        
        if piece == Piece::King {
            castle_rights.king_moved();
        } else if piece == Piece::Rook {
            if movement.from().column() == 0 {
                castle_rights.left_rook_moved();
            } else if movement.from().column() == 7 {
                castle_rights.right_rook_moved();
            }
        }

        friendly_pieces.add_piece(movement.to(), piece);
        friendly_pieces.remove_piece(movement.from());
        if piece == Piece::Pawn && enemy_pieces.is_en_passant(movement.to()) {
            enemy_pieces.capture_en_passant(movement.to(), self.turn);
        } else if piece == Piece::King
            && movement.to().column().abs_diff(movement.from().column()) == 2
        {
            let (current_rook_square, new_rook_square) =
                if movement.to().column() > movement.from().column() {
                    (
                        Square::new(piece_row, 7),
                        Square::new(piece_row, movement.to().column() - 1),
                    )
                } else {
                    (
                        Square::new(piece_row, 0),
                        Square::new(piece_row, movement.to().column() + 1),
                    )
                };

            friendly_pieces.add_piece(new_rook_square, Piece::Rook);
            friendly_pieces.remove_piece(current_rook_square);
        } else {
            enemy_pieces.remove_piece(movement.to());
        }
        enemy_pieces.unset_en_passant();

        if piece == Piece::Pawn {
            if movement.from().row() == pawn_row && movement.to().row().abs_diff(pawn_row) == 2 {
                friendly_pieces.set_en_passant(Square::new(
                    pawn_row + pawn_direction,
                    movement.from().column(),
                ));
            }

            if let Some(piece) = movement.promotion() {
                self.handle_promotion(movement.to(), piece);
            }
        }

        MoveResult::Ok
    }

    pub fn find_current_possible_moves(&self) -> Vec<Move> {
        match self.turn {
            Color::White => self.find_possible_moves(&self.white_pieces),
            Color::Black => self.find_possible_moves(&self.black_pieces),
        }
    }

    fn find_possible_moves(&self, attacking_pieces: &PieceSet) -> Vec<Move> {
        let mut attacks = Vec::new();
        for piece_type in PIECE_TYPES {
            let pieces = attacking_pieces.find_piece(piece_type);
            for piece in pieces {
                let mut moves = self.find_legal_moves(piece_type, piece, self.turn);
                attacks.append(&mut moves);
            }
        }

        attacks
    }

    pub fn end_turn(&mut self) {
        self.turn = !self.turn;
    }

    pub fn is_in_check(&self) -> bool {
        let friendly_pieces = match self.turn {
            Color::White => &self.white_pieces,
            Color::Black => &self.black_pieces,
        };

        let king_pos = *friendly_pieces
            .find_piece(Piece::King)
            .first()
            .expect("King not found\n{}");

        self.is_square_attacked(king_pos)
    }

    pub fn turn(&self) -> Color {
        self.turn
    }

    pub fn find_legal_moves(&self, piece: Piece, square: Square, color: Color) -> Vec<Move> {
        match piece {
            Piece::Pawn => self.find_legal_moves_pawn(square, color),
            Piece::Rook => self.find_legal_moves_rook(square, color),
            Piece::Knight => self.find_legal_moves_knight(square, color),
            Piece::Bishop => self.find_legal_moves_bishop(square, color),
            Piece::Queen => self.find_legal_moves_queen(square, color),
            Piece::King => self.find_legal_moves_king(square, color),
        }
    }

    fn find_legal_moves_pawn(&self, square: Square, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();
        let (move_direction, enemy_pieces, promotion_row, pawn_row) = match color {
            Color::White => (
                1,
                &self.black_pieces,
                Board::BLACK_PIECE_ROW,
                Board::WHITE_PAWN_ROW,
            ),
            Color::Black => (
                -1,
                &self.white_pieces,
                Board::WHITE_PIECE_ROW,
                Board::BLACK_PAWN_ROW,
            ),
        };

        let push = Square::new(square.row() + move_direction, square.column());
        if push.is_on_board()
            && !self.white_pieces.is_occupied(push)
            && !self.black_pieces.is_occupied(push)
        {
            if push.row() != promotion_row {
                moves.push(Move::new(push, square, None));
            } else {
                moves.push(Move::new(push, square, Some(Piece::Rook)));
                moves.push(Move::new(push, square, Some(Piece::Knight)));
                moves.push(Move::new(push, square, Some(Piece::Bishop)));
                moves.push(Move::new(push, square, Some(Piece::Queen)));
            }
        }

        if square.row() == pawn_row {
            let jump = Square::new(square.row() + move_direction * 2, square.column());
            if !self.black_pieces.is_occupied(jump)
                && !self.white_pieces.is_occupied(jump)
                && !self.white_pieces.is_occupied(push)
                && !self.black_pieces.is_occupied(push)
            {
                moves.push(Move::new(jump, square, None));
            }
        }

        let capture_left = Square::new(square.row() + move_direction, square.column() - 1);
        if capture_left.is_on_board()
            && (enemy_pieces.is_en_passant(capture_left) || enemy_pieces.is_occupied(capture_left))
        {
            if capture_left.row() != promotion_row {
                moves.push(Move::new(capture_left, square, None));
            } else {
                moves.push(Move::new(capture_left, square, Some(Piece::Rook)));
                moves.push(Move::new(capture_left, square, Some(Piece::Knight)));
                moves.push(Move::new(capture_left, square, Some(Piece::Bishop)));
                moves.push(Move::new(capture_left, square, Some(Piece::Queen)));
            }
        }

        let capture_right = Square::new(square.row() + move_direction, square.column() + 1);
        if capture_right.is_on_board()
            && (enemy_pieces.is_en_passant(capture_right)
                || enemy_pieces.is_occupied(capture_right))
        {
            if capture_right.row() != promotion_row {
                moves.push(Move::new(capture_right, square, None));
            } else {
                moves.push(Move::new(capture_right, square, Some(Piece::Rook)));
                moves.push(Move::new(capture_right, square, Some(Piece::Knight)));
                moves.push(Move::new(capture_right, square, Some(Piece::Bishop)));
                moves.push(Move::new(capture_right, square, Some(Piece::Queen)));
            }
        }

        moves
    }

    fn find_legal_moves_rook(&self, square: Square, color: Color) -> Vec<Move> {
        let move_directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let mut moves = Vec::new();
        let (frienly_pieces, enemy_pieces) = match color {
            Color::White => (&self.white_pieces, &self.black_pieces),
            Color::Black => (&self.black_pieces, &self.white_pieces),
        };
        for direction in move_directions {
            let mut new_square = Square::new(
                square.row() + direction.0,
                square.column() + direction.1,
            );

            while new_square.is_on_board() {
                if frienly_pieces.is_occupied(new_square) {
                    break;
                }

                if enemy_pieces.is_occupied(new_square) {
                    moves.push(new_square);
                    break;
                }

                moves.push(new_square);
                new_square = Square::new(
                    new_square.row() + direction.0,
                    new_square.column() + direction.1,
                );
            }
        }

        moves
            .into_iter()
            .map(|new_square| Move::new(new_square, square, None))
            .collect()
    }

    fn find_legal_moves_bishop(&self, square: Square, color: Color) -> Vec<Move> {
        let move_directions = [(1, 1), (-1, -1), (-1, 1), (1, -1)];
        let mut moves = Vec::new();
        let (frienly_pieces, enemy_pieces) = match color {
            Color::White => (&self.white_pieces, &self.black_pieces),
            Color::Black => (&self.black_pieces, &self.white_pieces),
        };
        for direction in move_directions {
            let mut new_square = Square::new(
                square.row() + direction.0,
                square.column() + direction.1,
            );

            while new_square.is_on_board() {
                if frienly_pieces.is_occupied(new_square) {
                    break;
                }

                if enemy_pieces.is_occupied(new_square) {
                    moves.push(new_square);
                    break;
                }

                moves.push(new_square);
                new_square = Square::new(
                    new_square.row() + direction.0,
                    new_square.column() + direction.1,
                );
            }
        }

        moves
            .into_iter()
            .map(|new_square| Move::new(new_square, square, None))
            .collect()
    }

    fn find_legal_moves_knight(&self, square: Square, color: Color) -> Vec<Move> {
        let move_directions = [
            (-2, -1),
            (-2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (2, -1),
            (2, 1),
        ];
        let mut moves = Vec::new();
        let frienly_pieces = match color {
            Color::White => &self.white_pieces,
            Color::Black => &self.black_pieces,
        };
        for direction in move_directions {
            let new_square = Square::new(
                square.row() + direction.0,
                square.column() + direction.1,
            );

            if !new_square.is_on_board() {
                continue;
            }

            if frienly_pieces.is_occupied(new_square) {
                continue;
            }

            moves.push(new_square);
        }

        moves
            .into_iter()
            .map(|new_square| Move::new(new_square, square, None))
            .collect()
    }

    fn find_legal_moves_queen(&self, square: Square, color: Color) -> Vec<Move> {
        let mut moves = self.find_legal_moves_rook(square, color);
        moves.append(&mut self.find_legal_moves_bishop(square, color));

        moves.sort();
        moves.dedup();

        moves
    }

    fn find_legal_moves_king(&self, square: Square, color: Color) -> Vec<Move> {
        let move_directions = [
            (1, -1),
            (1, 0),
            (1, 1),
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
        ];
        let mut moves = Vec::new();
        let (frienly_pieces, can_castle) = match color {
            Color::White => (&self.white_pieces, &self.white_castle_rights),
            Color::Black => (&self.black_pieces, &self.black_castle_rights),
        };

        for direction in move_directions {
            let new_square = Square::new(
                square.row() + direction.0,
                square.column() + direction.1,
            );

            if !new_square.is_on_board() {
                continue;
            }

            if frienly_pieces.is_occupied(new_square) {
                continue;
            }

            moves.push(new_square);
        }

        if can_castle.left() && self.is_castle_available(color, -1) {
            let left_two = Square::new(square.row(), square.column() - 2);
            moves.push(left_two);
        }

        if can_castle.right() && self.is_castle_available(color, 1) {
            let right_two = Square::new(square.row(), square.column() + 2);
            moves.push(right_two);
        }
        moves
            .into_iter()
            .map(|new_square| Move::new(new_square, square, None))
            .collect()
    }

    fn is_castle_available(&self, color: Color, direction: i8) -> bool {
        let (king_square, friendly_pieces) = match color {
            Color::White => (
                *self
                    .white_pieces
                    .find_piece(Piece::King)
                    .first()
                    .expect("King not found"),
                &self.white_pieces,
            ),
            Color::Black => (
                *self
                    .black_pieces
                    .find_piece(Piece::King)
                    .first()
                    .expect("King not fount"),
                &self.black_pieces,
            ),
        };

        if self.is_square_attacked(king_square) {
            return false;
        }

        let over_one = Square::new(king_square.row(), king_square.column() + direction);

        if self.white_pieces.is_occupied(over_one) || self.black_pieces.is_occupied(over_one) {
            return false;
        }

        if self.is_square_attacked(over_one) {
            return false;
        }

        let over_two = Square::new(king_square.row(), king_square.column() + 2 * direction);

        if self.white_pieces.is_occupied(over_two) || self.black_pieces.is_occupied(over_two) {
            return false;
        }

        if self.is_square_attacked(over_two) {
            return false;
        }

        if direction == -1 {
            let over_three =
                Square::new(king_square.row(), king_square.column() + 3 * direction);

            if self.white_pieces.is_occupied(over_three)
                || self.black_pieces.is_occupied(over_three)
            {
                return false;
            }

            if !friendly_pieces.is_piece_type_on(Piece::Rook, Square::new(king_square.row(), 0))
            {
                return false;
            }
        } else {
            if !friendly_pieces.is_piece_type_on(Piece::Rook, Square::new(king_square.row(), 7))
            {
                return false;
            }
        }

        true
    }

    fn handle_promotion(&mut self, square: Square, piece: Piece) -> bool {
        match piece {
            Piece::Pawn | Piece::King => {
                return false;
            }
            _ => (),
        }

        let friendly_pieces = match self.turn {
            Color::White => &mut self.white_pieces,
            Color::Black => &mut self.black_pieces,
        };

        friendly_pieces.remove_piece(square);
        friendly_pieces.add_piece(square, piece);

        true
    }

    fn is_square_attacked_by_pawns(&self, square: Square) -> bool {
        let (pawn_direction, enemy_pieces) = match self.turn {
            Color::White => (-1, &self.black_pieces),
            Color::Black => (1, &self.white_pieces),
        };

        let left_pawn = Square::new(square.row() - pawn_direction, square.column() - 1);

        if left_pawn.is_on_board() {
            if enemy_pieces.is_piece_type_on(Piece::Pawn, left_pawn) {
                return true;
            }
        }

        let right_pawn = Square::new(square.row() - pawn_direction, square.column() + 1);

        if right_pawn.is_on_board() {
            if enemy_pieces.is_piece_type_on(Piece::Pawn, right_pawn) {
                return true;
            }
        }

        false
    }

    fn is_square_attacked_diagonally(&self, square: Square) -> bool {
        let move_directions = [(1, 1), (-1, -1), (-1, 1), (1, -1)];
        let (frienly_pieces, enemy_pieces) = match self.turn {
            Color::White => (&self.white_pieces, &self.black_pieces),
            Color::Black => (&self.black_pieces, &self.white_pieces),
        };

        for direction in move_directions {
            let mut new_square =
                Square::new(square.row() + direction.0, square.column() + direction.1);

            while new_square.is_on_board() {
                if frienly_pieces.is_occupied(new_square) {
                    break;
                }

                if enemy_pieces.is_occupied(new_square) {
                    if enemy_pieces.is_piece_type_on(Piece::Bishop, new_square)
                        || enemy_pieces.is_piece_type_on(Piece::Queen, new_square)
                    {
                        return true;
                    }
                    break;
                }

                new_square = Square::new(
                    new_square.row() + direction.0,
                    new_square.column() + direction.1,
                );
            }
        }

        false
    }

    pub fn is_square_attacked_vert_horiz(&self, square: Square) -> bool {
        let move_directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let (friendly, enemy_pieces) = match self.turn {
            Color::White => (&self.white_pieces, &self.black_pieces),
            Color::Black => (&self.black_pieces, &self.white_pieces),
        };

        for direction in move_directions {
            let mut new_square =
                Square::new(square.row() + direction.0, square.column() + direction.1);

            while new_square.is_on_board() {
                if friendly.is_occupied(new_square) {
                    break;
                }

                if enemy_pieces.is_occupied(new_square) {
                    if enemy_pieces.is_piece_type_on(Piece::Rook, new_square)
                        || enemy_pieces.is_piece_type_on(Piece::Queen, new_square)
                    {
                        return true;
                    }
                    break;
                }

                new_square = Square::new(
                    new_square.row() + direction.0,
                    new_square.column() + direction.1,
                );
            }
        }

        false
    }

    pub fn is_square_attacked_by_king(&self, square: Square) -> bool {
        let move_directions = [
            (1, -1),
            (1, 0),
            (1, 1),
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
        ];
        let enemy_pieces = match self.turn {
            Color::White => &self.black_pieces,
            Color::Black => &self.white_pieces,
        };

        for direction in move_directions {
            let new_square =
                Square::new(square.row() + direction.0, square.column() + direction.1);

            if !new_square.is_on_board() {
                continue;
            }

            if enemy_pieces.is_piece_type_on(Piece::King, new_square) {
                return true;
            }
        }

        false
    }

    pub fn is_square_attacked_by_knight(&self, square: Square) -> bool {
        let move_directions = [
            (-2, -1),
            (-2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
            (2, -1),
            (2, 1),
        ];
        let enemy_pieces = match self.turn {
            Color::White => &self.black_pieces,
            Color::Black => &self.white_pieces,
        };

        for direction in move_directions {
            let new_square =
                Square::new(square.row() + direction.0, square.column() + direction.1);

            if !new_square.is_on_board() {
                continue;
            }

            if enemy_pieces.is_piece_type_on(Piece::Knight, new_square) {
                return true;
            }
        }

        false
    }

    pub fn is_square_attacked(&self, square: Square) -> bool {
        self.is_square_attacked_by_pawns(square)
            || self.is_square_attacked_diagonally(square)
            || self.is_square_attacked_vert_horiz(square)
            || self.is_square_attacked_by_king(square)
            || self.is_square_attacked_by_knight(square)
    }

    pub fn white_pieces(&self) -> &PieceSet {
        &self.white_pieces
    }

    pub fn black_pieces(&self) -> &PieceSet {
        &self.black_pieces
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut ascii: String = String::new();
        let letters: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

        for row in (Self::WHITE_PIECE_ROW..=Self::BLACK_PIECE_ROW).rev() {
            ascii.push_str(&(row + 1).to_string());
            ascii.push(' ');

            for column in 0..8 {
                let foreground_color;
                let current_square = Square::new(row, column);
                let mut square = if let Some(p) = self.white_pieces.get(current_square) {
                    foreground_color = Color::White;
                    format!("{} ", p)
                } else if let Some(p) = self.black_pieces.get(current_square) {
                    foreground_color = Color::Black;
                    format!("{} ", p)
                } else {
                    foreground_color = Color::Black;
                    String::from("  ")
                };

                if (row % 2) + (column % 2) == 1 {
                    square = terminal::color_square(&square, Color::White, foreground_color);
                } else {
                    square = terminal::color_square(&square, Color::Black, foreground_color);
                }

                ascii.push_str(&square);
            }

            ascii.push('\n');
        }
        ascii.push_str("  ");

        for letter in letters {
            ascii.push(letter);
            ascii.push(' ');
        }
        ascii.push('\n');

        write!(f, "{}", ascii)
    }
}
