
use crate::bitboard::{BitInt, Bounds};

use super::{Board, Game, Team};

#[inline(always)]
fn get_index<T : BitInt>(board: &Board<T>, team: Team, piece: usize, square: usize) -> usize {
    let teams = 2;
    let team_number: usize = if team == Team::White { 0 } else { 1 };
    
    let pieces = board.game.pieces.len();

    team_number + (piece * teams) + (square * teams * pieces)
}

pub struct ZobristTable {
    pub table: Vec<u64>
}

impl ZobristTable {
    pub fn compute<T : BitInt>(&self, board: &Board<T>) -> u64 {
        let mut hash = 0;
        for piece in 0..board.state.pieces.len() {
            for team in [board.state.moving_team, board.state.moving_team.next()] {
                let piece_team_board = board.state.pieces[piece as usize].and(board.state.team(team));
                for square in piece_team_board.iter() {
                    hash ^= self.table[get_index(board, team, piece, square as usize)];
                }
            }
        }    
        hash   
    }
}

impl<T : BitInt> Game<T> {
    pub fn gen_table(&self, bounds: Bounds) -> ZobristTable {
        let squares = (bounds.rows * bounds.cols) as usize;
        let pieces = self.pieces.len();
        let teams = 2;

        let hashes = squares * pieces * teams;
        let mut table = vec![0; hashes];

        for hash in 0..hashes {
            table[hash] = fastrand::u64(0..u64::MAX);
        }

        ZobristTable { table }
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
            println!("{} - {}", position.pos, zobrist.compute(&board));
        }
    }
}