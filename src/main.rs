use chessing::{chess::Chess, game::GameTemplate};

pub fn main() {
    let chess = Chess::create::<u64>();
    let mut board = chess.default();

    println!("{}", board.perft(4));
}