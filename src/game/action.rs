use crate::bitboard::BitBoard;

use super::{Board, Team};

#[derive(Clone, Copy, Debug)]
pub struct Action {
    pub from: u16,
    pub to: u16,
    pub info: u16,
    pub piece_type: usize
}

pub fn index_to_square(index: u16) -> String {
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
    pub fn from(from: u32, to: u32, piece_type: usize) -> Action {
        Action { from: from as u16, to: to as u16, info: 0, piece_type }
    }

    pub fn with_info(self, info: u16) -> Action {
        Action { from: self.from, to: self.to, info, piece_type: self.piece_type }
    }
}

pub enum HistoryUpdate {
    White(BitBoard),
    Black(BitBoard),
    FirstMove(BitBoard),
    Piece(usize, BitBoard)
}

pub struct HistoryState(pub Vec<HistoryUpdate>);

/// For debugging or development purposes only, restores every original BitBoard
pub fn restore_perfectly(board: &mut Board) -> HistoryState {
    let mut updates: Vec<HistoryUpdate> = Vec::with_capacity(12);

    for piece_type in 0..board.game.pieces.len() {
        updates.push(HistoryUpdate::Piece(piece_type, board.state.pieces[piece_type]));
    }

    updates.push(HistoryUpdate::White(board.state.white));
    updates.push(HistoryUpdate::Black(board.state.black));

    updates.push(HistoryUpdate::FirstMove(board.state.first_move));

    HistoryState(updates)

}

#[inline(always)]
pub fn make_chess_move(board: &mut Board, action: Action) -> HistoryState {
    let mut updates: Vec<HistoryUpdate> = Vec::with_capacity(6);
    let piece_index = action.piece_type;
    
    let from = BitBoard::index(action.from);
    let to = BitBoard::index(action.to);

    let is_white = board.state.moving_team == Team::White;
    let opp_team = board.state.opposite_team();
    let is_capture = to.and(opp_team).is_set();

    // Save the moved piece's old state
    let piece = board.state.pieces[piece_index];
    updates.push(HistoryUpdate::Piece(piece_index, piece));

    let white = board.state.white;
    let black = board.state.black;

    if is_capture {
        // Remove the captured piece type from its bitboard
        for piece_type in 0..board.game.pieces.len() {
            let piece = board.state.pieces[piece_type];
            if piece.and(to).is_set() {
                let same_piece_type = piece_type == piece_index;
                if !same_piece_type {
                    updates.push(HistoryUpdate::Piece(piece_type, piece));
                    board.state.pieces[piece_type] = piece.xor(to);
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
    board.state.pieces[piece_index] = piece.xor(from).or(to);

    // Update the moved piece's team bitboard
    if is_white {
        updates.push(HistoryUpdate::White(white));
        board.state.white = white.xor(from).or(to);
    } else {
        updates.push(HistoryUpdate::Black(black));
        board.state.black = black.xor(from).or(to);
    }

    let first_move = board.state.first_move.and_not(from.or(to));
    if first_move != board.state.first_move {
        updates.push(HistoryUpdate::FirstMove(board.state.first_move));
        board.state.first_move = first_move;
    }

    HistoryState(updates)
}