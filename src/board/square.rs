use std::fmt::Display;
use std::{cmp::Ordering, fmt};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
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
        self.column <= 7 && self.column >= 0 && self.row >=0 && self.row <= 7
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

    pub fn to_index(self) -> usize {
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
        Some(self.cmp(other))
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

