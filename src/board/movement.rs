use std::{cmp::Ordering, fmt::{self, Display}};

use crate::board::{piece::Piece, square::Square};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
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
        Some(self.cmp(other))
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
    CheckMate,
    Draw,
}

impl Display for MoveResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveResult::Ok => write!(f, "{}", "Ok"),
            MoveResult::Illegal => write!(f, "{}", "Illegal"),
            MoveResult::CheckMate => write!(f, "{}", "Checkmate"),
            MoveResult::Draw => write!(f, "{}", "Draw"),
        }
    }
}
