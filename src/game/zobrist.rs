
use rand::Rng;

use crate::bitboard::{BitBoard, BitInt, Bounds};

use super::{Board, Game, Team};

#[inline(always)]
fn get_index<T: BitInt>(board: &Board<T>, team: Team, piece: usize, square: usize, first_move: bool) -> usize {
    let pieces = board.game.pieces.len();
    let squares = (board.game.bounds.cols * board.game.bounds.rows) as usize;

    let team_offset = match team {
        Team::White => 0,
        Team::Black => 1,
    };

    // Index formula: (((team * pieces + piece) * squares) + square) * 2 + first_move
    (((team_offset * pieces + piece) * squares) + square) * 2 + first_move as usize
}

pub struct ZobristTable {
    pub pieces: Vec<u64>,
    pub teams: Vec<u64>
}

impl ZobristTable {
    pub fn compute<T : BitInt>(&self, board: &Board<T>) -> u64 {
        let team_index = match board.state.moving_team {
            Team::White => 0,
            Team::Black => 1
        };
        let mut hash = self.teams[team_index];
        for piece in 0..board.state.pieces.len() {
            for team in [Team::White, Team::Black] {
                let piece_team_board = board.state.pieces[piece as usize].and(board.state.team(team));
                for square in piece_team_board.iter() {
                    let first_move = board.state.first_move.and(BitBoard::index(square as u8)).is_set();
                    hash ^= self.pieces[get_index(board, team, piece, square as usize, first_move)];
                }
            }
        }    
        hash   
    }
}

impl<T : BitInt> Game<T> {
    pub fn gen_table(&self, bounds: Bounds) -> ZobristTable {
        let mut rng = rand::rng();
        let squares = (bounds.rows * bounds.cols) as usize;
        let pieces = self.pieces.len();
        let teams = 2;
        let has_first_move = 2;

        let hashes = squares * pieces * teams * has_first_move;
        let mut pieces = vec![0; hashes];
        let mut teams = vec![0; 2];

        for hash in 0..hashes {
            pieces[hash] = rng.random();
        }

        for hash in 0..2 {
            teams[hash] = rng.random();
        }

        ZobristTable { pieces, teams }
    }
}

#[cfg(test)]
mod tests {
    use crate::{bitboard::Bounds, chess::{suite::CHESS_SUITE, Chess}, game::{suite::parse_suite, GameTemplate}};

    #[test]
    fn zobrist() {
        let chess = Chess::create::<u64>();
        let bounds = Bounds::new(8, 8);
        let zobrist = chess.gen_table(bounds);

        let positions = parse_suite(CHESS_SUITE);

        for position in positions {
            let board = chess.load(&position.pos);
        }
    }
}