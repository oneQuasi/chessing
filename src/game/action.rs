use crate::bitboard::{BitBoard, BitInt};

use super::{Board, Team};

#[derive(Clone, Copy, Debug)]
pub struct Action {
    pub from: u8,
    pub to: u8,
    pub info: u8,
    pub piece_type: u8
}

pub fn index_to_square(index: u8) -> String {
    if index > 63 {
        return format!("N/A");
    }

    let file = (index % 8) as u8;
    let rank = (index / 8) as u8;

    let file_char = (b'a' + file) as char;
    let rank_char = (b'1' + rank) as char;

    format!("{}{}", file_char, rank_char)
}

impl Action {
    pub fn from(from: u8, to: u8, piece_type: u8) -> Action {
        Action { from, to, info: 0, piece_type }
    }

    pub fn with_info(self, info: u8) -> Action {
        Action { from: self.from, to: self.to, info, piece_type: self.piece_type }
    }
}

pub enum HistoryUpdate<T : BitInt> {
    White(BitBoard<T>),
    Black(BitBoard<T>),
    FirstMove(BitBoard<T>),
    Piece(u8, BitBoard<T>)
}

pub struct HistoryState<T : BitInt>(pub Vec<HistoryUpdate<T>>);

/// For debugging or development purposes only, restores every original BitBoard
pub fn restore_perfectly<T : BitInt>(board: &mut Board<T>) -> HistoryState<T> {
    let mut updates: Vec<HistoryUpdate<T>> = Vec::with_capacity(12);

    for piece_type in 0..board.game.pieces.len() as u8 {
        updates.push(HistoryUpdate::Piece(piece_type, board.state.pieces[piece_type as usize]));
    }

    updates.push(HistoryUpdate::White(board.state.white));
    updates.push(HistoryUpdate::Black(board.state.black));

    updates.push(HistoryUpdate::FirstMove(board.state.first_move));

    HistoryState(updates)

}

#[inline(always)]
pub fn make_chess_move<T : BitInt>(board: &mut Board<T>, action: Action) -> HistoryState<T> {
    let mut updates: Vec<HistoryUpdate<T>> = Vec::with_capacity(6);
    let piece_index = action.piece_type;
    
    let from = BitBoard::index(action.from.into());
    let to = BitBoard::index(action.to.into());

    let is_white = board.state.moving_team == Team::White;
    let opp_team = board.state.opposite_team();
    let is_capture = to.and(opp_team).is_set();

    // Save the moved piece's old state
    let piece = board.state.pieces[piece_index as usize];
    updates.push(HistoryUpdate::Piece(piece_index, piece));

    let white = board.state.white;
    let black = board.state.black;

    if is_capture {
        // Remove the captured piece type from its bitboard
        for piece_type in 0..board.game.pieces.len() as u8 {
            let piece = board.state.pieces[piece_type as usize];
            if piece.and(to).is_set() {
                let same_piece_type = piece_type == piece_index;
                if !same_piece_type {
                    updates.push(HistoryUpdate::Piece(piece_type, piece));
                    board.state.pieces[piece_type as usize] = piece.xor(to);
                }

                break;
            }
        }

        // Remove the captured piece from the opposite team's bitboard
        if is_white {
            updates.push(HistoryUpdate::Black(black));
            board.state.black = black.xor(to);
        } else {
            updates.push(HistoryUpdate::White(white));
            board.state.white = white.xor(to);
        }
    }

    // Update the moved piece's piece bitboard
    board.state.pieces[piece_index as usize] = piece.xor(from).or(to);

    // Update the moved piece's team bitboard
    if is_white {
        updates.push(HistoryUpdate::White(white));
        board.state.white = white.xor(from).or(to);
    } else {
        updates.push(HistoryUpdate::Black(black));
        board.state.black = black.xor(from).or(to);
    }

    if board.state.first_move.and(from.or(to)).is_set() {
        updates.push(HistoryUpdate::FirstMove(board.state.first_move));
        board.state.first_move = board.state.first_move.and_not(from.or(to));
    }

    HistoryState(updates)
}