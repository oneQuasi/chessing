
use crate::{bitboard::{BitBoard, BitInt, Edges}, game::{action::{make_chess_move, Action}, Board, Game}};

use super::{ray_attacks, repeat};

pub trait SliderMoves : Copy + Clone {
    fn rays<T: BitInt>(&self, pos: BitBoard<T>, edges: &Edges<T>)  -> Vec<BitBoard<T>>;
}

pub struct Slider<S : SliderMoves>(pub S);

impl<S : SliderMoves> Slider<S> {
    pub fn list_moves<T: BitInt, const N: usize>(
        game: &Game<T, N>,
        piece_index: usize,
        pos: usize,
        blockers: BitBoard<T>
    ) -> BitBoard<T> {
        let rays = game.lookup[piece_index].len() - 1;
    
        let mut moves = BitBoard::default();
    
        for dir in 0..rays {
            let ray = game.lookup[piece_index][dir][pos];
            moves = moves.or(ray_attacks(game, piece_index, pos, dir, ray, blockers));
        }
    
        moves
    }
    
    pub fn can_attack<T: BitInt, const N: usize>(
        game: &Game<T, N>,
        piece_index: usize,
        pos: usize,
        blockers: BitBoard<T>,
        mask: BitBoard<T>
    ) -> bool {
        let rays = game.lookup[piece_index].len() - 1;
        let all_ind = rays;
    
        if game.lookup[piece_index][all_ind][pos].and(mask).empty() {
            return false;
        }
    
        for dir in 0..rays {
            // If this ray couldn't be attacked unblocked, break.
            let ray = game.lookup[piece_index][dir][pos];
            if ray.and(mask).empty() {
                continue;
            }
    
            let ray = ray_attacks(game, piece_index, pos, dir, ray, blockers);
            if ray.and(mask).set() { return true; }
        }
    
        false
    }
}

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
        let team = board.state.team_to_move();
        let blockers = board.state.black.or(board.state.white);
    
        board.state.pieces[piece_index]
            .and(team)
            .iter()
            .any(|pos| Slider::<S>::can_attack(&board.game, piece_index, pos as usize, blockers, mask))
    }

    pub fn actions<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, piece_index: usize) -> Vec<Action> {
        let team = board.state.team_to_move();
        let blockers = board.state.black.or(board.state.white);
        let piece = piece_index as u8;
    
        board.state.pieces[piece_index]
            .and(team)
            .iter()
            .flat_map(|pos| {
                let from = pos as u16;
                let moves = Slider::<S>::list_moves(&board.game, piece_index, pos as usize, blockers).and_not(team);
                moves.iter().map(move |to| Action::from(from, to as u16, piece))
            })
            .collect()
    }
}