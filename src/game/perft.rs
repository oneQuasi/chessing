
use crate::bitboard::BitInt;

use super::Board;

impl<'a, T : BitInt, const N: usize> Board<'a, T, N> {
    pub fn perft(&mut self, depth: usize) -> usize {
        if depth == 0 { return 1; }
    
        let actions = self.actions();
    
        let mut nodes = 0;
        for action in actions {
            let state = self.play(action);
            let is_legal = self.game.rules.is_legal(self);
    
            if !is_legal {
                self.restore(state);
                continue;
            }
    
            let sub_nodes = self.perft(depth - 1);
            self.restore(state);

            nodes += sub_nodes;
        }
        nodes
    }
}