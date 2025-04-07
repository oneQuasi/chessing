use crate::{bitboard::BitBoard, game::Board};

pub mod bishop;
pub mod rook;
pub mod queen;

#[inline(always)]
pub fn ray_attacks_forward(board: &mut Board, pos: usize, piece_index: usize, dir: usize) -> BitBoard {
    let ray = board.lookup[piece_index][dir][pos];

    let blocker = ray.and(board.state.black.or(board.state.white));
    if blocker.is_set() {
        let square = blocker.bitscan_forward();
        ray.xor(board.lookup[piece_index][dir][square as usize])
    } else {
        ray
    }
}

#[inline(always)]
pub fn ray_attacks_backward(board: &mut Board, pos: usize, piece_index: usize, dir: usize) -> BitBoard {
    let ray = board.lookup[piece_index][dir][pos];

    let blocker = ray.and(board.state.black.or(board.state.white));
    if blocker.is_set() {
        let square = blocker.bitscan_backward();
        ray.xor(board.lookup[piece_index][dir][square as usize])
    } else {
        ray
    }
}

pub fn repeat(mut pos: BitBoard, apply: impl Fn(BitBoard) -> BitBoard) -> BitBoard {
    let mut out = BitBoard::empty();
    loop {
        let progress = apply(pos);

        if !progress.is_set() {
            return out;
        }

        out = out.or(progress);
        pos = progress.and_not(pos);
    }
}