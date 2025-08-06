
use crate::{bitboard::{BitBoard, BitInt, Edges}, game::{action::{make_chess_move, Action}, Board, Game}};

use super::{repeat, slider::SliderMoves};

#[derive(Copy, Clone)]
pub struct QueenMoves;

impl SliderMoves for QueenMoves {
    fn rays<T: BitInt>(&self, pos: BitBoard<T>, edges: &Edges<T>)  -> Vec<BitBoard<T>> {
        let up_ray = repeat(pos, |pos| pos.try_up(&edges, 1));
        let down_ray = repeat(pos, |pos| pos.try_down(&edges, 1));
        let left_ray = repeat(pos, |pos| pos.try_left(&edges, 1));
        let right_ray = repeat(pos, |pos| pos.try_right(&edges, 1));

        let up_right_ray = repeat(pos, |pos| pos.try_up(&edges, 1).try_right(&edges, 1));
        let up_left_ray = repeat(pos, |pos| pos.try_up(&edges, 1).try_left(&edges, 1));
        let down_right_ray = repeat(pos, |pos| pos.try_down(&edges, 1).try_right(&edges, 1));
        let down_left_ray = repeat(pos, |pos| pos.try_down(&edges, 1).try_left(&edges, 1));

        vec![
            up_ray,
            down_ray,
            left_ray,
            right_ray,
            up_right_ray,
            up_left_ray,
            down_right_ray,
            down_left_ray
        ]
    }
}