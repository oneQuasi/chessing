use crate::{bitboard::{BitBoard, BitInt, Edges}, chess::ROOK, game::{action::{index_to_square, make_chess_move, Action}, Board, BoardState, Game, Team}};

use super::leaper::LeaperMoves;

pub fn make_castling_move<T: BitInt, const N: usize>(state: &mut BoardState<T, N>, action: Action) {
    let piece_index = action.piece as usize;
    let rook_ind = state.piece_at(action.to).expect("Rook must exist in castling move");

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

    state.pieces[piece_index as usize] = state.pieces[piece_index as usize].xor(king).or(king_relocated);
    state.pieces[rook_ind as usize] = state.pieces[rook_ind as usize].xor(rook).or(rook_relocated);

    match state.moving_team {
        Team::White => {
            state.white = state.white.xor(king).xor(rook).or(king_relocated).or(rook_relocated);
        }
        Team::Black => {
            state.black = state.black.xor(king).xor(rook).or(king_relocated).or(rook_relocated)
        }
    }
}

pub fn castling_actions<T: BitInt, const N: usize>(board: &mut Board<T, N>, piece_index: usize) -> Vec<Action> {
    let moving_team = board.state.team_to_move();
    let mut castles: Vec<Action> = Vec::with_capacity(2);
    let piece = piece_index as u8;

    for king in board.state.pieces[piece_index].and(moving_team).and(board.state.first_move).iter() {
        let pos = king as u16;

        for rook in board.state.pieces[ROOK].and(moving_team).and(board.state.first_move).iter() {
            let between_squares = BitBoard::between(king as usize, rook as usize);
            
            // Can't castle if other pieces are in the way.
            if between_squares.and(board.state.black.or(board.state.white)).set() {
                continue;
            }
            
            let king_dest = if rook > king { king + 2 } else { king - 2 };
            let between_dest_squares = BitBoard::between_inclusive(king as usize, king_dest as usize);

            // We'll need the capture mask of the opp team
            board.state.moving_team = board.state.moving_team.next();
            let mask = board.attacks(between_dest_squares);
            board.state.moving_team = board.state.moving_team.next();


            // We can't castle through check or while in check, so we'll have to check if that's the case.
            if between_dest_squares.or(BitBoard::index(pos)).and(mask).set() {
                continue;
            }

            // We can castle! This move is represented as king goes to where the rook is.
            castles.push(Action::from(pos, rook as u16, piece).with_info(1));
        }
    }

    castles
}

pub struct KingMoves;

impl LeaperMoves for KingMoves {
    fn leaps<T: BitInt>(&self, pos: BitBoard<T>, edges: &Edges<T>) -> BitBoard<T> {
        let up = pos.try_up(&edges, 1);
        let down = pos.try_down(&edges, 1);
    
        let vertical = pos.or(up).or(down);
    
        let right = vertical.try_right(&edges, 1);
        let left = vertical.try_left(&edges, 1);
    
        let moves = vertical.or(right).or(left).and_not(pos);
        moves   
    }
}