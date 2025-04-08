use num::{PrimInt, Unsigned};

use super::Board;

impl<'a, T : PrimInt + Unsigned> Board<'a, T> {
    pub fn perft(&mut self, depth: usize) -> usize {
        if depth == 0 { return 1; }
    
        let actions = self.list_actions();
    
        let mut nodes = 0;
        for action in actions {
            let history = self.play(action);
            let is_legal = self.game.processor.is_legal(self);
    
            if !is_legal {
                self.restore(history);
                continue;
            }
    
            let sub_nodes = self.perft(depth - 1);
            nodes += sub_nodes;
    
            self.restore(history);
        }
        nodes
    }
}