

use rustc_hash::FxHashMap as HashMap;

use pieces::{king::{create_king, KingRules}, knight::{create_knight, KnightRules}, pawn::{self, create_pawn, PawnRules}, sliders::{bishop::{create_bishop, BishopRules}, queen::{create_queen, QueenRules}, rook::{create_rook, RookRules}}};

use crate::{bitboard::{BitBoard, BitInt, Bounds}, game::{action::{index_to_square, square_to_index, Action, ActionRecord}, piece::PieceRules, zobrist::ZobristTable, Board, Game, GameRules, GameState, GameTemplate, Team}};

pub mod pieces;
pub mod suite;
mod test_positions;

// For other variants, these constants will need to be redefined. Oops!

pub const PAWN: usize = 0;
pub const KNIGHT: usize = 1;
pub const BISHOP: usize = 2;
pub const ROOK: usize = 3;
pub const QUEEN: usize = 4;
pub const KING: usize = 5;

struct CastlingRights {
    white_king_side: bool,
    white_queen_side: bool,
    black_king_side: bool,
    black_queen_side: bool,
}

impl CastlingRights {
    fn index(&self) -> usize {
        let mut castling_ind = 0;

        if self.white_king_side {
            castling_ind += 8;
        }

        if self.white_queen_side {
            castling_ind += 4;
        }

        if self.black_king_side {
            castling_ind += 2;
        }

        if self.black_queen_side {
            castling_ind += 1;
        }

        castling_ind
    }
}

fn extract_castling_rights<T : BitInt, const N: usize>(board: &Board<T, N>) -> CastlingRights {
    let left_side = BitBoard::edges_left(board.game.bounds, board.game.bounds.cols / 2);
    let right_side = BitBoard::edges_right(board.game.bounds, board.game.bounds.cols / 2);

    let unmoved_rooks = board.state.first_move.and(board.state.pieces[ROOK]);

    CastlingRights {
        white_king_side: unmoved_rooks
            .and(board.state.white)
            .and(right_side)
            .is_set(),

        white_queen_side: unmoved_rooks
            .and(board.state.white)
            .and(left_side)
            .is_set(),

        black_king_side: unmoved_rooks
            .and(board.state.black)
            .and(right_side)
            .is_set(),

        black_queen_side: unmoved_rooks
            .and(board.state.black)
            .and(left_side)
            .is_set(),
    }
}

pub struct ChessProcessor;

impl<T : BitInt, const N: usize> GameRules<T, N> for ChessProcessor {
    fn list_actions(&self, board: &mut Board<T, N>) -> Vec<Action> {
        let mut actions = Vec::with_capacity(40); // Pre-allocate as you did

        actions.extend(PawnRules.list_actions(board, 0));
        actions.extend(KnightRules.list_actions(board, 1));
        actions.extend(BishopRules.list_actions(board, 2));
        actions.extend(RookRules.list_actions(board, 3));
        actions.extend(QueenRules.list_actions(board, 4));
        actions.extend(KingRules.list_actions(board, 5));

        actions
    }
    
    fn list_captures(&self, board: &mut Board<T, N>, mask: BitBoard<T>) -> BitBoard<T> {
        let mut captures = BitBoard::empty();

        captures = captures.or(PawnRules.capture_mask(board, 0, mask));
        captures = captures.or(KnightRules.capture_mask(board, 1, mask));
        captures = captures.or(BishopRules.capture_mask(board, 2, mask));
        captures = captures.or(RookRules.capture_mask(board, 3, mask));
        captures = captures.or(QueenRules.capture_mask(board, 4, mask));
        captures = captures.or(KingRules.capture_mask(board, 5, mask));

        captures
    }

