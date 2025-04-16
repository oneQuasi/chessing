use crate::bitboard::BitInt;

use super::{Board, Team};

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