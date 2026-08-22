use std::time::{Duration, Instant};

use crate::{
    board::{
        board::Board,
        movement::{Move, MoveResult},
        piece::Color,
    },
    game::{
        position::Position,
        transposition_table::{Bound, TTEntry, TranspositionTable},
    },
};

pub struct Chess {
    position: Position,
    ttables: Vec<TranspositionTable>,
    threads: usize,
    depth: usize,
}

impl Chess {
    const TT_CAPACITY: usize = 24;
    const CHECKMATE_SCORE: isize = 300000;
    const EXTRA_TURN_SCORE: isize = 100;

    pub fn new(depth: usize, threads: usize) -> Self {
        let mut ttables = Vec::with_capacity(threads);
        for _ in 0..threads {
            ttables.push(TranspositionTable::new(Self::TT_CAPACITY));
        }

        Self {
            position: Position::new(),
            ttables,
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
            ttables,
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

    pub fn search(&mut self) -> (Move, Duration, isize) {
        let now = Instant::now();
        let legal_moves = self.position.find_legal_moves();

        let chunk_size = legal_moves.len().div_ceil(self.threads);

        let mut handles = Vec::new();
        let mut tables = std::mem::take(&mut self.ttables);
        let depth = self.depth;

        for (thread_id, chunk) in legal_moves.chunks(chunk_size).enumerate() {
            let mut moves = chunk.to_vec();
            let mut position = self.position.clone();

            self.position.order_moves(&mut moves);

            let mut tt = std::mem::replace(&mut tables[thread_id], TranspositionTable::new(0));

            handles.push(std::thread::spawn(move || {
                let mut best_move = moves[0];
                let mut best_eval = isize::MIN;

                for m in moves {
                    position.save();
                    position.make_move(m);

                    let eval = -Self::negamax(
                        &mut position,
                        depth - 1,
                        isize::MIN + 1,
                        isize::MAX,
                        &mut tt,
                    );

                    position.undo();

                    if eval > best_eval {
                        best_eval = eval;
                        best_move = m;
                    }
                }

                (best_move, best_eval, tt)
            }));
        }

        let mut best_move = legal_moves[0];
        let mut best_eval = isize::MIN;

        for (i, handle) in handles.into_iter().enumerate() {
            let (m, eval, tt) = handle.join().unwrap();

            tables[i] = tt;

            if eval > best_eval {
                best_eval = eval;
                best_move = m;
            }
        }

        self.ttables = tables;

        (best_move, now.elapsed(), best_eval)
    }

    fn negamax(
        position: &mut Position,
        depth: usize,
        mut alpha: isize,
        mut beta: isize,
        transposition_table: &mut TranspositionTable,
    ) -> isize {
        if position.is_checkmate() {
            let mut score = -Self::CHECKMATE_SCORE;
            score -= depth as isize * Self::EXTRA_TURN_SCORE;
            return score;
        } else if position.is_stalemate() {
            return 0;
        }

        if depth == 0 {
            return position.evaluate();
        }

        let alpha_orig = alpha;
        let beta_orig = beta;
        let mut best = isize::MIN;
        let position_hash = position.hash();

        let mut legal_moves = position.find_legal_moves();
        position.order_moves(&mut legal_moves);

        if let Some(entry) = transposition_table.get(position_hash) {
            if entry.score() == Self::CHECKMATE_SCORE {
                return entry.score();
            }

            if entry.depth() as usize >= depth {
                match entry.bound() {
                    Bound::Exact => {
                        return entry.score();
                    }

                    Bound::Lower => {
                        alpha = alpha.max(entry.score());

                        if alpha >= beta {
                            return entry.score();
                        }
                    }

                    Bound::Upper => {
                        beta = beta.min(entry.score());

                        if alpha >= beta {
                            return entry.score();
                        }
                    }
                }

                if alpha >= beta {
                    return entry.score();
                }
            }
        }

        for m in legal_moves.into_iter() {
            position.save();
            position.make_move(m);

            let score = -Self::negamax(position, depth - 1, -beta, -alpha, transposition_table);

            position.undo();

            if score == Self::CHECKMATE_SCORE {
                return score;
            }

            if best <= score {
                best = score;
            }

            alpha = alpha.max(score);

            if alpha >= beta {
                break;
            }
        }

        let bound = if best <= alpha_orig {
            Bound::Upper
        } else if best >= beta_orig {
            Bound::Lower
        } else {
            Bound::Exact
        };

        transposition_table.insert(TTEntry::new(position_hash, depth as u8, best, bound));
        best
    }

    pub fn make_move(&mut self, movement: Move) -> MoveResult {
        let result = self.position.make_move(movement);
        if result == MoveResult::Illegal {
            return result;
        }

        if self.position.is_checkmate() {
            MoveResult::CheckMate
        } else if self.position.is_stalemate() {
            MoveResult::Draw
        } else {
            result
        }
    }

    pub fn board(&self) -> &Board {
        self.position.board()
    }
}
