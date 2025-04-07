use std::iter::Product;

use crate::{bitboard::BitBoard, game::action::{index_to_square, HistoryUpdate}};

use super::{action::{Action, HistoryState}, Board, Team};

/// `PieceProcessor` handles making piece-specific changes to the board.
/// For instance, `PieceProcessor` is where the generation of a piece's lookup table happens.
pub trait PieceProcessor {
    fn process(&self, board: &mut Board, piece_index: usize) {}
    
    fn list_actions(&self, board: &mut Board, piece_index: usize) -> Vec<Action>;
    fn make_move(&self, board: &mut Board, action: Action) -> HistoryState;

    fn display_action(&self, board: &mut Board, action: Action) -> String {
        format!("{}{}", index_to_square(action.from), index_to_square(action.to))
    }

    /// Only useful for chess; allows us to optimize checks
    fn capture_mask(&self, board: &mut Board, piece_index: usize, mask: BitBoard) -> BitBoard {
        BitBoard::empty()
    }
}

pub struct Piece {
    pub symbol: String,
    pub name: String,
    pub processor: Box<dyn PieceProcessor>
}

impl Piece {
    pub fn new(symbol: &str, name: &str, processor: Box<dyn PieceProcessor>) -> Piece {
        Piece {
            symbol: symbol.to_string(),
            name: name.to_string(),
            processor
        }
    }
}

pub struct EmptyPieceProcessor;

impl PieceProcessor for EmptyPieceProcessor {
    fn process(&self, board: &mut Board, piece_index: usize) {}
    fn list_actions(&self, board: &mut Board, piece_index: usize) -> Vec<Action> {
        vec![]
    }
    fn make_move(&self, board: &mut Board, action: Action) -> HistoryState {
        unimplemented!("No make_move implemented.");
    }
}