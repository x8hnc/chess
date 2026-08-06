use std::fmt;

use crate::board::{
    board::Board,
    piece::{Color, Piece},
    square::Square,
};

#[derive(Clone, Debug)]
pub struct PieceSet {
    pawns: u64,
    rooks: u64,
    knights: u64,
    bishops: u64,
    queens: u64,
    king: u64,
    en_passant: u64,
}

impl PieceSet {
    pub fn _empty() -> Self {
        Self {
            pawns: 0,
            rooks: 0,
            knights: 0,
            bishops: 0,
            queens: 0,
            king: 0,
            en_passant: 0,
        }
    }

    pub fn new(color: Color) -> Self {
        let (pawn_row, piece_row) = match color {
            Color::Black => (Board::BLACK_PAWN_ROW, Board::BLACK_PIECE_ROW),
            Color::White => (Board::WHITE_PAWN_ROW, Board::WHITE_PIECE_ROW),
        };

        let mut pawns = 0;
        for column in 0..=7 {
            pawns |= Square::coords_to_bitmap(pawn_row, column);
        }

        let mut rooks = 0;
        rooks |= Square::coords_to_bitmap(piece_row, 0);
        rooks |= Square::coords_to_bitmap(piece_row, 7);

        let mut knights = 0;
        knights |= Square::coords_to_bitmap(piece_row, 1);
        knights |= Square::coords_to_bitmap(piece_row, 6);

        let mut bishops = 0;
        bishops |= Square::coords_to_bitmap(piece_row, 2);
        bishops |= Square::coords_to_bitmap(piece_row, 5);

        let queens = Square::coords_to_bitmap(piece_row, 3);
        let king = Square::coords_to_bitmap(piece_row, 4);

        Self {
            pawns,
            rooks,
            knights,
            bishops,
            queens,
            king,
            en_passant: 0,
        }
    }

    pub fn is_occupied(&self, square: Square) -> bool {
        let square = square.to_bitmap();

        let pieces =
            self.pawns | self.rooks | self.knights | self.bishops | self.queens | self.king;

        pieces & square != 0
    }

    pub fn is_en_passant(&self, square: Square) -> bool {
        self.en_passant != 0 && square.to_bitmap() == self.en_passant
    }

    pub fn remove_piece(&mut self, square: Square) {
        let square = !square.to_bitmap();

        self.pawns &= square;
        self.rooks &= square;
        self.knights &= square;
        self.bishops &= square;
        self.queens &= square;
        self.king &= square;
    }

    pub fn add_piece(&mut self, square: Square, piece: Piece) {
        let square = square.to_bitmap();

        match piece {
            Piece::Pawn => self.pawns |= square,
            Piece::Rook => self.rooks |= square,
            Piece::Knight => self.knights |= square,
            Piece::Bishop => self.bishops |= square,
            Piece::Queen => self.queens |= square,
            Piece::King => self.king |= square,
        }
    }

    pub fn set_en_passant(&mut self, square: Square) {
        let square = square.to_bitmap();

        self.en_passant = square;
    }

    pub fn unset_en_passant(&mut self) {
        self.en_passant = 0;
    }

    pub fn find_piece(&self, piece: Piece) -> Vec<Square> {
        match piece {
            Piece::Pawn => Square::from_bitmap(self.pawns),
            Piece::Rook => Square::from_bitmap(self.rooks),
            Piece::Knight => Square::from_bitmap(self.knights),
            Piece::Bishop => Square::from_bitmap(self.bishops),
            Piece::Queen => Square::from_bitmap(self.queens),
            Piece::King => Square::from_bitmap(self.king),
        }
    }

    pub fn is_piece_type_on(&self, piece: Piece, square: Square) -> bool {
        let square = square.to_bitmap();

        match piece {
            Piece::Pawn => square & self.pawns != 0,
            Piece::Rook => square & self.rooks != 0,
            Piece::Knight => square & self.knights != 0,
            Piece::Bishop => square & self.bishops != 0,
            Piece::Queen => square & self.queens != 0,
            Piece::King => square & self.king != 0,
        }
    }

    pub fn has_queen(&self) -> bool {
        self.queens == 0
    }

    pub fn has_minor_pieces(&self) -> bool {
        self.rooks | self.knights | self.bishops == 0
    }

    pub fn get(&self, square: Square) -> Option<Piece> {
        let square = square.to_bitmap();

        if self.pawns & square != 0 {
            Some(Piece::Pawn)
        } else if self.rooks & square != 0 {
            Some(Piece::Rook)
        } else if self.knights & square != 0 {
            Some(Piece::Knight)
        } else if self.bishops & square != 0 {
            Some(Piece::Bishop)
        } else if self.queens & square != 0 {
            Some(Piece::Queen)
        } else if self.king & square != 0 {
            Some(Piece::King)
        } else {
            None
        }
    }

    pub fn find_king(&self) -> Square {
        let index = self.king.trailing_zeros();
        let row = index / 8;
        let col = index % 8;

        Square::new(row as i8, col as i8)
    }

    pub fn find_en_passant(&self) -> Option<Square> {
        if self.en_passant.count_ones() == 0 {
            return None;
        }

        let index = self.en_passant.trailing_zeros();
        let row = index / 8;
        let col = index % 8;

        Some(Square::new(row as i8, col as i8))
    }
}

impl fmt::Display for PieceSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Pawns:      {:064b}", self.pawns)?;
        writeln!(f, "Rooks:      {:064b}", self.rooks)?;
        writeln!(f, "Knights:    {:064b}", self.knights)?;
        writeln!(f, "Bishops:    {:064b}", self.bishops)?;
        writeln!(f, "Queens:     {:064b}", self.queens)?;
        writeln!(f, "King:       {:064b}", self.king)?;
        writeln!(f, "En Passant: {:064b}", self.en_passant)
    }
}
