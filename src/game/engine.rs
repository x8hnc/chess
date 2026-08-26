use std::{
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
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

    let mut threads = Vec::with_capacity(size);

    let mut moves = position.find_legal_moves();
    position.order_moves(&mut moves);

    for &movement in &moves {
        sender.send(movement).unwrap();
    }

    let tt = Arc::new(TranspositionTable::new(Chess::TT_CAPACITY));

    for _ in 0..size {
        threads.push(worker(
            Arc::clone(&receiver),
            position.clone(),
            moves[0],
            i32::MIN,
            depth,
            tt.clone(),
        ));
    }

    drop(sender);

    let mut best_move = moves[0];
    let mut best_eval = i32::MIN;
    for thread in threads {
        let (m, eval) = thread.join().unwrap();
        if eval > best_eval {
            best_eval = eval;
            best_move = m;
        }
    }

    best_move
}

fn worker(
    receiver: Arc<Mutex<mpsc::Receiver<Move>>>,
    mut position: Position,
    mut best_move: Move,
    mut best_eval: i32,
    depth: usize,
    tt: Arc<TranspositionTable>,
) -> JoinHandle<(Move, i32)> {
    let thread = thread::spawn(move || {
        loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(movement) => {
                    position.save();
                    position.make_move(movement);

                    let eval = -negamax(&mut position, depth - 1, i32::MIN + 1, i32::MAX, tt.clone());

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

    thread
}

fn negamax(
    position: &mut Position,
    depth: usize,
    mut alpha: i32,
    mut beta: i32,
    tt: Arc<TranspositionTable>,
) -> i32 {
    if position.is_checkmate() {
        let mut score = -Chess::CHECKMATE_SCORE;
        score -= depth as i32 * Chess::EXTRA_TURN_SCORE;
        return score;
    } else if position.is_draw() {
        return 0;
    }

    if depth == 0 {
        return position.evaluate();
    }

    let alpha_orig = alpha;
    let beta_orig = beta;
    let mut best = i32::MIN;
    let position_hash = position.hash();

    let mut legal_moves = position.find_legal_moves();
    position.order_moves(&mut legal_moves);

    if let Some(entry) = tt.get(position_hash) {
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

        let score = -negamax(position, depth - 1, -beta, -alpha, tt.clone());

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

    tt.insert(TTEntry::new(position_hash, depth as u8, best, bound));
    best
}
