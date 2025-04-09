use fancy_regex::{Captures, Regex};

use super::Uci;

#[derive(Debug)]
pub enum UciPosition {
    Startpos,
    Fen(String),
}

#[derive(Debug)]
pub enum GoOption {
    WTime(u64),
    BTime(u64),
    WInc(u64),
    BInc(u64),
    MovesToGo(u64),
    MoveTime(u64), // Added movetime support
}

#[derive(Debug)]
pub enum UciCommand {
    Uci(),
    Position {
        position: UciPosition,
        moves: Vec<String>,
    },
    Quit(),
    Stop(),
    UciNewGame(),
    IsReady(),
    Go {
        options: Vec<GoOption>,
    },
    Unknown(String),
}

fn find_moves(caps: &Captures) -> Vec<String> {
    let moves = caps.name("moves").map(|m| m.as_str());
    let moves = moves
        .map(|moves| moves.split(" ").map(|el| el.to_string()).collect())
        .unwrap_or(vec![]);
    moves
}

fn parse_and_push(
    caps: &Captures,
    name: &str,
    constructor: fn(u64) -> GoOption,
    options: &mut Vec<GoOption>,
) {
    if let Some(m) = caps.name(name) {
        if let Ok(value) = m.as_str().parse::<u64>() {
            options.push(constructor(value));
        }
    }
}

impl Uci {
    pub fn parse(&self, cmd: &str) -> UciCommand {
        let go_regex = Regex::new(
            r"^go(?: (?:(?:(?:wtime (?P<wtime>\d+))|(?:btime (?P<btime>\d+))|(?:winc (?P<winc>\d+))|(?:binc (?P<binc>\d+))|(?:movestogo (?P<movestogo>\d+))|(?:movetime (?P<movetime>\d+))) ?)*)?$"
        ).unwrap();

        let startpos_regex =
            Regex::new(r"position startpos(?: moves (?<moves>.*))?").unwrap();

        let fen_regex = Regex::new(r"position fen (?<fen>.*?)(?=\smoves|$)(?: moves (?<moves>.*))?").unwrap();

        if cmd == "uci" {
            return UciCommand::Uci();
        }

        if cmd == "quit" {
            return UciCommand::Quit();
        }

        if cmd == "stop" {
            return UciCommand::Stop();
        }

        if cmd == "ucinewgame" {
            return UciCommand::UciNewGame();
        }

        if cmd == "isready" {
            return UciCommand::IsReady();
        }

        if let Ok(Some(caps)) = startpos_regex.captures(cmd) {
            return UciCommand::Position {
                position: UciPosition::Startpos,
                moves: find_moves(&caps),
            };
        }

        if let Ok(Some(caps)) = fen_regex.captures(cmd) {
            let fen = caps["fen"].to_string();
            return UciCommand::Position {
                position: UciPosition::Fen(fen),
                moves: find_moves(&caps),
            };
        }

        if let Ok(Some(caps)) = go_regex.captures(&cmd) {
            let mut options: Vec<GoOption> = vec![];

            parse_and_push(&caps, "wtime", GoOption::WTime, &mut options);
            parse_and_push(&caps, "btime", GoOption::BTime, &mut options);
            parse_and_push(&caps, "winc", GoOption::WInc, &mut options);
            parse_and_push(&caps, "binc", GoOption::BInc, &mut options);
            parse_and_push(&caps, "movestogo", GoOption::MovesToGo, &mut options);
            parse_and_push(&caps, "movetime", GoOption::MoveTime, &mut options); // <-- Added

            return UciCommand::Go { options };
        }

        UciCommand::Unknown(cmd.to_string())
    }
}
