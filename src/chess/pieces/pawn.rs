use crate::{bitboard::{BitBoard, BitInt}, game::{action::{index_to_square, make_chess_move, Action, ActionRecord}, piece::{Piece, PieceRules}, Board, BoardState, Team}};

#[inline(always)]
fn list_white_pawn_captures<T : BitInt>(board: &mut Board<T>, piece_index: usize) -> BitBoard<T> {
    let pawns = board.state.pieces[piece_index];
    let edges = board.game.edges[0];

    let up_once = pawns.and(board.state.white).up(1);
    let left_captures = up_once.and_not(edges.left).left(1);
    let right_captures = up_once.and_not(edges.right).right(1);

    left_captures.or(right_captures)
}

#[inline(always)]
fn list_black_pawn_captures<T: BitInt>(board: &mut Board<T>, piece_index: usize) -> BitBoard<T> {
    let pawns = board.state.pieces[piece_index];
    let edges = board.game.edges[0];

    let down_once = pawns.and(board.state.black).down(1);
    let left_captures = down_once.and_not(edges.left).left(1);
    let right_captures = down_once.and_not(edges.right).right(1);

    left_captures.or(right_captures)
}

#[inline(always)]
fn add_white_action<T: BitInt>(board: &mut Board<T>, actions: &mut Vec<Action>, action: Action) {
    if action.to <= (board.game.bounds.rows * (board.game.bounds.cols - 1)) - 1 {
        actions.push(action);
    } else {
        actions.push(action.with_info(3));
        actions.push(action.with_info(4));
        actions.push(action.with_info(5));
        actions.push(action.with_info(6));
    }
}

#[inline(always)]
fn add_black_action<T: BitInt>(board: &mut Board<T>, actions: &mut Vec<Action>, action: Action) {
    if action.to >= board.game.bounds.rows {
        actions.push(action);
    } else {
        actions.push(action.with_info(3));
        actions.push(action.with_info(4));
        actions.push(action.with_info(5));
        actions.push(action.with_info(6));
    }
}

#[inline(always)]
fn list_white_pawn_actions<T: BitInt>(board: &mut Board<T>, piece_index: usize) -> Vec<Action> {
    let edges = board.game.edges[0];

    let white = board.state.white;
    let black = board.state.black;
    let all = white.or(black);

    let pawns = board.state.pieces[piece_index];
    let white_pawns = pawns.and(white);

    let moves = white_pawns
        .up(1).and_not(all);
    let first_moves = white_pawns.and(board.state.first_move)
        .up(1).and_not(all)
        .up(1).and_not(all);

    let up_once = white_pawns.up(1);

    let possible_left_captures = up_once.and_not(edges.left).left(1);
    let possible_right_captures = up_once.and_not(edges.right).right(1);

    let left_captures = possible_left_captures.and(black);
    let right_captures = possible_right_captures.and(black);

    let mut actions: Vec<Action> = Vec::with_capacity(pawns.count() as usize);
    let piece = piece_index as u8;

    for movement in moves.iter() {
        let movement = movement as u8;
        add_white_action(board, &mut actions, Action::from(movement - 8, movement, piece));
    }
    for movement in first_moves.iter() {
        let movement = movement as u8;
        add_white_action(board, &mut actions, Action::from(movement - 16, movement, piece));
    }
    for movement in left_captures.iter() {
        let movement = movement as u8;
        add_white_action(board, &mut actions, Action::from(movement - 8 + 1, movement, piece));
    }
    for movement in right_captures.iter() {
        let movement = movement as u8;
        add_white_action(board, &mut actions, Action::from(movement - 8 - 1, movement, piece));
    }

    if let Some(ActionRecord::Action(last_move)) = board.history.last() {
        let last_piece_index = board.state.piece_at(last_move.to);
        if let Some(last_piece_index) = last_piece_index {
            let was_pawn_move = last_piece_index == piece_index;

            if was_pawn_move {
                let was_double_move = last_move.to.abs_diff(last_move.from) == 16;
                if was_double_move {
                    let capture = last_move.from - 8;
                    let target = BitBoard::<T>::index(capture.into());
                    if possible_left_captures.and(target).is_set() {
                        add_white_action(board, &mut actions, Action::from(capture - 8 + 1, capture, piece).with_info(1));
                    }

                    if possible_right_captures.and(target).is_set() {
                        add_white_action(board, &mut actions, Action::from(capture - 8 - 1, capture, piece).with_info(1));
                    }
                }
            }
        }
    }

    actions
}

