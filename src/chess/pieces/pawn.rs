use num::{PrimInt, Unsigned};

use crate::{bitboard::BitBoard, game::{action::{index_to_square, make_chess_move, restore_perfectly, Action, HistoryState, HistoryUpdate}, piece::{EmptyPieceProcessor, Piece, PieceProcessor}, Board, Team}};

#[inline(always)]
fn list_white_pawn_captures<T : PrimInt + Unsigned>(board: &mut Board<T>, piece_index: usize) -> BitBoard<T> {
    let pawns = board.state.pieces[piece_index];
    let edges = board.edges[0];

    let up_once = pawns.and(board.state.white).up(1);
    let left_captures = up_once.and_not(edges.left).left(1);
    let right_captures = up_once.and_not(edges.right).right(1);

    left_captures.or(right_captures)
}

#[inline(always)]
fn list_black_pawn_captures<T: PrimInt + Unsigned>(board: &mut Board<T>, piece_index: usize) -> BitBoard<T> {
    let pawns = board.state.pieces[piece_index];
    let edges = board.edges[0];

    let down_once = pawns.and(board.state.black).down(1);
    let left_captures = down_once.and_not(edges.left).left(1);
    let right_captures = down_once.and_not(edges.right).right(1);

    left_captures.or(right_captures)
}

#[inline(always)]
fn add_white_action<T: PrimInt + Unsigned>(board: &mut Board<T>, actions: &mut Vec<Action>, action: Action) {
    if action.to > (board.game.bounds.rows * (board.game.bounds.cols - 1)) - 1 {
        actions.push(action.with_info(2));
        actions.push(action.with_info(3));
        actions.push(action.with_info(4));
        actions.push(action.with_info(5));
    } else {
        actions.push(action);
    }
}

#[inline(always)]
fn add_black_action<T: PrimInt + Unsigned>(board: &mut Board<T>, actions: &mut Vec<Action>, action: Action) {
    if action.to < board.game.bounds.rows {
        actions.push(action.with_info(2));
        actions.push(action.with_info(3));
        actions.push(action.with_info(4));
        actions.push(action.with_info(5));
    } else {
        actions.push(action);
    }
}

#[inline(always)]
fn list_white_pawn_actions<T: PrimInt + Unsigned>(board: &mut Board<T>, piece_index: usize) -> Vec<Action> {
    let edges = board.edges[0];
    let mut black = board.state.black;

    if let Some(last_move) = board.history.last() {
        let was_pawn_move = last_move.piece_type == piece_index;
        let was_double_move = last_move.to.abs_diff(last_move.from) == 16;

        if was_pawn_move && was_double_move {
            black = black.or(BitBoard::index(last_move.from - 8));
        }
    }

    let pawns = board.state.pieces[piece_index];

    let all = board.state.white.or(board.state.black);

    let moves = pawns.and(board.state.white)
        .up(1).and_not(all);
    let first_moves = pawns.and(board.state.first_move).and(board.state.white)
        .up(1).and_not(all)
        .up(1).and_not(all);

    let up_once = pawns.and(board.state.white).up(1);
    let left_captures = up_once.and_not(edges.left).left(1).and(black);
    let right_captures = up_once.and_not(edges.right).right(1).and(black);

    let mut actions: Vec<Action> = Vec::with_capacity(pawns.count() as usize);
    for movement in moves.iter() {
        let movement = movement as usize;
        add_white_action(board, &mut actions, Action::from(movement - 8, movement, piece_index));
    }
    for movement in first_moves.iter() {
        let movement = movement as usize;
        add_white_action(board, &mut actions, Action::from(movement - 16, movement, piece_index));
    }
    for movement in left_captures.iter() {
        let movement = movement as usize;
        add_white_action(board, &mut actions, Action::from(movement - 8 + 1, movement, piece_index));
    }
    for movement in right_captures.iter() {
        let movement = movement as usize;
        add_white_action(board, &mut actions, Action::from(movement - 8 - 1, movement, piece_index));
    }

    actions
}

