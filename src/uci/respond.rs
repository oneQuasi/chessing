use super::Uci;
use std::fmt::Write;

#[derive(Default)]
pub struct Info {
    pub depth: Option<u32>,
    pub seldepth: Option<u32>,
    pub multipv: Option<u32>,
    pub score_cp: Option<i32>,
    pub score_mate: Option<i32>,
    pub hashfull: Option<u32>,
    pub time: Option<u64>,
    pub nodes: Option<u64>,
    pub nps: Option<u64>,
    pub pv: Option<Vec<String>>,
}

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

    pub fn info(&self, info: Info) {
        let mut output = String::from("info");

        if let Some(depth) = info.depth {
            write!(output, " depth {}", depth).unwrap();
        }
        if let Some(seldepth) = info.seldepth {
            write!(output, " seldepth {}", seldepth).unwrap();
        }
        if let Some(multipv) = info.multipv {
            write!(output, " multipv {}", multipv).unwrap();
        }
        if let Some(score_cp) = info.score_cp {
            write!(output, " score cp {}", score_cp).unwrap();
        }
        if let Some(score_mate) = info.score_mate {
            write!(output, " score mate {}", score_mate).unwrap();
        }
        if let Some(hashfull) = info.hashfull {
            write!(output, " hashfull {}", hashfull).unwrap();
        }
        if let Some(time) = info.time {
            write!(output, " time {}", time).unwrap();
        }
        if let Some(nodes) = info.nodes {
            write!(output, " nodes {}", nodes).unwrap();
        }
        if let Some(nps) = info.nps {
            write!(output, " nps {}", nps).unwrap();
        }
        if let Some(pv) = &info.pv {
            write!(output, " pv {}", pv.join(" ")).unwrap();
        }

        println!("{}", output);
    }
}