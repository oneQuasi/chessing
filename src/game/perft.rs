
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

    pub fn perft_debug(&mut self, depth: usize) -> usize {
        if depth == 0 { return 1; }
    
        let actions = self.actions();
        let mut lines: Vec<String> = vec![];
    
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

            lines.push(format!("{} - {}", self.display_uci_action(action), sub_nodes));

            nodes += sub_nodes;
        }

        lines.sort();

        for line in lines {
            println!("{}", line);
        }

        nodes
    }
}