    fn play(&self, board: &mut Board<T, N>, act: Action) {
        let piece_index = board.piece_at(act.from).expect("Found piece making move");

        match piece_index {
            0 => PawnRules.make_move(board, act),
            1 => KnightRules.make_move(board, act),
            2 => BishopRules.make_move(board, act),
            3 => RookRules.make_move(board, act),
            4 => QueenRules.make_move(board, act),
            5 => KingRules.make_move(board, act),
            _ => unreachable!(),
        }
    }

    fn is_legal(&self, board: &mut Board<T, N>) -> bool {
        let king = board.state.pieces[KING].and(board.state.opposite_team());
        let mask = board.list_captures(king);

        mask.and(king).is_empty()
    }

    fn load(&self, board: &mut Board<T, N>, pos: &str) {
        let parts: Vec<String> = pos.split(" ").map(|el| el.to_string()).collect();

        let mut piece_map: HashMap<char, usize> = HashMap::default();
        piece_map.insert('p', 0);
        piece_map.insert('n', 1);
        piece_map.insert('b', 2);
        piece_map.insert('r', 3);
        piece_map.insert('q', 4);
        piece_map.insert('k', 5);

        // Piece Placement
        board.load_pieces(&parts[0], piece_map);

        // Team to Move
        board.state.moving_team = if parts[1] == "w" { Team::White } else { Team::Black };

        let left_side = BitBoard::edges_left(board.game.bounds, board.game.bounds.cols / 2);
        let right_side = BitBoard::edges_right(board.game.bounds, board.game.bounds.cols / 2);

        // If a `Game` is constructed using `ChessProcessor`, we won't know the order of `king` and `rook`.
        // Therefore, we can't be sure those pieces are at any given index.
        // `required_pieces` allows us to save the index of them for use in `is_legal` so that we don't need to deal with hashes.
        
        // Castling Rights
        if !parts[2].contains("K") {
            board.state.first_move = board.state.first_move
                .xor(board.state.pieces[ROOK].and(board.state.white).and(right_side));
        }

        if !parts[2].contains("Q") {
            board.state.first_move = board.state.first_move
                .xor(board.state.pieces[ROOK].and(board.state.white).and(left_side));
        }

        if !parts[2].contains("k") {
            board.state.first_move = board.state.first_move
                .xor(board.state.pieces[ROOK].and(board.state.black).and(right_side));
        }

        if !parts[2].contains("q") {
            board.state.first_move = board.state.first_move
                .xor(board.state.pieces[ROOK].and(board.state.black).and(left_side));
        }
        
        // En Passant
        if let Some(en_passant) = square_to_index(&parts[3]) {
            let width = board.game.bounds.cols;
            let one_back = match board.state.moving_team.next() {
                Team::White => en_passant - width, // down 1
                Team::Black => en_passant + width // up 1
            };
            let one_forward = match board.state.moving_team.next() {
                Team::White => en_passant + width, // up 1
                Team::Black => en_passant - width // down 1
            };

            board.history.push(ActionRecord::Action(Action::from(one_back, one_forward, PAWN as u8).with_info(1)));
        }

        PawnRules.load(board, 0);
        KnightRules.load(board, 1);
        BishopRules.load(board, 2);
        RookRules.load(board, 3);
        QueenRules.load(board, 4);
        KingRules.load(board, 5);
    }

