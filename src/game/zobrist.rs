use crate::board::{piece::{Color, Piece}, square::Square};

struct Splitmix64 {
    state: u64,
}

impl Splitmix64 {
    fn new() -> Self {
        Splitmix64 {
            state: 0,
        }
    }

    fn next_int(&mut self) -> u64 {
        let mut z = self.state.wrapping_add(0x9e3779b97f4a7c15);
        self.state = z;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}

#[derive(Clone)]
pub struct Zobrist {
    white_piece_hashes: [[u64; 64]; 6],
    black_piece_hashes: [[u64; 64]; 6],
    en_passant_hashes: [u64; 8],
    white_castle_left_hash: u64,
    white_castle_right_hash: u64,
    black_castle_left_hash: u64,
    black_castle_right_hash: u64,
    turn_hash: u64,
}

impl Zobrist {
    pub fn new() -> Self {
        let mut random = Splitmix64::new();

        let mut white_piece_hashes = [[0; 64]; 6];
        let mut black_piece_hashes = [[0; 64]; 6];

        for p in 0..6 {
            for s in 0..64 {
                white_piece_hashes[p][s] = random.next_int();
                black_piece_hashes[p][s] = random.next_int();
            }
        }

        let mut en_passant_hashes = [0; 8];

        for f in &mut en_passant_hashes {
            *f = random.next_int();
        }

        let white_castle_left_hash = random.next_int();
        let white_castle_right_hash = random.next_int();
        let black_castle_left_hash = random.next_int();
        let black_castle_right_hash = random.next_int();
        let turn_hash = random.next_int();

        Self {
            white_piece_hashes,
            black_piece_hashes,
            en_passant_hashes,
            white_castle_left_hash,
            white_castle_right_hash,
            black_castle_left_hash,
            black_castle_right_hash,
            turn_hash,
        }
    }

    pub fn get_castle_left(&self, color: Color) -> u64 {
        match color {
            Color::White => self.white_castle_left_hash,
            Color::Black => self.black_castle_left_hash,
        }
    }

    pub fn get_castle_right(&self, color: Color) -> u64 {
        match color {
            Color::White => self.white_castle_right_hash,
            Color::Black => self.black_castle_right_hash,
        }
    }

    pub fn get_piece(&self, color: Color, piece: Piece, square: Square) -> u64 {
        match color {
            Color::White => self.white_piece_hashes[piece.to_index()][square.to_index()],
            Color::Black => self.black_piece_hashes[piece.to_index()][square.to_index()],
        }
    }

    pub fn get_en_passant(&self, column: usize) -> u64 {
        self.en_passant_hashes[column]
    }

    pub fn get_turn(&self) -> u64 {
        self.turn_hash
    }
}