#[inline(always)]
fn list_black_pawn_actions<T: BitInt>(board: &mut Board<T>, piece_index: usize) -> Vec<Action> {
    let edges = board.game.edges[0];

    let white = board.state.white;
    let black = board.state.black;
    let all = white.or(black);

    let pawns = board.state.pieces[piece_index];
    let black_pawns = pawns.and(black);

    let moves = black_pawns
        .down(1).and_not(all);
    let first_moves = black_pawns.and(board.state.first_move)
        .down(1).and_not(all)
        .down(1).and_not(all);

    let down_once = black_pawns.down(1);

    let possible_left_captures = down_once.and_not(edges.left).left(1);
    let possible_right_captures = down_once.and_not(edges.right).right(1);

    let left_captures = possible_left_captures.and(white);
    let right_captures = possible_right_captures.and(white);

    let mut actions: Vec<Action> = Vec::with_capacity(pawns.count() as usize);

    let piece = piece_index as u8;

    for movement in moves.iter() {
        let movement = movement as u8;
        add_black_action(board, &mut actions, Action::from(movement + 8, movement, piece));
    }
    for movement in first_moves.iter() {
        let movement = movement as u8;
        add_black_action(board, &mut actions, Action::from(movement + 16, movement, piece));
    }
    for movement in left_captures.iter() {
        let movement = movement as u8;
        add_black_action(board, &mut actions, Action::from(movement + 8 + 1, movement, piece));
    }
    for movement in right_captures.iter() {
        let movement = movement as u8;
        add_black_action(board, &mut actions, Action::from(movement + 8 - 1, movement, piece));
    }

    if let Some(ActionRecord::Action(last_move)) = board.history.last() {
        let last_piece_index = board.state.piece_at(last_move.to);
        if let Some(last_piece_index) = last_piece_index {
            let was_pawn_move = last_piece_index == piece_index;

            if was_pawn_move {
                let was_double_move = last_move.to.abs_diff(last_move.from) == 16;
                if was_double_move {
                    let capture = last_move.from + 8;
                    let target = BitBoard::<T>::index(capture.into());
                    if possible_left_captures.and(target).is_set() {
                        add_black_action(board, &mut actions, Action::from(capture + 8 + 1, capture, piece).with_info(1));
                    }

                    if possible_right_captures.and(target).is_set() {
                        add_black_action(board, &mut actions, Action::from(capture + 8 - 1, capture, piece).with_info(1));
                    }
                }
            }
        }
    }

    actions
}

fn make_en_passant_move<T: BitInt>(state: &mut BoardState<T>, action: Action) {
    let team = state.moving_team;
    let from = BitBoard::index(action.from);
    let to = BitBoard::index(action.to);

    // The taken pawn is one square ahead of the en passant destination.
    let taken_pos = match team { 
        Team::White => action.to - 8,
        Team::Black => action.to + 8 
    };
    let taken = BitBoard::index(taken_pos);

    let piece_index = action.piece as usize;

    match team {
        Team::White => {
            state.white = state.white.xor(from).or(to);
            state.black = state.black.xor(taken);
        }
        Team::Black => {
            state.black = state.black.xor(from).or(to);
            state.white = state.white.xor(taken);
        }
    }

    state.pieces[piece_index as usize] = state.pieces[piece_index as usize].xor(from).xor(taken).or(to);
    state.first_move = state.first_move.xor(from).xor(taken);
}

fn make_promotion_move<T: BitInt>(state: &mut BoardState<T>, action: Action) {
    let piece_index = action.piece as usize;
    let victim_index = state.piece_at(action.to);
    let promoted_piece_type = action.info - 2;

    let white = state.white;
    let black = state.black;

    let pawns = state.pieces[piece_index as usize];

    let from = BitBoard::index(action.from);
    let to = BitBoard::index(action.to);

    let team = state.moving_team;

    if let Some(piece_type) = victim_index {
        // Remove the captured piece type from its bitboard
        let piece = state.pieces[piece_type as usize];
        let same_piece_type = piece_type == piece_index;
        if !same_piece_type {
            state.pieces[piece_type as usize] = piece.xor(to);
        }

        match team {
            Team::White => {
                state.black = black.xor(to);
            }
            Team::Black => {
                state.white = white.xor(to);
            }
        }
    }

    // Remove the pawn
    state.pieces[piece_index as usize] = pawns.xor(from);

    // Add the new piece where the pawn left.
    state.pieces[promoted_piece_type as usize] = state.pieces[promoted_piece_type as usize].or(to);

    // Update the moved piece's team bitboard
    match team {
        Team::White => {
            state.white = white.xor(from).or(to);
        }
        Team::Black => {
            state.black = black.xor(from).or(to);
        }
    }

    let first_move = state.first_move.and_not(from.or(to));

    if first_move != state.first_move {
        state.first_move = first_move;
    }
}


pub struct PawnRules;

impl<T : BitInt> PieceRules<T> for PawnRules {
    fn load(&self, board: &mut Board<T>, piece_index: usize) {
        let edges = board.game.edges[0];

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

    fn capture_mask(&self, board: &mut Board<T>, piece_index: usize, _: BitBoard<T>) -> BitBoard<T> {
        if board.state.moving_team == Team::White {
            list_white_pawn_captures(board, piece_index)
        } else {
            list_black_pawn_captures(board, piece_index)
        }
    }

    fn make_move(&self, board: &mut Board<T>, action: Action) {
        match action.info {
            0 => make_chess_move(&mut board.state, action),
            1 => make_en_passant_move(&mut board.state, action),
            _ => make_promotion_move(&mut board.state, action)
        }
    }

    fn display_action(&self, board: &mut Board<T>, action: Action) -> Vec<String> {
        let promotion_piece_type = if action.info > 1 {
            board.game.pieces[(action.info - 2) as usize].symbol.to_lowercase()
        } else { "".to_string() };

        vec![
            format!("{}{}{}", index_to_square(action.from), index_to_square(action.to), promotion_piece_type)
        ]
    }

}

pub fn create_pawn<T : BitInt>() -> Piece<T> {
    Piece::new("p", "pawn", Box::new(PawnRules))
}