    fn save(&self, board: &mut Board<T, N>) -> String {
        // 1. Piece Placement
        let mut piece_rows = Vec::new();
        for row in (0..board.game.bounds.rows).rev() {
            let mut row_str = String::new();
            let mut empty_count = 0;
    
            for col in 0..board.game.bounds.cols {
                let idx = row * board.game.bounds.cols + col;
                let mut found = false;
    
                for (piece_index, piece_board) in board.state.pieces.iter().enumerate() {
                    if piece_board.and(BitBoard::index(idx)).is_set() {
                        let team = if board.state.white.and(BitBoard::index(idx)).is_set() {
                            Team::White
                        } else {
                            Team::Black
                        };
    
                        if empty_count > 0 {
                            row_str.push_str(&empty_count.to_string());
                            empty_count = 0;
                        }
    
                        let piece_char = match piece_index {
                            0 => "p",
                            1 => "n",
                            2 => "b",
                            3 => "r",
                            4 => "q",
                            5 => "k",
                            _ => unreachable!()
                        };
                        row_str.push_str(&match team {
                            Team::White => piece_char.to_ascii_uppercase(),
                            Team::Black => piece_char.to_ascii_lowercase(),
                        });
    
                        found = true;
                        break;
                    }
                }
    
                if !found {
                    empty_count += 1;
                }
            }
    
            if empty_count > 0 {
                row_str.push_str(&empty_count.to_string());
            }
    
            piece_rows.push(row_str);
        }
    
        let piece_placement = piece_rows.join("/");
    
        // 2. Active Color
        let active_color = match board.state.moving_team {
            Team::White => "w",
            Team::Black => "b",
        };
    
        // 3. Castling Availability
        let left_side = BitBoard::edges_left(board.game.bounds, board.game.bounds.cols / 2);
        let right_side = BitBoard::edges_right(board.game.bounds, board.game.bounds.cols / 2);
    
        let mut castling = String::new();
    
        if board.state.first_move.and(board.state.white.and(right_side)).or(board.state.pieces[ROOK].and(board.state.white).and(right_side)).is_empty() == false {
            castling.push('K');
        }
        if board.state.first_move.and(board.state.white.and(left_side)).or(board.state.pieces[ROOK].and(board.state.white).and(left_side)).is_empty() == false {
            castling.push('Q');
        }
        if board.state.first_move.and(board.state.black.and(right_side)).or(board.state.pieces[ROOK].and(board.state.black).and(right_side)).is_empty() == false {
            castling.push('k');
        }
        if board.state.first_move.and(board.state.black.and(left_side)).or(board.state.pieces[ROOK].and(board.state.black).and(left_side)).is_empty() == false {
            castling.push('q');
        }
    
        if castling.is_empty() {
            castling.push('-');
        }
    
        // 4. En Passant
        let mut en_passant = "-".to_string();
    
        if let Some(ActionRecord::Action(last_move)) = board.history.last() {
            let last_piece_index = board.state.piece_at(last_move.to).expect("Found en passant piece");
            let was_pawn_move = last_piece_index == PAWN;
    
            if was_pawn_move {
                let diff = last_move.to.abs_diff(last_move.from);
                if diff == board.game.bounds.cols * 2 {
                    let square = match board.state.moving_team.next() {
                        Team::White => last_move.to - board.game.bounds.cols,
                        Team::Black => last_move.to + board.game.bounds.cols,
                    };
                    en_passant = index_to_square(square);
                }
            }
        }
    
        format!("{} {} {} {} 0 1", piece_placement, active_color, castling, en_passant)
    }

    fn game_state(&self, board: &mut Board<T, N>, actions: &[Action]) -> crate::game::GameState {
        if actions.len() == 0 {
            let king = board.state.pieces[KING].and(board.state.team_to_move());
            board.state.moving_team = board.state.moving_team.next();
            let captures = board.list_captures(king);
            board.state.moving_team = board.state.moving_team.next();

            if captures.and(king).is_set() {
                GameState::Win(board.state.moving_team.next())
            } else {
                GameState::Draw
            }
        } else {
            GameState::Ongoing
        }
    }

    fn gen_zobrist(&self, board: &mut Board<T, N>, seed: u64) -> ZobristTable {
        let pieces = N;
        let squares = (board.game.bounds.rows * board.game.bounds.cols) as usize;
        let teams = 2;

        let piece_features = pieces * squares * teams;
        let team_to_move_features = teams;
        let castling_features = 16;
        let en_passant_features = (2 * squares) + 1;

        ZobristTable::generate(
            piece_features + team_to_move_features + castling_features + en_passant_features,
            seed
        )
    }

