
use crate::{bitboard::{BitBoard, BitInt, Edges}, game::{action::{make_chess_move, Action}, Board, Game}};

use super::{repeat, slider::SliderMoves};

#[derive(Copy, Clone)]
pub struct RookMoves;

impl SliderMoves for RookMoves {
    fn rays<T: BitInt>(&self, pos: BitBoard<T>, edges: &Edges<T>)  -> Vec<BitBoard<T>> {
        let up_ray = repeat(pos, |pos| pos.try_up(&edges, 1));
        let down_ray = repeat(pos, |pos| pos.try_down(&edges, 1));
        let left_ray = repeat(pos, |pos| pos.try_left(&edges, 1));
        let right_ray = repeat(pos, |pos| pos.try_right(&edges, 1));

        vec![
            up_ray,
            down_ray,
            left_ray,
            right_ray
        ]
    }
}