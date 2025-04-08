

use num::{PrimInt, Unsigned};
use pieces::{king::create_king, knight::create_knight, pawn::create_pawn, sliders::{bishop::create_bishop, queen::create_queen, rook::create_rook}};

use crate::{bitboard::{BitBoard, Bounds}, game::{action::{index_to_square, Action}, piece::{Piece, PieceProcessor}, Board, Game, GameProcessor, GameState, GameTemplate, Team}};

pub mod pieces;
pub mod suite;

pub struct ChessProcessor;

impl<T : PrimInt + Unsigned> GameProcessor<T> for ChessProcessor {
    fn is_legal(&self, board: &mut Board<T>) -> bool {
        let king_ind = board.find_piece("king").expect("King is required for chess");
        let king = board.state.pieces[king_ind].and(board.state.opposite_team());
        let mask = board.list_captures(king);

        mask.and(king).is_empty()
    }

    fn load(&self, board: &mut Board<T>, pos: &str) {
        let parts: Vec<String> = pos.split(" ").map(|el| el.to_string()).collect();

        // Piece Placement
        board.load_pieces(&parts[0]);

        // Team to Move
        board.state.moving_team = if parts[1] == "w" { Team::White } else { Team::Black };

        let left_side = BitBoard::edges_left(board.game.bounds, board.game.bounds.cols / 2);
        let right_side = BitBoard::edges_right(board.game.bounds, board.game.bounds.cols / 2);

        let rook_ind = board.find_piece("rook").expect("Cannot handle castling rights w/o rook");
        
        // Castling Rights
        if !parts[2].contains("K") {
            board.state.first_move = board.state.first_move
                .xor(board.state.pieces[rook_ind].and(board.state.white).and(right_side));
        }

        if !parts[2].contains("Q") {
            board.state.first_move = board.state.first_move
                .xor(board.state.pieces[rook_ind].and(board.state.white).and(left_side));
        }

        if !parts[2].contains("k") {
            board.state.first_move = board.state.first_move
                .xor(board.state.pieces[rook_ind].and(board.state.black).and(right_side));
        }

        if !parts[2].contains("q") {
            board.state.first_move = board.state.first_move
                .xor(board.state.pieces[rook_ind].and(board.state.black).and(left_side));
        }
        
        // En Passant
        if parts[3] != "-" {
            // TODO
            // Allows for loading FENs with en passants, but not needed for active play so I delayed it
        }
    }

    fn game_state(&self, board: &mut Board<T>, actions: &[Action]) -> crate::game::GameState {
        if actions.len() == 0 {
            let king_ind = board.find_piece("king").expect("King is required for chess");
            let king = board.state.pieces[king_ind].and(board.state.team_to_move());
            board.state.moving_team = board.state.moving_team.next();
            let captures = board.list_captures(king);
            board.state.moving_team = board.state.moving_team.next();

            if captures.and(king).is_set() {
                GameState::Win(board.state.moving_team)
            } else {
                GameState::Draw
            }
        } else {
            GameState::Ongoing
        }
    }
}

pub struct Chess;

impl GameTemplate for Chess {
    fn create<T : PrimInt + Unsigned>() -> Game<T> {
        Game {
            processor: Box::new(ChessProcessor),
            pieces: vec![
                create_pawn(),
                create_knight(),
                create_bishop(),
                create_rook(),
                create_queen(),
                create_king()
            ],
            bounds: Bounds::new(8, 8),
            default_pos: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()
        }
    }
}