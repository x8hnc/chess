use std::time::{Duration, Instant};

use crate::{
    board::{
        Board,
        movement::{Move, MoveResult},
        piece::Color,
    },
    game::{
        engine, position::Position, transposition_table::TranspositionTable
    },
};

pub struct Chess {
    position: Position,
    threads: usize,
    depth: usize,
}

impl Chess {
    pub const TT_CAPACITY: usize = 24;
    pub const CHECKMATE_SCORE: isize = 300000;
    pub const EXTRA_TURN_SCORE: isize = 100;

    pub fn new(depth: usize, threads: usize) -> Self {
        let mut ttables = Vec::with_capacity(threads);
        for _ in 0..threads {
            ttables.push(TranspositionTable::new(1));
        }

        Self {
            position: Position::new(),
            threads,
            depth,
        }
    }

    pub fn _from_fen(fen: &str, depth: usize, threads: usize) -> Result<Self, String> {
        let mut ttables = Vec::with_capacity(threads);
        for _ in 0..threads {
            ttables.push(TranspositionTable::new(Self::TT_CAPACITY));
        }
        Ok(Self {
            position: Position::_from_fen(fen)?,
            threads,
            depth,
        })
    }

    pub fn reset(&mut self) {
        self.position = Position::new();
    }

    pub fn turn(&self) -> Color {
        self.position.turn()
    }

    pub fn search(&mut self) -> (Move, Duration) {
        let now = Instant::now();
        let best_move = engine::search(self.threads, self.position.clone(), self.depth);
        let elapsed = now.elapsed();
        (best_move, elapsed)
    }

    pub fn make_move(&mut self, movement: Move) -> MoveResult {
        let result = self.position.make_move(movement);
        if result == MoveResult::Illegal {
            return result;
        }

        if self.position.is_checkmate() {
            MoveResult::CheckMate
        } else if self.position.is_draw() {
            MoveResult::Draw
        } else {
            result
        }
    }

    pub fn board(&self) -> &Board {
        self.position.board()
    }
}
