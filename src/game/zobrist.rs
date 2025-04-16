use crate::bitboard::BitInt;

use super::{Board, Team};

#[inline(always)]
fn get_index<T: BitInt, const N: usize>(board: &Board<T, N>, team: Team, piece: usize, square: usize) -> usize {
    let pieces = board.game.pieces.len();
    let squares = (board.game.bounds.cols * board.game.bounds.rows) as usize;
    
    let team_offset = match team {
        Team::White => 0,
        Team::Black => 1,
    };

    // Index formula: (team * pieces + piece) * squares + square
    (team_offset * pieces + piece) * squares + square
}

pub struct ZobristTable {
    pub table: Vec<u64>
}

impl ZobristTable {
    pub fn generate(hashes: usize, seed: u64) -> ZobristTable {
        fastrand::seed(seed);
        let mut table = vec![0; hashes];

        for hash in 0..hashes {
            table[hash] = fastrand::u64(0..u64::MAX);
        }

        ZobristTable { table }
    }

    pub fn compute(&self, attrs: &[usize]) -> u64 {
        let mut hash = 0;
        for attr in attrs {
            hash ^= self.table[*attr];
        }
        hash   
    }
}

#[cfg(test)]
mod tests {
    
}