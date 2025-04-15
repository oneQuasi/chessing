use crate::bitboard::{BitBoard, BitInt};

use super::{Board, BoardState, Team};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Action {
    pub from: u8,
    pub to: u8,
    pub info: u8,
    pub piece: u8
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionRecord {
    Action(Action),
    Null()
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

pub fn square_to_index(square: &str) -> Option<u8> {
    let bytes = square.as_bytes();
    if bytes.len() != 2 {
        return None;
    }

    let file = bytes[0];
    let rank = bytes[1];

    if file < b'a' || file > b'h' || rank < b'1' || rank > b'8' {
        return None;
    }

    let file_index = file - b'a';
    let rank_index = rank - b'1';

    Some(rank_index * 8 + file_index)
}

impl Action {
    pub fn from(from: u8, to: u8, piece: u8) -> Action {
        Action { from, to, piece, info: 0 }
    }

    pub fn with_info(self, info: u8) -> Action {
        Action { from: self.from, to: self.to, piece: self.piece, info }
    }
}

#[inline(always)]
pub fn make_chess_move<T : BitInt>(state: &mut BoardState<T>, action: Action) {
    let to_idx = action.to as usize;
    let from_idx = action.from as usize;

    let piece_index = state.mailbox[from_idx] - 1;
    let mailbox = state.mailbox[to_idx];
    
    let from = BitBoard::index(action.from);
    let to = BitBoard::index(action.to);

    let team = state.moving_team;
    let is_capture = mailbox > 0;

    // Save the moved piece's old state
    let piece = state.pieces[piece_index as usize];

    let white = state.white;
    let black = state.black;

    if is_capture {
        let piece_type = mailbox - 1;

        // Remove the captured piece type from its bitboard
        let same_piece_type = piece_type == piece_index;
        if !same_piece_type {
            let piece = state.pieces[piece_type as usize];
            state.pieces[piece_type as usize] = piece.xor(to);
        }

        // Remove the captured piece from the opposite team's bitboard
        match team {
            Team::White => {
                state.black = black.xor(to);
            }
            Team::Black => {
                state.white = white.xor(to);
            }
        }
    }

    // Update the moved piece's piece bitboard
    state.pieces[piece_index as usize] = piece.xor(from).or(to);

    state.mailbox[from_idx] = 0;
    state.mailbox[to_idx] = piece_index + 1;

    // Update the moved piece's team bitboard
    match team {
        Team::White => {
            state.white = white.xor(from).or(to);
        }
        Team::Black => {
            state.black = black.xor(from).or(to);
        }
    }

    if state.first_move.and(from.or(to)).is_set() {
        state.first_move = state.first_move.and_not(from.or(to));
    }
}