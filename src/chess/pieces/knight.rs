
use crate::{bitboard::{BitBoard, BitInt}, game::{action::{make_chess_move, Action}, Board, Game}};

pub struct Knight;

impl Knight {
    pub fn process<T: BitInt, const N: usize>(&self, game: &mut Game<T, N>, piece_index: usize) {
        let edges = game.edges[0];
        let deep_edges = game.edges[1];
        game.lookup[piece_index] = vec![ vec![] ];

        for index in 0..64 {
            let knight = BitBoard::index(index);

            let two_right = knight.try_right(&edges, 2);
            let two_left = knight.try_left(&edges, 2);
            let two_up = knight.try_up(&edges, 2);
            let two_down = knight.try_down(&edges, 2);
        
            let horizontal = two_right.or(two_left);
            let vertical = two_up.or(two_down);
        
            let horizontal_moves = horizontal.try_up(&edges, 1).or(horizontal.try_down(&edges, 1));
            let vertical_moves = vertical.try_right(&edges, 1).or(vertical.try_left(&edges, 1));

            let moves = horizontal_moves.or(vertical_moves);
            game.lookup[piece_index][0].push(moves);
        }
    }
    
    pub fn attacks<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, piece_index: usize, mask: BitBoard<T>) -> BitBoard<T> {
        let moving_team = board.state.team_to_move();
        for knight in board.state.pieces[piece_index].and(moving_team).iter() {
            let attacks = board.game.lookup[piece_index][0][knight as usize];
            if attacks.and(mask).set() {
                return mask;
            }
        }
        BitBoard::default()
    }


    pub fn actions<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, piece_index: usize) -> Vec<Action> {
        let moving_team = board.state.team_to_move();
        let mut actions: Vec<Action> = Vec::with_capacity(8);

        let piece = piece_index as u8;
        for knight in board.state.pieces[piece_index].and(moving_team).iter() {
            let pos = knight as u16;
            let moves = board.game.lookup[piece_index][0][knight as usize].and_not(moving_team);
            for movement in moves.iter() {
                actions.push(Action::from(pos, movement as u16, piece))
            }
        }
    
        actions
    }
}