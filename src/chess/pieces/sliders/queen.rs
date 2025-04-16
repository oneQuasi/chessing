
use crate::{bitboard::{BitBoard, BitInt}, game::{action::{make_chess_move, Action}, piece::{Piece, PieceRules}, Board, Game}};

use super::{ray_attacks_backward, ray_attacks_forward, repeat};

const UP: usize = 0;
const DOWN: usize = 1;
const LEFT: usize = 2;
const RIGHT: usize = 3;
const UP_RIGHT: usize = 4;
const UP_LEFT: usize = 5;
const DOWN_RIGHT: usize = 6;
const DOWN_LEFT: usize = 7;
const SIDES: usize = 8;
const DIAGONALS: usize = 9;
const ALL: usize = 10;

pub struct QueenRules;

impl<T: BitInt, const N: usize> PieceRules<T, N> for QueenRules {
    fn process(&self, game: &mut Game<T, N>, piece_index: usize) {
        let edges = game.edges[0];
        game.lookup[piece_index] = vec![ vec![]; 11 ];

        for index in 0..64 {
            let queen = BitBoard::index(index);
            
            let up_ray = repeat(queen, |pos| pos.and_not(edges.top).up(1));
            let down_ray = repeat(queen, |pos| pos.and_not(edges.bottom).down(1));
            let left_ray = repeat(queen, |pos| pos.and_not(edges.left).left(1));
            let right_ray = repeat(queen, |pos| pos.and_not(edges.right).right(1));

            let up_right_ray = repeat(queen, |pos| pos.and_not(edges.top).and_not(edges.right).up(1).right(1));
            let up_left_ray = repeat(queen, |pos| pos.and_not(edges.top).and_not(edges.left).up(1).left(1));
            let down_right_ray = repeat(queen, |pos| pos.and_not(edges.bottom).and_not(edges.right).down(1).right(1));
            let down_left_ray = repeat(queen, |pos| pos.and_not(edges.bottom).and_not(edges.left).down(1).left(1));
            
            game.lookup[piece_index][UP].push(up_ray);
            game.lookup[piece_index][DOWN].push(down_ray);
            game.lookup[piece_index][LEFT].push(left_ray);
            game.lookup[piece_index][RIGHT].push(right_ray);

            game.lookup[piece_index][UP_RIGHT].push(up_right_ray);
            game.lookup[piece_index][UP_LEFT].push(up_left_ray);
            game.lookup[piece_index][DOWN_RIGHT].push(down_right_ray);
            game.lookup[piece_index][DOWN_LEFT].push(down_left_ray);

            let sides = up_ray.or(down_ray).or(left_ray).or(right_ray);
            let diagonals = up_right_ray.or(up_left_ray).or(down_right_ray).or(down_left_ray);

            let all = sides.or(diagonals);
            
            game.lookup[piece_index][SIDES].push(sides);
            game.lookup[piece_index][DIAGONALS].push(diagonals);
            game.lookup[piece_index][ALL].push(all);
        }
    }

    fn capture_mask(&self, board: &mut Board<T, N>, piece_index: usize, mask: BitBoard<T>) -> BitBoard<T> {
        let moving_team = board.state.team_to_move();
        let mut captures = BitBoard::empty();

        for queen in board.state.pieces[piece_index].and(moving_team).iter() {
            let pos = queen as usize;
            
            if board.game.lookup[piece_index][ALL][pos].and(mask).is_empty() {
                continue;
            }

            let mut moves = BitBoard::empty();
            
            if board.game.lookup[piece_index][SIDES][pos].and(mask).is_set() {
                let up = ray_attacks_forward(board, pos, piece_index, UP);
                let down = ray_attacks_backward(board, pos, piece_index, DOWN);
                let left = ray_attacks_backward(board, pos, piece_index, LEFT);
                let right = ray_attacks_forward(board, pos, piece_index, RIGHT);

                moves = moves.or(up).or(down).or(left).or(right);
            }

            if board.game.lookup[piece_index][DIAGONALS][pos].and(mask).is_set() {
                let up_right = ray_attacks_forward(board, pos, piece_index, UP_RIGHT);
                let up_left = ray_attacks_forward(board, pos, piece_index, UP_LEFT);
                let down_right = ray_attacks_backward(board, pos, piece_index, DOWN_RIGHT);
                let down_left = ray_attacks_backward(board, pos, piece_index, DOWN_LEFT);

                moves = moves.or(up_right).or(up_left).or(down_right).or(down_left);
            }

            captures = captures.or(moves);
        }

        captures
    }

    fn list_actions(&self, board: &mut Board<T, N>, piece_index: usize) -> Vec<Action> {
        let moving_team = board.state.team_to_move();
        let mut actions: Vec<Action> = Vec::with_capacity(4);

        let piece = piece_index as u8;
        for queen in board.state.pieces[piece_index].and(moving_team).iter() {
            let pos = queen as usize;
            let stored_pos = queen as u8;
            
            let up = ray_attacks_forward(board, pos, piece_index, UP);
            let down = ray_attacks_backward(board, pos, piece_index, DOWN);
            let left = ray_attacks_backward(board, pos, piece_index, LEFT);
            let right = ray_attacks_forward(board, pos, piece_index, RIGHT);

            let up_right = ray_attacks_forward(board, pos, piece_index, UP_RIGHT);
            let up_left = ray_attacks_forward(board, pos, piece_index, UP_LEFT);
            let down_right = ray_attacks_backward(board, pos, piece_index, DOWN_RIGHT);
            let down_left = ray_attacks_backward(board, pos, piece_index, DOWN_LEFT);

            let moves = up.or(down).or(left).or(right).or(up_right).or(up_left).or(down_right).or(down_left).and_not(moving_team);
            for movement in moves.iter() {
                actions.push(Action::from(stored_pos, movement as u8, piece))
            }
        }

        actions
    }

    fn make_move(&self, board: &mut Board<T, N>, action: Action) {
        make_chess_move(&mut board.state, action)
    }
}

pub fn create_queen<T: BitInt, const N: usize>() -> Piece<T, N> {
    Piece::new("q", "queen", Box::new(QueenRules))
}
