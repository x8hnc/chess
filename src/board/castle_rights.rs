#[derive(Clone)]
pub struct CastleRights {
    king_moved: bool,
    right_rook_moved: bool,
    left_rook_moved: bool,
}

impl CastleRights {
    pub fn new() -> Self {
        Self {
            king_moved: false,
            right_rook_moved: false,
            left_rook_moved: false,
        }
    }

    pub fn _none() -> Self {
        Self {
            king_moved: true,
            right_rook_moved: false,
            left_rook_moved: false,
        }
    }

    pub fn king_moved(&mut self) {
        self.king_moved = true;
    }

    pub fn left_rook_moved(&mut self) {
        self.left_rook_moved = true;
    }

    pub fn right_rook_moved(&mut self) {
        self.right_rook_moved = true;
    }

    pub fn left(&self) -> bool {
        !self.king_moved && !self.left_rook_moved
    }

    pub fn right(&self) -> bool {
        !self.king_moved && !self.right_rook_moved
    }
}