#[inline(always)]
fn list_black_pawn_actions<T: PrimInt + Unsigned>(board: &mut Board<T>, piece_index: usize) -> Vec<Action> {
    let edges = board.edges[0];
    let mut white = board.state.white;

    if let Some(last_move) = board.history.last() {
        let was_pawn_move = last_move.piece_type == piece_index;
        let was_double_move = last_move.to.abs_diff(last_move.from) == 16;

        if was_pawn_move && was_double_move {
            white = white.or(BitBoard::index(last_move.from + 8));
        }
    }

    let pawns = board.state.pieces[piece_index];

    let all = board.state.white.or(board.state.black);

    let moves = pawns.and(board.state.black)
        .down(1).and_not(all);
    let first_moves = pawns.and(board.state.first_move).and(board.state.black)
        .down(1).and_not(all)
        .down(1).and_not(all);

    let down_once = pawns.and(board.state.black).down(1);
    let left_captures = down_once.and_not(edges.left).left(1).and(white);
    let right_captures = down_once.and_not(edges.right).right(1).and(white);

    let mut actions: Vec<Action> = Vec::with_capacity(pawns.count() as usize);
    for movement in moves.iter() {
        let movement = movement as usize;
        add_black_action(board, &mut actions, Action::from(movement + 8, movement, piece_index));
    }
    for movement in first_moves.iter() {
        let movement = movement as usize;
        add_black_action(board, &mut actions, Action::from(movement + 16, movement, piece_index));
    }
    for movement in left_captures.iter() {
        let movement = movement as usize;
        add_black_action(board, &mut actions, Action::from(movement + 8 + 1, movement, piece_index));
    }
    for movement in right_captures.iter() {
        let movement = movement as usize;
        add_black_action(board, &mut actions, Action::from(movement + 8 - 1, movement, piece_index));
    }

    actions
}

fn make_en_passant_move<T: PrimInt + Unsigned>(board: &mut Board<T>, action: Action) -> HistoryState<T> {
    let is_white = board.state.moving_team == Team::White;
    let from = BitBoard::index(action.from);
    let to = BitBoard::index(action.to);

    // The taken pawn is one square ahead of the en passant destination.
    let taken_pos = if is_white { action.to - 8 } else { action.to + 8 };
    let taken = BitBoard::index(taken_pos);

    let mut updates: Vec<HistoryUpdate<T>> = Vec::with_capacity(4);

    updates.push(HistoryUpdate::Black(board.state.black));
    updates.push(HistoryUpdate::White(board.state.white));
    updates.push(HistoryUpdate::Piece(action.piece_type, board.state.pieces[action.piece_type]));
    updates.push(HistoryUpdate::FirstMove(board.state.first_move));

    if is_white {
        board.state.white = board.state.white.xor(from).or(to);
        board.state.black = board.state.black.xor(taken);
    } else {
        board.state.black = board.state.black.xor(from).or(to);
        board.state.white = board.state.white.xor(taken);
    }

    board.state.pieces[action.piece_type] = board.state.pieces[action.piece_type].xor(from).xor(taken).or(to);
    board.state.first_move = board.state.first_move.xor(from).xor(taken);

    HistoryState(updates)
}

