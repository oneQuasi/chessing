use crate::bitboard::{BitBoard, BitInt};

use super::{Board, Team};

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
pub fn make_chess_move<T : BitInt>(board: &mut Board<T>, action: Action) -> HistoryState<T> {
    let mut updates: Vec<HistoryUpdate<T>> = Vec::with_capacity(8);
    let piece_index = board.state.mailbox[action.from as usize] - 1;
    let mailbox = board.state.mailbox[action.to as usize];

    updates.push(HistoryUpdate::Mailbox(action.from, piece_index + 1));
    updates.push(HistoryUpdate::Mailbox(action.to, mailbox));
    
    let from = BitBoard::index(action.from.into());
    let to = BitBoard::index(action.to.into());

    let is_white = board.state.moving_team == Team::White;
    let is_capture = mailbox > 0;

    // Save the moved piece's old state
    let piece = board.state.pieces[piece_index as usize];
    updates.push(HistoryUpdate::Piece(piece_index, piece));

    let white = board.state.white;
    let black = board.state.black;

    if is_capture {
        let piece_type = mailbox - 1;

        // Remove the captured piece type from its bitboard
        let piece = board.state.pieces[piece_type as usize];
        let same_piece_type = piece_type == piece_index;
        if !same_piece_type {
            updates.push(HistoryUpdate::Piece(piece_type, piece));
            board.state.pieces[piece_type as usize] = piece.xor(to);
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

    board.state.mailbox[action.from as usize] = 0;
    board.state.mailbox[action.to as usize] = piece_index + 1;

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