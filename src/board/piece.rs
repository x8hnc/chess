use std::{cmp::Ordering, fmt::Display, ops::Not};

use crate::board::square::Square;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

pub const PIECE_TYPES: [Piece; 6] = [
    Piece::Pawn,
    Piece::Rook,
    Piece::Knight,
    Piece::Bishop,
    Piece::Queen,
    Piece::King,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    White,
    Black,
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Color::White => write!(f, "White"),
            Color::Black => write!(f, "Black"),
        }
    }
}

impl PartialOrd for Piece {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Piece {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Piece::Pawn => match other {
                Piece::Pawn => Ordering::Equal,
                _ => Ordering::Less,
            },
            Piece::Rook => match other {
                Piece::Rook => Ordering::Equal,
                Piece::Queen | Piece::King => Ordering::Less,
                _ => Ordering::Greater,
            },
            Piece::Knight => match other {
                Piece::Knight => Ordering::Equal,
                Piece::Pawn => Ordering::Greater,
                _ => Ordering::Less,
            },
            Piece::Bishop => match other {
                Piece::Bishop => Ordering::Equal,
                Piece::Pawn | Piece::Knight => Ordering::Greater,
                _ => Ordering::Less,
            },
            Piece::Queen => match other {
                Piece::Queen => Ordering::Equal,
                Piece::King => Ordering::Less,
                _ => Ordering::Greater,
            },
            Piece::King => match other {
                Piece::King => Ordering::Equal,
                _ => Ordering::Greater,
            },
        }
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            Piece::King => '♚',
            Piece::Queen => '♛',
            Piece::Rook => '♜',
            Piece::Knight => '♞',
            Piece::Bishop => '♝',
            Piece::Pawn => '♟',
        };

        write!(f, "{}", symbol)
    }
}

impl Piece {
    const PAWN_TABLE: [isize; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5,
        5, 10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10,
        -20, -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    const ROOK_TABLE: [isize; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0,
        0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0,
        -5, 0, 0, 0, 5, 5, 0, 0, 0,
    ];

    const KNIGHT_TABLE: [isize; 64] = [
        -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15,
        10, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15,
        15, 10, 5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
    ];

    const BISHOP_TABLE: [isize; 64] = [
        -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5,
        0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10,
        10, 10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
    ];

    const KING_TABLE: [isize; 64] = [
        -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40,
        -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40,
        -40, -30, -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20,
        30, 10, 0, 0, 10, 30, 20,
    ];

    const KING_TABLE_ENDGAME: [isize; 64] = [
        -50, -40, -30, -20, -20, -30, -40, -50, -30, -20, -10, 0, 0, -10, -20, -30, -30, -10, 20,
        30, 30, 20, -10, -30, -30, -10, 30, 40, 40, 30, -10, -30, -30, -10, 30, 40, 40, 30, -10,
        -30, -30, -10, 20, 30, 30, 20, -10, -30, -30, -30, 0, 0, 0, 0, -30, -30, -50, -30, -30,
        -30, -30, -30, -30, -50,
    ];

    const QUEEN_TABLE: [isize; 64] = [
        -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0,
        -10, -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0,
        5, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
    ];

    pub fn value(&self, square: Square, color: Color, endgame: bool) -> isize {
        let square_index = match color {
            Color::White => square.to_index(),
            Color::Black => square.to_index() ^ 56,
        };

        match self {
            Piece::Pawn => 100 + Self::PAWN_TABLE[square_index],
            Piece::Rook => 500 + Self::ROOK_TABLE[square_index],
            Piece::Knight => 320 + Self::KNIGHT_TABLE[square_index],
            Piece::Bishop => 330 + Self::BISHOP_TABLE[square_index],
            Piece::Queen => 900 + Self::QUEEN_TABLE[square_index],
            Piece::King => {
                if endgame {
                    Self::KING_TABLE_ENDGAME[square_index]
                } else {
                    Self::KING_TABLE[square_index]
                }
            }
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            Piece::Pawn => 0,
            Piece::Rook => 1,
            Piece::Knight => 2,
            Piece::Bishop => 3,
            Piece::Queen => 4,
            Piece::King => 5,
        }
    }

    pub fn to_net(self) -> char {
        match self {
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
        }
    }
}
