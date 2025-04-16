

use crate::{bitboard::{BitBoard, BitInt}, game::action::index_to_square};

use super::{action::Action, Board, Game};

/// `PieceRules` handles making piece-specific changes to the board.
/// For instance, `PieceRules` is where the generation of a piece's lookup table happens.
pub trait PieceRules<T : BitInt, const N: usize> {
    fn process(&self, _game: &mut Game<T, N>, _piece_index: usize) {}
    fn load(&self, _board: &mut Board<T, N>, _piece_index: usize) {}
    
    fn list_actions(&self, board: &mut Board<T, N>, piece_index: usize) -> Vec<Action>;
    fn make_move(&self, board: &mut Board<T, N>, action: Action);

    fn display_action(&self, _board: &mut Board<T, N>, action: Action) -> Vec<String> {
        vec![
            format!("{}{}", index_to_square(action.from), index_to_square(action.to))
        ]
    }

    fn display_uci_action(&self, board: &mut Board<T, N>, action: Action) -> String {
        self.display_action(board, action)[0].clone()
    }

    /// Only useful for chess; allows us to optimize checks
    fn capture_mask(&self, board: &mut Board<T, N>, piece_index: usize, mask: BitBoard<T>) -> BitBoard<T>;
}

pub struct Piece<T : BitInt, const N: usize> {
    pub symbol: String,
    pub name: String,
    pub rules: Box<dyn PieceRules<T, N>>
}

impl<T : BitInt, const N: usize> Piece<T, N> {
    pub fn new(symbol: &str, name: &str, processor: Box<dyn PieceRules<T, N>>) -> Piece<T, N> {
        Piece {
            symbol: symbol.to_string(),
            name: name.to_string(),
            rules: processor
        }
    }
}

pub struct EmptyPieceRules;

impl<T : BitInt, const N: usize> PieceRules<T, N> for EmptyPieceRules {
    fn process(&self, _: &mut Game<T, N>, _: usize) {}
    fn list_actions(&self, _: &mut Board<T, N>, _: usize) -> Vec<Action> {
        vec![]
    }
    fn make_move(&self, _: &mut Board<T, N>, _: Action) {
        unimplemented!("No make_move implemented.");
    }
    fn capture_mask(&self, _: &mut Board<T, N>, _: usize, _: BitBoard<T>) -> BitBoard<T> {
        BitBoard::empty()
    }
}