use std::iter::Product;

use num::{PrimInt, Unsigned};

use crate::{bitboard::BitBoard, game::action::{index_to_square, HistoryUpdate}};

use super::{action::{Action, HistoryState}, Board, Team};

/// `PieceProcessor` handles making piece-specific changes to the board.
/// For instance, `PieceProcessor` is where the generation of a piece's lookup table happens.
pub trait PieceProcessor<T : PrimInt + Unsigned> {
    fn process(&self, board: &mut Board<T>, piece_index: usize) {}
    
    fn list_actions(&self, board: &mut Board<T>, piece_index: usize) -> Vec<Action>;
    fn make_move(&self, board: &mut Board<T>, action: Action) -> HistoryState<T>;

    fn display_action(&self, board: &mut Board<T>, action: Action) -> Vec<String> {
        vec![
            format!("{}{}", index_to_square(action.from), index_to_square(action.to))
        ]
    }

    fn display_uci_action(&self, board: &mut Board<T>, action: Action) -> String {
        self.display_action(board, action)[0].clone()
    }

    /// Only useful for chess; allows us to optimize checks
    fn capture_mask(&self, board: &mut Board<T>, piece_index: usize, mask: BitBoard<T>) -> BitBoard<T> {
        BitBoard::empty()
    }
}

pub struct Piece<T : PrimInt + Unsigned> {
    pub symbol: String,
    pub name: String,
    pub processor: Box<dyn PieceProcessor<T>>
}

impl<T : PrimInt + Unsigned> Piece<T> {
    pub fn new(symbol: &str, name: &str, processor: Box<dyn PieceProcessor<T>>) -> Piece<T> {
        Piece {
            symbol: symbol.to_string(),
            name: name.to_string(),
            processor
        }
    }
}

pub struct EmptyPieceProcessor;

impl<T : PrimInt + Unsigned> PieceProcessor<T> for EmptyPieceProcessor {
    fn process(&self, board: &mut Board<T>, piece_index: usize) {}
    fn list_actions(&self, board: &mut Board<T>, piece_index: usize) -> Vec<Action> {
        vec![]
    }
    fn make_move(&self, board: &mut Board<T>, action: Action) -> HistoryState<T> {
        unimplemented!("No make_move implemented.");
    }
}