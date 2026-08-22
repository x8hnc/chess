use std::time::{Duration, Instant};

use crate::{
    board::{
        board::Board,
        square::{Move, MoveResult},
    },
    game::{
        position::Position,
        transposition_table::{Bound, TTEntry, TranspositionTable},
    },
};

pub struct Chess {
    position: Position,
    thread_tt: Vec<TranspositionTable>,
    threads: usize,
    depth: usize,
}

impl Chess {
    const TT_CAPACITY: usize = 24;

    pub fn new(depth: usize, threads: usize) -> Self {
        let mut thread_tt = Vec::with_capacity(threads);
        for _ in 0..threads {
            thread_tt.push(TranspositionTable::new(Self::TT_CAPACITY));
        }

        Self {
            position: Position::new(),
            thread_tt,
            threads,
            depth,
        }
    }

    pub fn _from_fen(fen: &str, depth: usize, threads: usize) -> Result<Self, String> {
        let position = Position::_from_fen(fen)?;

        let mut thread_tt = Vec::with_capacity(threads);
        for _ in 0..threads {
            thread_tt.push(TranspositionTable::new(Self::TT_CAPACITY));
        }

        Ok(Self {
            position,
            thread_tt,
            threads,
            depth,
        })
    }

    pub fn search(&mut self) -> (Move, Duration) {
        let now = Instant::now();
        let legal_moves = self.position.find_legal_moves();

        let chunk_size = legal_moves.len().div_ceil(self.threads);

        let mut handles = Vec::new();
        let mut tables = std::mem::take(&mut self.thread_tt);
        let depth = self.depth;

        for (thread_id, chunk) in legal_moves.chunks(chunk_size).enumerate() {
            let moves = chunk.to_vec();
            let mut position = self.position.clone();

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

        self.thread_tt = tables;

        (best_move, now.elapsed())
    }

    fn negamax(
        position: &mut Position,
        depth: usize,
        mut alpha: isize,
        mut beta: isize,
        transposition_table: &mut TranspositionTable,
    ) -> isize {
        if position.is_checkmate() {
            return -isize::MAX;
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

        let legal_moves = position.find_legal_moves();
        if let Some(entry) = transposition_table.get(position_hash) {
            if entry.depth() as usize >= depth {
                transposition_table.stats_mut().usable += 1;
                match entry.bound() {
                    Bound::Exact => {
                        transposition_table.stats_mut().exact_cutoffs += 1;
                        return entry.score();
                    }

                    Bound::Lower => {
                        alpha = alpha.max(entry.score());

                        if alpha >= beta {
                            transposition_table.stats_mut().lower_bound_hit += 1;
                            return entry.score();
                        }
                    }

                    Bound::Upper => {
                        beta = beta.min(entry.score());

                        if alpha >= beta {
                            transposition_table.stats_mut().upper_bound_hit += 1;
                            return entry.score();
                        }
                    }
                }

                if alpha >= beta {
                    return entry.score();
                }
            } else {
                transposition_table.stats_mut().insufficient_depth += 1;
            }
        }

        for m in legal_moves.into_iter() {
            position.save();
            position.make_move(m);

            let score = -Self::negamax(position, depth - 1, -beta, -alpha, transposition_table);

            position.undo();

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
            MoveResult::CheckMate(!self.board().turn())
        } else if self.position.is_stalemate() {
            MoveResult::Draw
        } else {
            result
        }
    }

    pub fn _perft(&mut self, depth: usize, divide: bool) -> usize {
        if depth == 0 {
            return 1;
        }

        let mut positions = 0;

        let legal_moves = self.position.find_legal_moves();

        for m in legal_moves.into_iter() {
            self.position.save();
            self.position.make_move(m);

            let count = self._perft(depth - 1, false);
            if divide {
                println!("move: {} = {}", m.to_string(), count);
            }
            positions += count;

            self.position.undo();
        }

        positions
    }

    pub fn board(&self) -> &Board {
        self.position.board()
    }
}
