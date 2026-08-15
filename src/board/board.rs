use std::fmt::Display;

use crate::{
    board::{
        castle_rights::CastleRights,
        piece::{Color, PIECE_TYPES, Piece},
        piece_set::PieceSet,
        square::{Move, MoveResult, Square},
    },
    game::zobrist::Zobrist,
    ui::terminal,
};

struct MoveContext<'a> {
    friendly_pieces: &'a mut PieceSet,
    enemy_pieces: &'a mut PieceSet,
    castle_rights: &'a mut CastleRights,
    enemy_castle_rights: &'a mut CastleRights,
    pawn_row: i8,
    piece_row: i8,
    enemy_piece_row: i8,
    pawn_direction: i8,
    zobrist: &'a Zobrist,
    turn: Color,
}

pub struct Board {
    white_pieces: PieceSet,
    black_pieces: PieceSet,
    turn: Color,
    white_castle_rights: CastleRights,
    black_castle_rights: CastleRights,
    hash: u64,
    zobrist: Zobrist,
    moves: Vec<Move>,
}

impl Board {
    pub const WHITE_PAWN_ROW: i8 = 1;
    pub const BLACK_PAWN_ROW: i8 = 6;
    pub const WHITE_PIECE_ROW: i8 = 0;
    pub const BLACK_PIECE_ROW: i8 = 7;
    const MAX_MOVE: usize = 300;

    pub fn new() -> Self {
        let mut hash = 0;
        let white_pieces = PieceSet::new(Color::White);
        let black_pieces = PieceSet::new(Color::Black);
        let zobrist = Zobrist::new();

        for piece in PIECE_TYPES {
            for square in white_pieces.find_piece(piece) {
                hash ^= zobrist.get_piece(Color::White, piece, square)
            }
        }

        for piece in PIECE_TYPES {
            for square in black_pieces.find_piece(piece) {
                hash ^= zobrist.get_piece(Color::Black, piece, square)
            }
        }

        hash ^= zobrist.get_castle_left(Color::White);
        hash ^= zobrist.get_castle_right(Color::White);
        hash ^= zobrist.get_castle_left(Color::Black);
        hash ^= zobrist.get_castle_right(Color::Black);

        Self {
            white_pieces,
            black_pieces,
            turn: Color::White,
            white_castle_rights: CastleRights::new(),
            black_castle_rights: CastleRights::new(),
            zobrist,
            hash,
            moves: Vec::with_capacity(Self::MAX_MOVE),
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
            zobrist: Zobrist::new(),
            hash: 0,
            moves: Vec::with_capacity(Self::MAX_MOVE),
        };

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

        board.turn = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err("Invalid side to move".into()),
        };

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

