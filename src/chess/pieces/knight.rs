
use crate::{bitboard::{BitBoard, BitInt}, game::{action::{make_chess_move, Action}, piece::{Piece, PieceRules}, Board, Game}};

pub struct KnightRules;

impl<T: BitInt, const N: usize> PieceRules<T, N> for KnightRules {
    fn process(&self, game: &mut Game<T, N>, piece_index: usize) {
        let edges = game.edges[0];
        let deep_edges = game.edges[1];
        game.lookup[piece_index] = vec![ vec![] ];

        for index in 0..64 {
            let knight = BitBoard::index(index);

            let two_right = knight.and_not(deep_edges.right).right(2);
            let two_left = knight.and_not(deep_edges.left).left(2);
            let two_up = knight.and_not(deep_edges.top).up(2);
            let two_down = knight.and_not(deep_edges.bottom).down(2);
        
            let horizontal = two_right.or(two_left);
            let vertical = two_up.or(two_down);
        
            let horizontal_moves = horizontal.and_not(edges.top).up(1).or(horizontal.and_not(edges.bottom).down(1));
            let vertical_moves = vertical.and_not(edges.right).right(1).or(vertical.and_not(edges.left).left(1));

            let moves = horizontal_moves.or(vertical_moves);
            game.lookup[piece_index][0].push(moves);
        }
    }
    
    fn capture_mask(&self, board: &mut Board<T, N>, piece_index: usize, _: BitBoard<T>) -> BitBoard<T> {
        let mut mask = BitBoard::empty();
        let moving_team = board.state.team_to_move();
        for knight in board.state.pieces[piece_index].and(moving_team).iter() {
            mask = mask.or(board.game.lookup[piece_index][0][knight as usize]);
        }
        mask
    }


    fn list_actions(&self, board: &mut Board<T, N>, piece_index: usize) -> Vec<Action> {
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

    fn make_move(&self, board: &mut Board<T, N>, action: Action) {
        make_chess_move(&mut board.state, action)
    }
}