
use crate::{bitboard::{BitBoard, BitInt}, game::{action::{make_chess_move, Action}, Board, Game}};

use super::{ray_attacks_backward, ray_attacks_forward, repeat};

const UP: usize = 0;
const DOWN: usize = 1;
const LEFT: usize = 2;
const RIGHT: usize = 3;
const ALL: usize = 4;

pub struct Rook;

impl Rook {
    pub fn process<T: BitInt, const N: usize>(&self, game: &mut Game<T, N>, piece_index: usize) {
        let edges = game.edges[0];
        game.lookup[piece_index] = vec![ vec![]; 5 ];

        for index in 0..64 {
            let rook = BitBoard::index(index);

            let up_ray = repeat(rook, |pos| pos.and_not(edges.top).up(1));
            let down_ray = repeat(rook, |pos| pos.and_not(edges.bottom).down(1));
            let left_ray = repeat(rook, |pos| pos.and_not(edges.left).left(1));
            let right_ray = repeat(rook, |pos| pos.and_not(edges.right).right(1));

            let all = up_ray.or(down_ray).or(left_ray).or(right_ray);

            game.lookup[piece_index][UP].push(up_ray);
            game.lookup[piece_index][DOWN].push(down_ray);
            game.lookup[piece_index][LEFT].push(left_ray);
            game.lookup[piece_index][RIGHT].push(right_ray);

            game.lookup[piece_index][ALL].push(all);
        }
    }

    pub fn capture_mask<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, piece_index: usize, mask: BitBoard<T>) -> BitBoard<T> {
        let moving_team = board.state.team_to_move();
        let mut captures = BitBoard::empty();

        for rook in board.state.pieces[piece_index].and(moving_team).iter() {
            let pos = rook as usize;

            if board.game.lookup[piece_index][ALL][pos].and(mask).is_empty() {
                continue;
            }
            
            let up = ray_attacks_forward(board, pos, piece_index, UP);
            let down = ray_attacks_backward(board, pos, piece_index, DOWN);
            let left = ray_attacks_backward(board, pos, piece_index, LEFT);
            let right = ray_attacks_forward(board, pos, piece_index, RIGHT);

            let moves = up.or(down).or(left).or(right);
            captures = captures.or(moves);
        }

        captures
    }

    pub fn list_actions<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, piece_index: usize) -> Vec<Action> {
        let moving_team = board.state.team_to_move();
        let mut actions: Vec<Action> = Vec::with_capacity(8);

        let piece = piece_index as u8;
        for rook in board.state.pieces[piece_index].and(moving_team).iter() {
            let pos = rook as usize;
            let stored_pos = rook as u16;
            
            let up = ray_attacks_forward(board, pos, piece_index, UP);
            let down = ray_attacks_backward(board, pos, piece_index, DOWN);
            let left = ray_attacks_backward(board, pos, piece_index, LEFT);
            let right = ray_attacks_forward(board, pos, piece_index, RIGHT);

            let moves = up.or(down).or(right).or(left).and_not(moving_team);
            for movement in moves.iter() {
                actions.push(Action::from(stored_pos, movement as u16, piece))
            }
        }

        actions
    }

    pub fn make_move<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, action: Action) {
        make_chess_move(&mut board.state, action)
    }
}