use crate::bitboard::{BitBoard, BitInt};

use super::{Board, BoardState, Team};

#[derive(Clone, Copy, Debug)]
pub struct Action {
    pub from: u8,
    pub to: u8,
    pub info: u8
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
    pub fn from(from: u8, to: u8) -> Action {
        Action { from, to, info: 0 }
    }

    pub fn with_info(self, info: u8) -> Action {
        Action { from: self.from, to: self.to, info }
    }
}

pub enum HistoryUpdate<T : BitInt> {
    White(BitBoard<T>),
    Black(BitBoard<T>),
    FirstMove(BitBoard<T>),
    Piece(u8, BitBoard<T>),
    Mailbox(u8, u8)
}


pub struct HistoryState<T : BitInt> (pub Vec<HistoryUpdate<T>>);

#[inline(always)]
pub fn make_chess_move<T : BitInt>(state: &mut BoardState<T>, action: Action) -> HistoryState<T> {
    let to_idx = action.to as usize;
    let from_idx = action.from as usize;

    let mut updates: Vec<HistoryUpdate<T>> = Vec::with_capacity(7);
    let piece_index = state.mailbox[from_idx] - 1;
    let mailbox = state.mailbox[to_idx];

    updates.push(HistoryUpdate::Mailbox(action.from, piece_index + 1));
    updates.push(HistoryUpdate::Mailbox(action.to, mailbox));
    
    let from = BitBoard::index(action.from);
    let to = BitBoard::index(action.to);

    let team = state.moving_team;
    let is_capture = mailbox > 0;

    // Save the moved piece's old state
    let piece = state.pieces[piece_index as usize];
    updates.push(HistoryUpdate::Piece(piece_index, piece));

    let white = state.white;
    let black = state.black;

    if is_capture {
        let piece_type = mailbox - 1;

        // Remove the captured piece type from its bitboard
        let same_piece_type = piece_type == piece_index;
        if !same_piece_type {
            let piece = state.pieces[piece_type as usize];
            updates.push(HistoryUpdate::Piece(piece_type, piece));
            state.pieces[piece_type as usize] = piece.xor(to);
        }

        // Remove the captured piece from the opposite team's bitboard
        match team {
            Team::White => {
                updates.push(HistoryUpdate::Black(black));
                state.black = black.xor(to);
            }
            Team::Black => {
                updates.push(HistoryUpdate::White(white));
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
            updates.push(HistoryUpdate::White(white));
            state.white = white.xor(from).or(to);
        }
        Team::Black => {
            updates.push(HistoryUpdate::Black(black));
            state.black = black.xor(from).or(to);
        }
    }

    if state.first_move.and(from.or(to)).is_set() {
        updates.push(HistoryUpdate::FirstMove(state.first_move));
        state.first_move = state.first_move.and_not(from.or(to));
    }

    HistoryState(updates)
}