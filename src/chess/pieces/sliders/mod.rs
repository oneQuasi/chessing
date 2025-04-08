
use crate::{bitboard::{BitBoard, BitInt}, game::Board};

pub mod bishop;
pub mod rook;
pub mod queen;

#[inline(always)]
pub fn ray_attacks_forward<T : BitInt>(board: &mut Board<T>, pos: usize, piece_index: usize, dir: usize) -> BitBoard<T> {
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
pub fn ray_attacks_backward<T : BitInt>(board: &mut Board<T>, pos: usize, piece_index: usize, dir: usize) -> BitBoard<T> {
    let ray = board.lookup[piece_index][dir][pos];

    let blocker = ray.and(board.state.black.or(board.state.white));
    if blocker.is_set() {
        let square = blocker.bitscan_backward();
        ray.xor(board.lookup[piece_index][dir][square as usize])
    } else {
        ray
    }
}

pub fn repeat<T : BitInt>(mut pos: BitBoard<T>, apply: impl Fn(BitBoard<T>) -> BitBoard<T>) -> BitBoard<T> {
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