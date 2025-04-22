
use crate::{bitboard::{BitBoard, BitInt, Edges}, game::{action::{make_chess_move, Action}, Board, Game}};

use super::{ray_attacks, repeat};

pub trait SliderMoves {
    fn rays<T: BitInt>(&self, pos: BitBoard<T>, edges: &Edges<T>)  -> Vec<BitBoard<T>>;
}

pub struct Slider<S : SliderMoves>(pub S);

impl <S : SliderMoves> Slider<S> {
    pub fn process<T: BitInt, const N: usize>(&self, game: &mut Game<T, N>, piece_index: usize) {
        let edges = game.edges[0];
        game.lookup[piece_index] = vec![];

        for index in 0..64 {
            let slider = BitBoard::index(index);
            let rays = self.0.rays(slider, &edges);

            while game.lookup[piece_index].len() < rays.len() + 1 {
                game.lookup[piece_index].push(vec![]);
            }

            let mut all = BitBoard::default();
            for (ray_ind, &ray) in rays.iter().enumerate() {
                game.lookup[piece_index][ray_ind].push(ray);
                all = all.or(ray);
            }

            let all_ind = rays.len();
            game.lookup[piece_index][all_ind].push(all);
        }
    }

    pub fn attacks<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, piece_index: usize, mask: BitBoard<T>) -> bool {
        let moving_team = board.state.team_to_move();

        for slider in board.state.pieces[piece_index].and(moving_team).iter() {
            let pos = slider as usize;
            let rays = board.game.lookup[piece_index].len() - 1;
            let all_ind = rays;

            if board.game.lookup[piece_index][all_ind][pos].and(mask).empty() {
                continue;
            }

            for dir in 0..rays {
                let ray = board.game.lookup[piece_index][dir][pos];
                if ray.and(mask).empty() {
                    continue;
                }

                let ray = ray_attacks(board, pos, piece_index, dir, ray);
                if ray.and(mask).set() { return true; }
            }
        }

        false
    }

    pub fn actions<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, piece_index: usize) -> Vec<Action> {
        let moving_team = board.state.team_to_move();
        let mut actions: Vec<Action> = Vec::with_capacity(30);

        let piece = piece_index as u8;
        for slider in board.state.pieces[piece_index].and(moving_team).iter() {
            let pos = slider as usize;
            let stored_pos = slider as u16;
            let rays = board.game.lookup[piece_index].len() - 1;

            let mut moves = BitBoard::default();

            for dir in 0..rays {
                let ray = board.game.lookup[piece_index][dir][pos];
                moves = moves.or(ray_attacks(board, pos, piece_index, dir, ray));
            }

            moves = moves.and_not(moving_team);

            for movement in moves.iter() {
                actions.push(Action::from(stored_pos, movement as u16, piece))
            }
        }

        actions
    }
}