        Ok(board)
    }

    fn move_context<'a>(&'a mut self) -> MoveContext<'a> {
        match self.turn {
            Color::White => MoveContext {
                friendly_pieces: &mut self.white_pieces,
                enemy_pieces: &mut self.black_pieces,
                castle_rights: &mut self.white_castle_rights,
                enemy_castle_rights: &mut self.black_castle_rights,
                pawn_row: Self::WHITE_PAWN_ROW,
                piece_row: Self::WHITE_PIECE_ROW,
                enemy_piece_row: Self::BLACK_PIECE_ROW,
                pawn_direction: 1,
                zobrist: &self.zobrist,
                turn: self.turn,
            },
            Color::Black => MoveContext {
                friendly_pieces: &mut self.black_pieces,
                enemy_pieces: &mut self.white_pieces,
                castle_rights: &mut self.black_castle_rights,
                enemy_castle_rights: &mut self.white_castle_rights,
                pawn_row: Self::BLACK_PAWN_ROW,
                piece_row: Self::BLACK_PIECE_ROW,
                enemy_piece_row: Self::WHITE_PIECE_ROW,
                pawn_direction: -1,
                zobrist: &self.zobrist,
                turn: self.turn,
            },
        }
    }

    fn update_castle_rights(
        ctx: &mut MoveContext,
        piece: Piece,
        movement: &Move,
        new_hash: &mut u64,
    ) {
        let (old_left, old_right) = (ctx.castle_rights.left(), ctx.castle_rights.right());

        if piece == Piece::King {
            ctx.castle_rights.king_moved();
        } else if piece == Piece::Rook {
            let left_rook_home_square =
                movement.from().column() == 0 && movement.from().row() == ctx.piece_row;
            let right_rook_home_square =
                movement.from().column() == 7 && movement.from().row() == ctx.piece_row;

            if left_rook_home_square {
                ctx.castle_rights.left_rook_moved();
            } else if right_rook_home_square {
                ctx.castle_rights.right_rook_moved();
            }
        }

        if old_left != ctx.castle_rights.left() {
            *new_hash ^= ctx.zobrist.get_castle_left(ctx.turn);
        }

        if old_right != ctx.castle_rights.right() {
            *new_hash ^= ctx.zobrist.get_castle_right(ctx.turn);
        }
    }

    fn handle_piece_move(ctx: &mut MoveContext, piece: Piece, movement: &Move, new_hash: &mut u64) {
        ctx.friendly_pieces.add_piece(movement.to(), piece);
        ctx.friendly_pieces.remove_piece(movement.from());

        *new_hash ^= ctx.zobrist.get_piece(ctx.turn, piece, movement.to());
        *new_hash ^= ctx.zobrist.get_piece(ctx.turn, piece, movement.from());
    }

    fn handle_capture(ctx: &mut MoveContext, movement: &Move, new_hash: &mut u64) {
        if let Some(captured) = ctx.enemy_pieces.get(movement.to()) {
            *new_hash ^= ctx.zobrist.get_piece(!ctx.turn, captured, movement.to());
            if captured == Piece::Rook {
                let (old_enemy_left, old_enemy_right) = (
                    ctx.enemy_castle_rights.left(),
                    ctx.enemy_castle_rights.right(),
                );

                let capture_left_rook_home_square =
                    movement.to().column() == 0 && movement.to().row() == ctx.enemy_piece_row;
                let capture_right_rook_home_square =
                    movement.to().column() == 7 && movement.to().row() == ctx.enemy_piece_row;

                if capture_left_rook_home_square {
                    ctx.enemy_castle_rights.left_rook_moved();
                } else if capture_right_rook_home_square {
                    ctx.enemy_castle_rights.right_rook_moved();
                }

                if old_enemy_left != ctx.enemy_castle_rights.left() {
                    *new_hash ^= ctx.zobrist.get_castle_left(!ctx.turn);
                }

                if old_enemy_right != ctx.enemy_castle_rights.right() {
                    *new_hash ^= ctx.zobrist.get_castle_right(!ctx.turn);
                }
            }

            ctx.enemy_pieces.remove_piece(movement.to());
        }
    }

    fn handle_castle(ctx: &mut MoveContext, movement: &Move, new_hash: &mut u64) {
        let (current_rook_square, new_rook_square) =
            if movement.to().column() > movement.from().column() {
                (
                    Square::new(ctx.piece_row, 7),
                    Square::new(ctx.piece_row, movement.to().column() - 1),
                )
            } else {
                (
                    Square::new(ctx.piece_row, 0),
                    Square::new(ctx.piece_row, movement.to().column() + 1),
                )
            };

        ctx.friendly_pieces.add_piece(new_rook_square, Piece::Rook);
        ctx.friendly_pieces.remove_piece(current_rook_square);

        *new_hash ^= ctx
            .zobrist
            .get_piece(ctx.turn, Piece::Rook, new_rook_square);
        *new_hash ^= ctx
            .zobrist
            .get_piece(ctx.turn, Piece::Rook, current_rook_square);
    }

    fn handle_en_passant(ctx: &mut MoveContext, movement: &Move, new_hash: &mut u64) {
        let capture_square = match ctx.turn {
            Color::White => Square::new(movement.to().row() - 1, movement.to().column()),
            Color::Black => Square::new(movement.to().row() + 1, movement.to().column()),
        };

        ctx.enemy_pieces.remove_piece(capture_square);
        *new_hash ^= ctx
            .zobrist
            .get_piece(!ctx.turn, Piece::Pawn, capture_square);
    }

    fn handle_pawn_jump(ctx: &mut MoveContext, movement: &Move, new_hash: &mut u64) {
        let en_passant = Square::new(ctx.pawn_row + ctx.pawn_direction, movement.from().column());

        ctx.friendly_pieces.set_en_passant(en_passant);
        *new_hash ^= ctx.zobrist.get_en_passant(en_passant.column() as usize)
    }

    fn handle_promotion(
        ctx: &mut MoveContext,
        movement: &Move,
        promoted_piece: Piece,
        new_hash: &mut u64,
    ) {
        ctx.friendly_pieces.remove_piece(movement.to());
        ctx.enemy_pieces.remove_piece(movement.to());
        ctx.friendly_pieces.add_piece(movement.to(), promoted_piece);

        *new_hash ^= ctx.zobrist.get_piece(ctx.turn, Piece::Pawn, movement.to());
        *new_hash ^= ctx
            .zobrist
            .get_piece(ctx.turn, promoted_piece, movement.to());
    }

    pub fn make_move(&mut self, movement: Move) -> MoveResult {
        let mut new_hash = self.hash;
        let mut ctx = self.move_context();
        let Some(piece) = ctx.friendly_pieces.get(movement.from()) else {
            return MoveResult::Illegal;
        };

        Self::update_castle_rights(&mut ctx, piece, &movement, &mut new_hash);
        Self::handle_piece_move(&mut ctx, piece, &movement, &mut new_hash);

        let move_was_castle =
            piece == Piece::King && movement.to().column().abs_diff(movement.from().column()) == 2;

        let move_was_jump = movement.from().row() == ctx.pawn_row
            && movement.to().row().abs_diff(ctx.pawn_row) == 2;
        if piece == Piece::Pawn && move_was_jump {
            Self::handle_pawn_jump(&mut ctx, &movement, &mut new_hash);
        } else if piece == Piece::Pawn && ctx.enemy_pieces.is_en_passant(movement.to()) {
            Self::handle_en_passant(&mut ctx, &movement, &mut new_hash);
        } else if let Some(promoted_piece) = movement.promotion() {
            Self::handle_promotion(&mut ctx, &movement, promoted_piece, &mut new_hash);
        } else if move_was_castle {
            Self::handle_castle(&mut ctx, &movement, &mut new_hash);
        } else {
            Self::handle_capture(&mut ctx, &movement, &mut new_hash);
        }

        if let Some(en_passant) = ctx.enemy_pieces.find_en_passant() {
            new_hash ^= ctx.zobrist.get_en_passant(en_passant.column() as usize);
        }

        ctx.enemy_pieces.unset_en_passant();

        self.hash = new_hash;

        MoveResult::Ok
    }

    pub fn find_current_possible_moves(&mut self) -> &Vec<Move> {
        self.moves.clear();

        for piece_type in PIECE_TYPES {
            let attacking_pieces = match self.turn {
                Color::White => &self.white_pieces,
                Color::Black => &self.black_pieces,
            };

            let pieces = attacking_pieces.find_piece(piece_type);
            for piece in pieces {
                self.find_legal_moves(piece_type, piece, self.turn);
            }
        }

        &self.moves
    }

    pub fn end_turn(&mut self) {
        self.hash ^= self.zobrist.get_turn();
        self.turn = !self.turn;
    }

    pub fn is_in_check(&self) -> bool {
        let friendly_pieces = match self.turn {
            Color::White => &self.white_pieces,
            Color::Black => &self.black_pieces,
        };

        let king_pos = friendly_pieces.find_king();

        self.is_square_attacked(king_pos)
    }

    pub fn turn(&self) -> Color {
        self.turn
    }

    fn find_legal_moves(&mut self, piece: Piece, square: Square, color: Color) {
        match piece {
            Piece::Pawn => self.find_legal_moves_pawn(square, color),
            Piece::Rook => self.find_legal_moves_rook(square, color),
            Piece::Knight => self.find_legal_moves_knight(square, color),
            Piece::Bishop => self.find_legal_moves_bishop(square, color),
            Piece::Queen => self.find_legal_moves_queen(square, color),
            Piece::King => self.find_legal_moves_king(square, color),
        };
    }

    fn find_legal_moves_pawn(&mut self, square: Square, color: Color) {
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
                self.moves.push(Move::new(push, square, None));
            } else {
                self.moves.push(Move::new(push, square, Some(Piece::Rook)));
                self.moves
                    .push(Move::new(push, square, Some(Piece::Knight)));
                self.moves
                    .push(Move::new(push, square, Some(Piece::Bishop)));
                self.moves.push(Move::new(push, square, Some(Piece::Queen)));
            }
        }

        if square.row() == pawn_row {
            let jump = Square::new(square.row() + move_direction * 2, square.column());
            if !self.black_pieces.is_occupied(jump)
                && !self.white_pieces.is_occupied(jump)
                && !self.white_pieces.is_occupied(push)
                && !self.black_pieces.is_occupied(push)
            {
                self.moves.push(Move::new(jump, square, None));
            }
        }

        let capture_left = Square::new(square.row() + move_direction, square.column() - 1);
        if capture_left.is_on_board()
            && (enemy_pieces.is_en_passant(capture_left) || enemy_pieces.is_occupied(capture_left))
        {
            if capture_left.row() != promotion_row {
                self.moves.push(Move::new(capture_left, square, None));
            } else {
                self.moves
                    .push(Move::new(capture_left, square, Some(Piece::Rook)));
                self.moves
                    .push(Move::new(capture_left, square, Some(Piece::Knight)));
                self.moves
                    .push(Move::new(capture_left, square, Some(Piece::Bishop)));
                self.moves
                    .push(Move::new(capture_left, square, Some(Piece::Queen)));
            }
        }

        let capture_right = Square::new(square.row() + move_direction, square.column() + 1);
        if capture_right.is_on_board()
            && (enemy_pieces.is_en_passant(capture_right)
                || enemy_pieces.is_occupied(capture_right))
        {
            if capture_right.row() != promotion_row {
                self.moves.push(Move::new(capture_right, square, None));
            } else {
                self.moves
                    .push(Move::new(capture_right, square, Some(Piece::Rook)));
                self.moves
                    .push(Move::new(capture_right, square, Some(Piece::Knight)));
                self.moves
                    .push(Move::new(capture_right, square, Some(Piece::Bishop)));
                self.moves
                    .push(Move::new(capture_right, square, Some(Piece::Queen)));
            }
        }
    }

    fn find_legal_moves_rook(&mut self, square: Square, color: Color) {
        let move_directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let (frienly_pieces, enemy_pieces) = match color {
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
                    self.moves.push(Move::new(new_square, square, None));
                    break;
                }

                self.moves.push(Move::new(new_square, square, None));
                new_square = Square::new(
                    new_square.row() + direction.0,
                    new_square.column() + direction.1,
                );
            }
        }
    }

    fn find_legal_moves_bishop(&mut self, square: Square, color: Color) {
        let move_directions = [(1, 1), (-1, -1), (-1, 1), (1, -1)];
        let (frienly_pieces, enemy_pieces) = match color {
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
                    self.moves.push(Move::new(new_square, square, None));
                    break;
                }

                self.moves.push(Move::new(new_square, square, None));
                new_square = Square::new(
                    new_square.row() + direction.0,
                    new_square.column() + direction.1,
                );
            }
        }
    }

    fn find_legal_moves_knight(&mut self, square: Square, color: Color) {
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

        let frienly_pieces = match color {
            Color::White => &self.white_pieces,
            Color::Black => &self.black_pieces,
        };

        for direction in move_directions {
            let new_square = Square::new(square.row() + direction.0, square.column() + direction.1);

            if !new_square.is_on_board() {
                continue;
            }

            if frienly_pieces.is_occupied(new_square) {
                continue;
            }

            self.moves.push(Move::new(new_square, square, None));
        }
    }

    fn find_legal_moves_queen(&mut self, square: Square, color: Color) {
        self.find_legal_moves_rook(square, color);
        self.find_legal_moves_bishop(square, color);
    }

    fn find_legal_moves_king(&mut self, square: Square, color: Color) {
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

        let (frienly_pieces, can_castle) = match color {
            Color::White => (&self.white_pieces, &self.white_castle_rights),
            Color::Black => (&self.black_pieces, &self.black_castle_rights),
        };

        for direction in move_directions {
            let new_square = Square::new(square.row() + direction.0, square.column() + direction.1);

            if !new_square.is_on_board() {
                continue;
            }

            if frienly_pieces.is_occupied(new_square) {
                continue;
            }

            self.moves.push(Move::new(new_square, square, None));
        }

        if can_castle.left() && self.is_castle_available(color, -1) {
            let left_two = Square::new(square.row(), square.column() - 2);
            self.moves.push(Move::new(left_two, square, None));
        }

        if can_castle.right() && self.is_castle_available(color, 1) {
            let right_two = Square::new(square.row(), square.column() + 2);
            self.moves.push(Move::new(right_two, square, None));
        }
    }

    fn is_castle_available(&self, color: Color, direction: i8) -> bool {
        let (king_square, friendly_pieces) = match color {
            Color::White => (self.white_pieces.find_king(), &self.white_pieces),
            Color::Black => (self.black_pieces.find_king(), &self.black_pieces),
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
            let over_three = Square::new(king_square.row(), king_square.column() + 3 * direction);

            if self.white_pieces.is_occupied(over_three)
                || self.black_pieces.is_occupied(over_three)
            {
                return false;
            }

            if !friendly_pieces.is_piece_type_on(Piece::Rook, Square::new(king_square.row(), 0)) {
                return false;
            }
        } else {
            if !friendly_pieces.is_piece_type_on(Piece::Rook, Square::new(king_square.row(), 7)) {
                return false;
            }
        }

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
            let new_square = Square::new(square.row() + direction.0, square.column() + direction.1);

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
            let new_square = Square::new(square.row() + direction.0, square.column() + direction.1);

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

    pub fn hash(&self) -> u64 {
        self.hash
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

impl Clone for Board {
    fn clone(&self) -> Self {
        Self {
            white_pieces: self.white_pieces.clone(),
            black_pieces: self.black_pieces.clone(),
            turn: self.turn.clone(),
            white_castle_rights: self.white_castle_rights.clone(),
            black_castle_rights: self.black_castle_rights.clone(),
            hash: self.hash.clone(),
            zobrist: self.zobrist.clone(),
            moves: Vec::with_capacity(Self::MAX_MOVE),
        }
    }
}
