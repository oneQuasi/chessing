
use crate::{bitboard::{BitBoard, BitInt, Edges}, game::{action::{make_chess_move, Action}, Board, Game}};

use super::{repeat, slider::SliderMoves};

#[derive(Copy, Clone)]
pub struct BishopMoves;

impl SliderMoves for BishopMoves {
    fn rays<T: BitInt>(&self, pos: BitBoard<T>, edges: &Edges<T>)  -> Vec<BitBoard<T>> {
        let up_right_ray = repeat(pos, |pos| pos.try_up(&edges, 1).try_right(&edges, 1));
        let up_left_ray = repeat(pos, |pos| pos.try_up(&edges, 1).try_left(&edges, 1));
        let down_right_ray = repeat(pos, |pos| pos.try_down(&edges, 1).try_right(&edges, 1));
        let down_left_ray = repeat(pos, |pos| pos.try_down(&edges, 1).try_left(&edges, 1));

        vec![
            up_right_ray,
            up_left_ray,
            down_right_ray,
            down_left_ray
        ]
    }
}