
use crate::bitboard::BitInt;

use super::Board;

impl<'a, T : BitInt> Board<'a, T> {
    pub fn perft(&mut self, depth: usize) -> usize {
        if depth == 0 { return 1; }
    
        let actions = self.list_actions();
    
        let mut nodes = 0;
        for action in actions {
            let mut board = self.play(action);
            let is_legal = board.game.processor.is_legal(&mut board);
    
            if !is_legal {
                continue;
            }
    
            let sub_nodes = board.perft(depth - 1);
            nodes += sub_nodes;
        }
        nodes
    }
}