    fn hash(&self, board: &mut Board<T, N>, table: &ZobristTable) -> u64 {
        let mut attrs = Vec::with_capacity(30);

        let pieces = N;
        let squares = (board.game.bounds.rows * board.game.bounds.cols) as usize;

        let mut features = 0;

        for team in [Team::White, Team::Black] {
            let team_index = team.index();
            for piece in 0..N {
                let piece_team_board = board.state.pieces[piece as usize].and(board.state.team(team));
                for square in piece_team_board.iter() {
                    attrs.push(
                        (square as usize) + (piece * squares) + (team_index * pieces * squares)
                    );
                }
            }
        }

        features += squares * pieces * 2;

        let team_to_move = match board.state.moving_team { Team::White => 0, Team::Black => 1 };
        attrs.push(team_to_move + features);

        features += 2;

        let castling_rights = extract_castling_rights(board);
        let castling_index = castling_rights.index();
        attrs.push(castling_index + features);

        features += 16;

        let mut en_passant = false;

        if let Some(ActionRecord::Action(last_move)) = board.history.last() {
            let last_piece_index = board.state.piece_at(last_move.to);
            if let Some(last_piece_index) = last_piece_index {
                let was_pawn_move = last_piece_index == PAWN;
        
                if was_pawn_move {
                    let was_double_move = last_move.to.abs_diff(last_move.from) == 16;
                    if was_double_move {
                        en_passant = true;
                        let team_index = board.state.moving_team.index();
                        attrs.push((last_move.to as usize) + (squares * team_index) + features);
                    }
                }
            }
        }

        if !en_passant {
            attrs.push(features + (squares * 2));
        }

        table.compute(&attrs)
    }
}

pub struct Chess;

impl GameTemplate for Chess {
    fn create<T : BitInt, const N: usize>() -> Game<T, N> {
        let bounds = Bounds::new(8, 8);
        let mut game = Game {
            rules: Box::new(ChessProcessor),
            bounds,
            default_pos: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            lookup: vec![ vec![]; 6 ],
            edges: vec![
                BitBoard::edges(bounds, 1),
                BitBoard::edges(bounds, 2)
            ],
        };

        let pieces = vec![
            create_pawn(),
            create_knight(),
            create_bishop(),
            create_rook(),
            create_queen(),
            create_king()
        ];

        let mut index = 0;
        for piece in pieces {
            piece.rules.process(&mut game, index);
            index += 1;
        }

        game
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::{chess::Chess, game::{suite::{parse_suite, test_suite}, GameTemplate}};

    use super::{suite::CHESS_SUITE, test_positions::TEST_POSITIONS};

    #[test]
    fn chess_zobrist() {
        let chess = Chess::create::<u64, 6>();
        let mut board = chess.default();

        let table = chess.rules.gen_zobrist(&mut board, 64);
        let mut hashes = HashMap::new();

        let mut collisions = 0;

        for position in TEST_POSITIONS.split("\n") {
            board = chess.load(&position);
            let hash = chess.rules.hash(&mut board, &table);

            if hashes.contains_key(&hash) {
                println!("COLLISION! {}", hash);
                println!("- {}", position);
                println!("- {}", hashes.get(&hash).expect("Value found"));
                collisions += 1;
            } else {
                hashes.insert(hash, position);
            }
        }

        println!("{} collisions ({} positions)", collisions, hashes.len());
        assert_eq!(collisions, 0);
    }

    #[test]
    fn chess_fens() {
        let chess = Chess::create::<u64, 6>();

        let mut positions = HashSet::<String>::new();
        let mut collisions = 0;

        for position in TEST_POSITIONS.split("\n") {
            let mut board = chess.load(&position);
            let out = board.game.rules.save(&mut board);

            if positions.contains(&out) {
                collisions += 1;
            }

            positions.insert(out);
        }

        println!("{} collisions", collisions);
        assert_eq!(collisions, 0);
    }
}