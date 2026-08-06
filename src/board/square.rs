use std::fmt::Display;
use std::{cmp::Ordering, fmt};

use crate::board::castle_rights::CastleRights;
use crate::board::piece::{Color, Piece};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Square {
    row: i8,
    column: i8,
}

impl Square {
    pub fn new(row: i8, column: i8) -> Self {
        Self { row, column }
    }

    pub fn to_bitmap(self) -> u64 {
        let offset = self.column + self.row * 8;

        1 << offset
    }

    pub fn coords_to_bitmap(row: i8, column: i8) -> u64 {
        let offset = column + row * 8;

        1 << offset
    }

    pub fn is_on_board(self) -> bool {
        if self.row > 7 || self.row < 0 {
            false
        } else if self.column > 7 || self.column < 0 {
            false
        } else {
            true
        }
    }

    pub fn from_bitmap(bitmap: u64) -> Vec<Self> {
        let mut coords = Vec::new();

        for i in 0..64 {
            if (bitmap >> i) & 1 != 0 {
                let x = i / 8;
                let y = i % 8;
                coords.push(Self::new(x, y));
            }
        }

        coords
    }

    pub fn row(&self) -> i8 {
        self.row
    }

    pub fn column(&self) -> i8 {
        self.column
    }

    pub fn to_index(&self) -> usize {
        (self.column + self.row * 8) as usize
    }

    pub fn from_algebraic(s: &str) -> Result<Self, String> {
        if s.len() != 2 {
            return Err("Square must have length 2".into());
        }

        let bytes = s.as_bytes();

        let file = match bytes[0] {
            b'a'..=b'h' => bytes[0] - b'a',
            _ => return Err("Invalid file".into()),
        };

        let rank = match bytes[1] {
            b'1'..=b'8' => bytes[1] - b'1',
            _ => return Err("Invalid rank".into()),
        };

        Ok(Square::new(rank as i8, file as i8))
    }
}
impl PartialOrd for Square {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.row != other.row {
            Some(self.row.cmp(&other.row))
        } else {
            Some(self.column.cmp(&other.column))
        }
    }
}

impl Ord for Square {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.row != other.row {
            self.row.cmp(&other.row)
        } else {
            self.column.cmp(&other.column)
        }
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file = (b'a' + self.column as u8) as char;
        let rank = (b'1' + self.row as u8) as char;

        write!(f, "{}{}", file, rank)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
    to: Square,
    from: Square,
    promotion: Option<Piece>,
}

impl Move {
    pub fn new(to: Square, from: Square, promotion: Option<Piece>) -> Self {
        Self {
            to,
            from,
            promotion,
        }
    }

    pub fn to(&self) -> Square {
        self.to
    }

    pub fn from(&self) -> Square {
        self.from
    }

    pub fn promotion(&self) -> Option<Piece> {
        self.promotion
    }

    pub fn from_uci(uci: &str) -> Result<Self, String> {
        if uci.len() != 4 && uci.len() != 5 {
            return Err(format!("Invalid UCI move: {}", uci));
        }

        let from = Square::from_algebraic(&uci[0..2])?;
        let to = Square::from_algebraic(&uci[2..4])?;

        let promotion = if uci.len() == 5 {
            Some(match uci.as_bytes()[4] as char {
                'q' => Piece::Queen,
                'r' => Piece::Rook,
                'b' => Piece::Bishop,
                'n' => Piece::Knight,
                c => return Err(format!("Invalid promotion piece: {}", c)),
            })
        } else {
            None
        };

        Ok(Self {
            from,
            to,
            promotion,
        })
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)?;

        if let Some(promo) = self.promotion {
            let c = match promo {
                Piece::Queen => 'q',
                Piece::Rook => 'r',
                Piece::Bishop => 'b',
                Piece::Knight => 'n',
                _ => unreachable!("invalid promotion piece"),
            };

            write!(f, "{c}")?;
        }

        Ok(())
    }
}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.to.partial_cmp(&other.to) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.from.partial_cmp(&other.from) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.promotion.partial_cmp(&other.promotion)
    }
}

impl Ord for Move {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.to != other.to {
            self.to.cmp(&other.to)
        } else if self.from != other.from {
            self.from.cmp(&other.from)
        } else if self.promotion != other.promotion {
            self.promotion.cmp(&other.promotion)
        } else {
            Ordering::Equal
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum MoveResult {
    Ok,
    Illegal,
    CheckMate(Color),
    Draw,
}

#[derive(Clone)]
pub struct Undo {
    pub movement: Move,
    pub piece: Piece,
    pub hash: u64,
    pub captured_piece: Option<(Piece, Square)>,
    pub white_castle_rights: CastleRights,
    pub black_castle_rights: CastleRights,
    pub en_passant: Option<Square>,
}

impl Undo {
    pub fn new(
        reverse_move: Move,
        piece: Piece,
        hash: u64,
        captured_piece: Option<(Piece, Square)>,
        white_castle_rights: CastleRights,
        black_castle_rights: CastleRights,
        en_passant: Option<Square>,
    ) -> Self {
        Self {
            movement: reverse_move,
            piece,
            hash,
            captured_piece,
            white_castle_rights,
            black_castle_rights,
            en_passant,
        }
    }
}
