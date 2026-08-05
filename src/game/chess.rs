use crate::{
    board::{
        board::Board,
        square::{Move, MoveResult},
    },
    game::position::Position,
};

pub struct Chess {
    position: Position,
}

impl Chess {
    const BOT_DEPTH: u8 = 4;
    pub fn new() -> Self {
        Self {
            position: Position::new(),
        }
    }

    const SEARCH_THREADS: usize = 20;

    fn search(&mut self) -> Move {
        let legal_moves = self.position.find_legal_moves();

        let chunk_size = legal_moves.len().div_ceil(Self::SEARCH_THREADS);

        let mut handles = Vec::new();

        for chunk in legal_moves.chunks(chunk_size) {
            let moves = chunk.to_vec();
            let mut position = self.position.clone();

            handles.push(std::thread::spawn(move || {
                let mut best_move = moves[0];
                let mut best_eval = isize::MIN;

                for m in moves {
                    position.save();
                    position.make_move(m);

                    let eval = -Self::negamax(&mut position, Self::BOT_DEPTH);

                    position.load();

                    if eval > best_eval {
                        best_eval = eval;
                        best_move = m;
                    }
                }

                (best_move, best_eval)
            }));
        }

        let mut best_move = legal_moves[0];
        let mut best_eval = isize::MIN;

        for handle in handles {
            let (m, eval) = handle.join().unwrap();

            if eval > best_eval {
                best_eval = eval;
                best_move = m;
            }
        }

        best_move
    }

    fn negamax(position: &mut Position, depth: u8) -> isize {
        if depth == 0 {
            return position.evaluate();
        }

        let mut eval = isize::MIN;
        let legal_moves = position.find_legal_moves();

        for m in legal_moves.into_iter() {
            position.save();
            position.make_move(m);

            eval = eval.max(-Self::negamax(position, depth - 1));
            position.load();
        }

        eval
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

            self.position.load();
        }

        positions
    }

    pub fn board(&self) -> &Board {
        self.position.board()
    }
}
