use crate::{bitboard::BitBoard, game::{action::{make_chess_move, Action, HistoryState}, piece::{Piece, PieceProcessor}, Board, Team}};

use super::{ray_attacks_backward, ray_attacks_forward, repeat};

const UP: usize = 0;
const DOWN: usize = 1;
const LEFT: usize = 2;
const RIGHT: usize = 3;
const ALL: usize = 4;

pub struct RookProcess;

impl PieceProcessor for RookProcess {
    fn process(&self, board: &mut Board, piece_index: usize) {
        let edges = board.edges[0];
        board.lookup[piece_index] = vec![ vec![]; 5 ];

        for index in 0..64 {
            let rook = BitBoard::index(index);

            let up_ray = repeat(rook, |pos| pos.and_not(edges.top).up(1));
            let down_ray = repeat(rook, |pos| pos.and_not(edges.bottom).down(1));
            let left_ray = repeat(rook, |pos| pos.and_not(edges.left).left(1));
            let right_ray = repeat(rook, |pos| pos.and_not(edges.right).right(1));

            let all = up_ray.or(down_ray).or(left_ray).or(right_ray);

            board.lookup[piece_index][UP].push(up_ray);
            board.lookup[piece_index][DOWN].push(down_ray);
            board.lookup[piece_index][LEFT].push(left_ray);
            board.lookup[piece_index][RIGHT].push(right_ray);

            board.lookup[piece_index][ALL].push(all);
        }
    }

    fn capture_mask(&self, board: &mut Board, piece_index: usize, mask: BitBoard) -> BitBoard {
        let moving_team = board.state.team_to_move();
        let mut captures: BitBoard = BitBoard::empty();

        for rook in board.state.pieces[piece_index].and(moving_team).iter() {
            let pos = rook as usize;

            if board.lookup[piece_index][ALL][pos].and(mask).is_empty() {
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

    fn list_actions(&self, board: &mut Board, piece_index: usize) -> Vec<Action> {
        let moving_team = board.state.team_to_move();
        let mut actions: Vec<Action> = Vec::with_capacity(4);

        for rook in board.state.pieces[piece_index].and(moving_team).iter() {
            let pos = rook as usize;
            
            let up = ray_attacks_forward(board, pos, piece_index, UP);
            let down = ray_attacks_backward(board, pos, piece_index, DOWN);
            let left = ray_attacks_backward(board, pos, piece_index, LEFT);
            let right = ray_attacks_forward(board, pos, piece_index, RIGHT);

            let moves = up.or(down).or(right).or(left).and_not(moving_team);
            for movement in moves.iter() {
                actions.push(Action::from(rook, movement, piece_index))
            }
        }

        actions
    }

    fn make_move(&self, board: &mut Board, action: Action) -> HistoryState {
        make_chess_move(board, action)
    }

}

pub fn create_rook() -> Piece {
    Piece::new("r", "rook", Box::new(RookProcess))
}