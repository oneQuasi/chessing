

use crate::{bitboard::{BitBoard, BitInt}, game::action::index_to_square};

use super::{action::Action, Board};

/// `PieceProcessor` handles making piece-specific changes to the board.
/// For instance, `PieceProcessor` is where the generation of a piece's lookup table happens.
pub trait PieceProcessor<T : BitInt> {
    fn process(&self, _board: &mut Board<T>, _piece_index: usize) {}
    
    fn list_actions(&self, board: &mut Board<T>, piece_index: usize) -> Vec<Action>;
    fn make_move(&self, board: &mut Board<T>, action: Action);

    fn display_action(&self, _board: &mut Board<T>, action: Action) -> Vec<String> {
        vec![
            format!("{}{}", index_to_square(action.from), index_to_square(action.to))
        ]
    }

    fn display_uci_action(&self, board: &mut Board<T>, action: Action) -> String {
        self.display_action(board, action)[0].clone()
    }

    /// Only useful for chess; allows us to optimize checks
    fn capture_mask(&self, board: &mut Board<T>, piece_index: usize, mask: BitBoard<T>) -> BitBoard<T>;
}

pub struct Piece<T : BitInt> {
    pub symbol: String,
    pub name: String,
    pub processor: Box<dyn PieceProcessor<T>>
}

impl<T : BitInt> Piece<T> {
    pub fn new(symbol: &str, name: &str, processor: Box<dyn PieceProcessor<T>>) -> Piece<T> {
        Piece {
            symbol: symbol.to_string(),
            name: name.to_string(),
            processor
        }
    }
}

pub struct EmptyPieceProcessor;

impl<T : BitInt> PieceProcessor<T> for EmptyPieceProcessor {
    fn process(&self, _: &mut Board<T>, _: usize) {}
    fn list_actions(&self, _: &mut Board<T>, _: usize) -> Vec<Action> {
        vec![]
    }
    fn make_move(&self, _: &mut Board<T>, _: Action) {
        unimplemented!("No make_move implemented.");
    }
    fn capture_mask(&self, _: &mut Board<T>, _: usize, _: BitBoard<T>) -> BitBoard<T> {
        BitBoard::empty()
    }
}