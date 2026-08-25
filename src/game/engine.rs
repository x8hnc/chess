use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

use crate::{
    board::movement::Move,
    game::{
        chess::Chess,
        position::Position,
        transposition_table::{Bound, TTEntry, TranspositionTable},
    },
};

pub fn search(size: usize, mut position: Position, depth: usize) -> Move {
    let (sender, receiver) = mpsc::channel();
    let receiver = Arc::new(Mutex::new(receiver));

    let mut workers = Vec::with_capacity(size);

    let mut moves = position.find_legal_moves();
    position.order_moves(&mut moves);

    for &movement in &moves {
        sender.send(movement).unwrap();
    }

    for _ in 0..size {
        workers.push(Worker::new(
            Arc::clone(&receiver),
            position.clone(),
            moves[0],
            isize::MIN,
            depth,
        ));
    }

    drop(sender);

    let mut best_move = moves[0];
    let mut best_eval = isize::MIN;
    for worker in &mut workers {
        let Some(thread) = worker.thread.take() else {
            continue;
        };

        let (m, eval) = thread.join().unwrap();
        if eval > best_eval {
            best_eval = eval;
            best_move = m;
        }
    }

    best_move
}

struct Worker {
    thread: Option<thread::JoinHandle<(Move, isize)>>,
}

impl Worker {
    fn new(
        receiver: Arc<Mutex<mpsc::Receiver<Move>>>,
        mut position: Position,
        mut best_move: Move,
        mut best_eval: isize,
        depth: usize,
    ) -> Worker {
        let mut tt = TranspositionTable::new(Chess::TT_CAPACITY);
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();
                match message {
                    Ok(movement) => {
                        position.save();
                        position.make_move(movement);

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
                            best_move = movement;
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }

            (best_move, best_eval)
        });

        Worker {
            thread: Some(thread),
        }
    }

    fn negamax(
        position: &mut Position,
        depth: usize,
        mut alpha: isize,
        mut beta: isize,
        transposition_table: &mut TranspositionTable,
    ) -> isize {
        if position.is_checkmate() {
            let mut score = -Chess::CHECKMATE_SCORE;
            score -= depth as isize * Chess::EXTRA_TURN_SCORE;
            return score;
        } else if position.is_draw() {
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
            if entry.score() == Chess::CHECKMATE_SCORE {
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

            if score == Chess::CHECKMATE_SCORE {
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
}
