
use crate::{bitboard::{BitBoard, BitInt}, game::{action::{make_chess_move, Action}, Board, Game}};

use super::{ray_attacks_backward, ray_attacks_forward, repeat};

const UP_RIGHT: usize = 0;
const UP_LEFT: usize = 1;
const DOWN_RIGHT: usize = 2;
const DOWN_LEFT: usize = 3;
const ALL: usize = 4;

pub struct Bishop;

impl Bishop {
    pub fn process<T: BitInt, const N: usize>(&self, game: &mut Game<T, N>, piece_index: usize) {
        let edges = game.edges[0];
        game.lookup[piece_index] = vec![ vec![]; 5 ];

        for index in 0..64 {
            let bishop = BitBoard::index(index);

            let up_right_ray = repeat(bishop, |pos| pos.try_up(&edges, 1).try_right(&edges, 1));
            let up_left_ray = repeat(bishop, |pos| pos.try_up(&edges, 1).try_left(&edges, 1));
            let down_right_ray = repeat(bishop, |pos| pos.try_down(&edges, 1).try_right(&edges, 1));
            let down_left_ray = repeat(bishop, |pos| pos.try_down(&edges, 1).try_left(&edges, 1));

            game.lookup[piece_index][UP_RIGHT].push(up_right_ray);
            game.lookup[piece_index][UP_LEFT].push(up_left_ray);
            game.lookup[piece_index][DOWN_RIGHT].push(down_right_ray);
            game.lookup[piece_index][DOWN_LEFT].push(down_left_ray);

            let all = up_right_ray.or(up_left_ray).or(down_right_ray).or(down_left_ray);

            game.lookup[piece_index][ALL].push(all);
        }
    }

    pub fn attacks<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, piece_index: usize, mask: BitBoard<T>) -> BitBoard<T> {
        let moving_team = board.state.team_to_move();

        for bishop in board.state.pieces[piece_index].and(moving_team).iter() {
            let pos = bishop as usize;

            if board.game.lookup[piece_index][ALL][pos].and(mask).empty() {
                continue;
            }
            
            let up_right = ray_attacks_forward(board, pos, piece_index, UP_RIGHT);
            if up_right.and(mask).set() { return up_right; }
        
            let up_left = ray_attacks_forward(board, pos, piece_index, UP_LEFT);
            if up_left.and(mask).set() { return up_left; }
        
            let down_right = ray_attacks_backward(board, pos, piece_index, DOWN_RIGHT);
            if down_right.and(mask).set() { return down_right; }
        
            let down_left = ray_attacks_backward(board, pos, piece_index, DOWN_LEFT);
            if down_left.and(mask).set() { return down_left; }
        }

        BitBoard::default()
    }

    pub fn actions<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, piece_index: usize) -> Vec<Action> {
        let moving_team = board.state.team_to_move();
        let bishops = board.state.pieces[piece_index];

        let mut actions: Vec<Action> = Vec::with_capacity(13);

        let piece = piece_index as u8;
        for bishop in bishops.and(moving_team).iter() {
            let pos = bishop as usize;
            let stored_pos = bishop as u16;
            
            let up_right = ray_attacks_forward(board, pos, piece_index, UP_RIGHT);
            let up_left = ray_attacks_forward(board, pos, piece_index, UP_LEFT);
            let down_right = ray_attacks_backward(board, pos, piece_index, DOWN_RIGHT);
            let down_left = ray_attacks_backward(board, pos, piece_index, DOWN_LEFT);

            let moves = up_right.or(up_left).or(down_right).or(down_left).and_not(moving_team);
            for movement in moves.iter() {
                actions.push(Action::from(stored_pos, movement as u16, piece))
            }
        }

        actions
    }
}