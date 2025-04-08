use num::{PrimInt, Unsigned};

use crate::{bitboard::BitBoard, game::{action::{index_to_square, make_chess_move, Action, HistoryState, HistoryUpdate}, piece::{Piece, PieceProcessor}, Board, Team}};

fn make_castling_move<T : PrimInt + Unsigned>(board: &mut Board<T>, action: Action) -> HistoryState<T> {
    let mut updates: Vec<HistoryUpdate<T>> = Vec::with_capacity(4);

    if board.state.moving_team == Team::White {
        updates.push(HistoryUpdate::White(board.state.white));
    } else {
        updates.push(HistoryUpdate::Black(board.state.black));
    }

    updates.push(HistoryUpdate::Piece(action.piece_type, board.state.pieces[action.piece_type]));

    let rook_ind = board.find_piece("rook").expect("Cannot castle w/o rook");
    updates.push(HistoryUpdate::Piece(3, board.state.pieces[rook_ind]));

    // This isn't Fischer-Random compatible yet.

    let (relocated_king, relocated_rook) = if action.to > action.from {
        (action.from + 2, action.from + 1)
    } else {
        (action.from - 2, action.from - 1)
    };

    let king = BitBoard::index(action.from);
    let rook = BitBoard::index(action.to);
    let king_relocated = BitBoard::index(relocated_king);
    let rook_relocated = BitBoard::index(relocated_rook);

    board.state.pieces[action.piece_type] = board.state.pieces[action.piece_type].xor(king).or(king_relocated);
    board.state.pieces[rook_ind] = board.state.pieces[rook_ind].xor(rook).or(rook_relocated);
    
    if board.state.moving_team == Team::White {
        board.state.white = board.state.white.xor(king).xor(rook).or(king_relocated).or(rook_relocated);
    } else {
        board.state.black = board.state.black.xor(king).xor(rook).or(king_relocated).or(rook_relocated);
    }

    HistoryState(updates)
}

pub struct KingProcess;

impl<T : PrimInt + Unsigned> PieceProcessor<T> for KingProcess {
    fn process(&self, board: &mut Board<T>, piece_index: usize) {
        let edges = board.edges[0];
        board.lookup[piece_index] = vec![ vec![ ] ];

        for index in 0..64 {
            let king = BitBoard::index(index);

            let up = king.and_not(edges.top).up(1);
            let down = king.and_not(edges.bottom).down(1);
        
            let vertical = king.or(up).or(down);
        
            let right = vertical.and_not(edges.right).right(1);
            let left = vertical.and_not(edges.left).left(1);
        
            let moves = vertical.or(right).or(left).and_not(king);
            board.lookup[piece_index][0].push(moves);
        }
    }

    fn capture_mask(&self, board: &mut Board<T>, piece_index: usize, mask: BitBoard<T>) -> BitBoard<T> {
        let mut mask = BitBoard::empty();
        let moving_team = board.state.team_to_move();
        for king in board.state.pieces[piece_index].and(moving_team).iter() {
            mask = mask.or(board.lookup[piece_index][0][king as usize]);
        }
        mask
    }

    fn list_actions(&self, board: &mut Board<T>, piece_index: usize) -> Vec<Action> {
        let moving_team = board.state.team_to_move();
        let mut actions: Vec<Action> = Vec::with_capacity(8);

        for king in board.state.pieces[piece_index].and(moving_team).iter() {
            let pos = king as usize;
            let moves = board.lookup[piece_index][0][king as usize].and_not(moving_team);
            for movement in moves.iter() {
                actions.push(Action::from(pos, movement as usize, piece_index))
            }

            // Castling: Rook is required

            let king_in_place = board.state.first_move.and(BitBoard::index(king as usize)).is_set();
            if !king_in_place { continue; }

            let rook_ind = board.find_piece("rook");

            if let Some(rook_ind) = rook_ind {
                for rook in board.state.pieces[rook_ind].and(moving_team).and(board.state.first_move).iter() {
                    let between_squares = BitBoard::between(king as usize, rook as usize);
                    
                    // Can't castle if other pieces are in the way.
                    if between_squares.and(board.state.black.or(board.state.white)).is_set() {
                        continue;
                    }
                    
                    let king_dest = if rook > king { king + 2 } else { king - 2 };
                    let between_dest_squares = BitBoard::between_inclusive(king as usize, king_dest as usize);

                    // We'll need the capture mask of the opp team
                    board.state.moving_team = board.state.moving_team.next();
                    let mask = board.list_captures(between_dest_squares);
                    board.state.moving_team = board.state.moving_team.next();


                    // We can't castle through check or while in check, so we'll have to check if that's the case.
                    if between_dest_squares.or(BitBoard::index(king as usize)).and(mask).is_set() {
                        continue;
                    }

                    // We can castle! This move is represented as king goes to where the rook is.
                    actions.push(Action::from(pos, rook as usize, piece_index));
                }
            }
        }
    
        actions
    }

    fn display_action(&self, board: &mut Board<T>, action: Action) -> Vec<String> {
        let display = format!("{}{}", index_to_square(action.from), index_to_square(action.to));
        if BitBoard::index(action.to).and(board.state.team_to_move()).is_set() {
            let king_dest = if action.to > action.from { action.from + 2 } else { action.from - 2 };
            let alternate_display = format!("{}{}", index_to_square(action.from), index_to_square(king_dest));

            // King cannot move two tiles left or right, meaning this must be a castling move
            vec![
                display,
                alternate_display
            ]
        } else {
            vec![
                display
            ]
        }
    }

    fn display_uci_action(&self, board: &mut Board<T>, action: Action) -> String {
        if BitBoard::index(action.to).and(board.state.team_to_move()).is_set() {
            let king_dest = if action.to > action.from { action.from + 2 } else { action.from - 2 };
            let alternate_display = format!("{}{}", index_to_square(action.from), index_to_square(king_dest));

            // King cannot move two tiles left or right, meaning this must be a castling move
            alternate_display
        } else {
            let display = format!("{}{}", index_to_square(action.from), index_to_square(action.to));
            display
        }
    }

    fn make_move(&self, board: &mut Board<T>, action: Action) -> HistoryState<T> {
        if BitBoard::index(action.to).and(board.state.team_to_move()).is_set() {
            // King cannot move two tiles left or right, meaning this must be a castling move
            make_castling_move(board, action)
        } else {
            make_chess_move(board, action)
        }
    }
}

pub fn create_king<T : PrimInt + Unsigned>() -> Piece<T> {
    Piece::new("k", "king", Box::new(KingProcess))
}