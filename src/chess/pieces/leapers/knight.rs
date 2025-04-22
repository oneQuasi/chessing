
use crate::{bitboard::{BitBoard, BitInt, Edges}, game::{action::{make_chess_move, Action}, Board, Game}};

use super::leaper::LeaperMoves;

pub struct KnightMoves;

impl LeaperMoves for KnightMoves {
    fn leaps<T: BitInt>(&self, pos: BitBoard<T>, edges: &Edges<T>) -> BitBoard<T> {
        let two_right = pos.try_right(&edges, 2);
        let two_left = pos.try_left(&edges, 2);
        let two_up = pos.try_up(&edges, 2);
        let two_down = pos.try_down(&edges, 2);
    
        let horizontal = two_right.or(two_left);
        let vertical = two_up.or(two_down);
    
        let horizontal_moves = horizontal.try_up(&edges, 1).or(horizontal.try_down(&edges, 1));
        let vertical_moves = vertical.try_right(&edges, 1).or(vertical.try_left(&edges, 1));

        let moves = horizontal_moves.or(vertical_moves);    
        moves   
    }
}