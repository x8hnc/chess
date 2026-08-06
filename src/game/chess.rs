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
}

impl Chess {
    const BOT_DEPTH: u8 = 5;
    const SEARCH_THREADS: usize = 20;
    const TT_CAPACITY: usize = 24;

    pub fn new() -> Self {
        let mut thread_tt = Vec::with_capacity(Self::SEARCH_THREADS);
        for _ in 0..Self::SEARCH_THREADS {
            thread_tt.push(TranspositionTable::new(Self::TT_CAPACITY));
        }

        Self {
            position: Position::new(),
            thread_tt,
        }
    }

    pub fn search(&mut self) -> Move {
        let legal_moves = self.position.find_legal_moves();

        let chunk_size = legal_moves.len().div_ceil(Self::SEARCH_THREADS);

        let mut handles = Vec::new();
        let mut tables = std::mem::take(&mut self.thread_tt);

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
                        Self::BOT_DEPTH - 1,
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

        best_move
    }

    fn negamax(
        position: &mut Position,
        depth: u8,
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
        let mut best = isize::MIN;
        let position_hash = position.hash();

        let mut legal_moves = position.find_legal_moves();
        if transposition_table.contains(position_hash) {
            let entry = transposition_table.get(position_hash);
            if entry.depth() >= depth {
                match entry.bound() {
                    Bound::Exact => return entry.score(),

                    Bound::Lower => alpha = alpha.max(entry.score()),

                    Bound::Upper => beta = beta.min(entry.score()),
                }

                if alpha >= beta {
                    return entry.score();
                }
            } else if let Some(idx) = legal_moves.iter().position(|&m| m == entry.best()) {
                legal_moves.swap(0, idx);
            }
        }

        let mut best_move = legal_moves[0];
        for m in legal_moves.into_iter() {
            position.save();
            position.make_move(m);

            let score = -Self::negamax(position, depth - 1, -beta, -alpha, transposition_table);

            position.undo();

            if best <= score {
                best = score;
                best_move = m;
            }
            alpha = alpha.max(score);

            if alpha >= beta {
                break;
            }
        }

        let bound = if best <= alpha_orig {
            Bound::Upper
        } else if best >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };

        transposition_table.insert(TTEntry::new(position_hash, depth, best, best_move, bound));
        best
    }

    pub fn bot_move(&mut self) -> MoveResult {
        let movement = self.search();
        self.make_move(movement)
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
