
use crate::{bitboard::{BitBoard, BitInt}, game::Board};

pub mod bishop;
pub mod rook;
pub mod queen;
pub mod slider;
pub mod magics;

#[inline(always)]
pub fn ray_attacks<T: BitInt, const N: usize>(board: &mut Board<T, N>, pos: usize, piece_index: usize, dir: usize, ray: BitBoard<T>) -> BitBoard<T> {
    let blocker = ray.and(board.state.black.or(board.state.white));
    if blocker.set() {
        let square = if BitBoard::index(pos as u16).lt(blocker) {
            blocker.bitscan_forward()
        } else {
            blocker.bitscan_backward()
        };
        ray.xor(board.game.lookup[piece_index][dir][square as usize])
    } else {
        ray
    }
}

pub fn repeat<T: BitInt>(mut pos: BitBoard<T>, apply: impl Fn(BitBoard<T>) -> BitBoard<T>) -> BitBoard<T> {
    let mut out = BitBoard::default();
    loop {
        let progress = apply(pos);

        if !progress.set() {
            return out;
        }

        out = out.or(progress);
        pos = progress.and_not(pos);
    }
}