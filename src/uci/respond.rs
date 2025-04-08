use super::Uci;

impl Uci {
    pub fn id_author(&self, author: &str) {
        println!("id author {}", author);
    }

    pub fn id_name(&self, name: &str) {
        println!("id name {}", name);
    }

    pub fn uciok(&self) {
        println!("uciok");
    }

    pub fn readyok(&self) {
        println!("readyok");
    }

    pub fn bestmove(&self, act: &str) {
        println!("bestmove {}", act);
    }
}