fn make_promotion_move<T: PrimInt + Unsigned>(board: &mut Board<T>, action: Action) -> HistoryState<T> {
    let mut updates: Vec<HistoryUpdate<T>> = Vec::with_capacity(5);
    let piece_index = action.piece_type;
    let promoted_piece_type = (action.info - 1) as usize;

    let pawns = board.state.pieces[piece_index];

    let from = BitBoard::index(action.from);
    let to = BitBoard::index(action.to);

    let is_white = board.state.moving_team == Team::White;
    let opp_team = board.state.opposite_team();
    let is_capture = to.and(opp_team).is_set();

    // Save the moved piece's old state
    updates.push(HistoryUpdate::Piece(piece_index, pawns));

    // Add the promotion type's old state
    updates.push(HistoryUpdate::Piece(promoted_piece_type, board.state.pieces[promoted_piece_type]));

    if is_white {
        updates.push(HistoryUpdate::White(board.state.white));
    } else {
        updates.push(HistoryUpdate::Black(board.state.black));
    }

    if is_capture {
        // Remove the captured piece type from its bitboard
        for piece_type in 0..board.game.pieces.len() {
            if board.state.pieces[piece_type].and(to).is_set() {
                let same_piece_type = piece_type == piece_index;
                if !same_piece_type {
                    updates.push(HistoryUpdate::Piece(piece_type, board.state.pieces[piece_type]));
                    board.state.pieces[piece_type] = board.state.pieces[piece_type].xor(to);
                }

                break;
            }
        }

        if is_white {
            updates.push(HistoryUpdate::Black(board.state.black));
            board.state.black = board.state.black.xor(to);
        } else {
            updates.push(HistoryUpdate::White(board.state.white));
            board.state.white = board.state.white.xor(to);
        }
    }

    // Remove the pawn
    board.state.pieces[piece_index] = pawns.xor(from);

    // Add the new piece where the pawn left.
    board.state.pieces[promoted_piece_type] = board.state.pieces[promoted_piece_type].or(to);

    // Update the moved piece's team bitboard
    if is_white {
        board.state.white = board.state.white.xor(from).or(to);
    } else {
        board.state.black = board.state.black.xor(from).or(to);
    }

    let first_move = board.state.first_move.and_not(from.or(to));

    if first_move != board.state.first_move {
        updates.push(HistoryUpdate::FirstMove(board.state.first_move));
        board.state.first_move = first_move;
    }

    HistoryState(updates)
}


pub struct PawnProcess;

impl<T : PrimInt + Unsigned> PieceProcessor<T> for PawnProcess {
    fn process(&self, board: &mut Board<T>, piece_index: usize) {
        let edges = board.edges[0];

        let pawns = board.state.pieces[piece_index];

        let moved_white_pawns = pawns.and(board.state.white).and_not(edges.bottom.up(1));
        let moved_black_pawns = pawns.and(board.state.black).and_not(edges.top.down(1));

        board.state.first_move = board.state.first_move.and_not(moved_white_pawns).and_not(moved_black_pawns);
    }

    fn list_actions(&self, board: &mut Board<T>, piece_index: usize) -> Vec<Action> {
        if board.state.moving_team == Team::White {
            list_white_pawn_actions(board, piece_index)
        } else {
            list_black_pawn_actions(board, piece_index)
        }
    }

    fn capture_mask(&self, board: &mut Board<T>, piece_index: usize, mask: BitBoard<T>) -> BitBoard<T> {
        if board.state.moving_team == Team::White {
            list_white_pawn_captures(board, piece_index)
        } else {
            list_black_pawn_captures(board, piece_index)
        }
    }

    fn make_move(&self, board: &mut Board<T>, action: Action) -> HistoryState<T> {
        // If a move is a capture (because it's not going directly in front), but there's no piece where it's capturing, well it must be en passant!
        let non_capture = action.from.abs_diff(action.to) % 8 == 0;
        let en_passant = !non_capture && BitBoard::index(action.to).and(board.state.opposite_team()).is_empty();

        if en_passant {
            make_en_passant_move(board, action)
        } else if action.info > 0 {
            make_promotion_move(board, action)
        } else {
            make_chess_move(board, action)
        }
    }

    fn display_action(&self, board: &mut Board<T>, action: Action) -> Vec<String> {
        let promotion_piece_type = if action.info > 0 {
            board.game.pieces[(action.info - 1) as usize].symbol.to_lowercase()
        } else { "".to_string() };

        vec![
            format!("{}{}{}", index_to_square(action.from), index_to_square(action.to), promotion_piece_type)
        ]
    }

}

pub fn create_pawn<T : PrimInt + Unsigned>() -> Piece<T> {
    Piece::new("p", "pawn", Box::new(PawnProcess))
}