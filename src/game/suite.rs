use std::time::{SystemTime, UNIX_EPOCH};

fn current_time_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
}

use num::{PrimInt, Unsigned};

use super::{Board, Game};

#[derive(Debug)]
pub struct Position {
    pub pos: String,
    pub nodes: Vec<u32>
}

pub fn parse_suite(positions: &str) -> Vec<Position> {
    let mut out = vec![];
    for position in positions.split("\n") {
        let parts: Vec<_> = position.split(";").collect();
        let (pos, nodes) = parts.split_first().unwrap();

        let nodes: Vec<_> = nodes.iter().map(|el| el.parse::<u32>().expect("Must be a number!")).collect();
        out.push(Position { pos: pos.to_string(), nodes })
    }

    out
}

pub fn test_suite<'a, T : PrimInt + Unsigned>(positions: &str, game: &Game<T>) {
    let positions = parse_suite(positions);
    let mut total_nodes = 0;

    let full_start = current_time_millis();

    for (pos_ind, position) in positions.iter().enumerate() {
        let mut board = game.load(&position.pos);

        for (index, nodes) in position.nodes.iter().enumerate() {
            let depth = index + 1;

            let start = current_time_millis();
            let found_nodes = board.perft(depth);
            let end = current_time_millis();

            total_nodes += found_nodes;

            let mut time = (end - start) as usize;
            if time == 0 { time = 1 };

            let nps = found_nodes / time * 1000;

            println!("[#{}] {} ({} depth) - {} found - {} expected ({} nps)", pos_ind + 1, position.pos, depth, found_nodes, nodes, nps);
            
            assert_eq!(found_nodes, *nodes as usize, "{} found - {} expected", found_nodes, nodes);
        }
    }

    let full_end = current_time_millis();
    let time = (full_end - full_start) as usize;
    let nps = total_nodes / time * 1000;

    println!("Total nodes: {} ({} nps)", total_nodes, nps);

}

