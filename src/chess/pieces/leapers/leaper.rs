use crate::{bitboard::{BitBoard, BitInt, Edges}, game::{action::Action, Board, Game}};

pub trait LeaperMoves {
    fn leaps<T: BitInt>(&self, pos: BitBoard<T>, edges: &Edges<T>) -> BitBoard<T>;
}

pub struct Leaper<S : LeaperMoves>(pub S);

impl<S : LeaperMoves> Leaper<S> {
    pub fn process<T: BitInt, const N: usize>(&self, game: &mut Game<T, N>, piece_index: usize) {
        let edges = game.edges[0];
        game.lookup[piece_index] = vec![ vec![] ];

        for index in 0..64 {
            let leaper = BitBoard::index(index);
            let moves = self.0.leaps(leaper, &edges);

            game.lookup[piece_index][0].push(moves);
        }
    }
    
    pub fn attacks<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, piece_index: usize, mask: BitBoard<T>) -> bool {
        let moving_team = board.state.team_to_move();
        for leaper in board.state.pieces[piece_index].and(moving_team).iter() {
            let attacks = board.game.lookup[piece_index][0][leaper as usize];
            if attacks.and(mask).set() {
                return true;
            }
        }
        
        false
    }


    pub fn actions<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, piece_index: usize) -> Vec<Action> {
        let moving_team = board.state.team_to_move();
        let mut actions: Vec<Action> = Vec::with_capacity(8);

        let piece = piece_index as u8;
        for leaper in board.state.pieces[piece_index].and(moving_team).iter() {
            let pos = leaper as u16;
            let moves = board.game.lookup[piece_index][0][leaper as usize].and_not(moving_team);
            for movement in moves.iter() {
                actions.push(Action::from(pos, movement as u16, piece))
            }
        }
    
        actions
    }
}