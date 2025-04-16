use arrayvec::ArrayVec;
use rustc_hash::FxHashMap as HashMap;

use action::{ActionRecord, Action};
use piece::Piece;
use zobrist::ZobristTable;

use crate::bitboard::{BitBoard, BitInt, Bounds, Edges};

pub mod piece;
pub mod action;
pub mod perft;
pub mod suite;
pub mod zobrist;

pub type AttackDirections<T > = Vec<BitBoard<T>>;
/// AttackLookup is indexed by the index of the Most Significant 1-Bit.
///
/// It stores an `AttackDirections` (alias for `Vec<BitBoard>`).
///     For pieces that always move the same way (like Delta Pieces), only the first slot of this AttackDirections is used, because there's no directions.
///     For slider pieces, there are different indexes for specific ray directions of it.

pub type AttackLookup<T > = Vec<AttackDirections<T>>;

/// Indexed by the piece type; find a piece's attack lookups.
pub type PieceLookup<T > = Vec<AttackLookup<T>>;

pub struct Game<T : BitInt, const N: usize> {
    pub rules: Box<dyn GameRules<T, N>>,
    pub pieces: Vec<Piece<T, N>>,
    pub edges: Vec<Edges<T>>,
    pub bounds: Bounds,
    pub default_pos: String,
    pub lookup: PieceLookup<T>
}

impl<T : BitInt, const N: usize> Game<T, N> {
    pub fn init(&self) -> Board<T, N> {
        Board::new(self)
    }

    pub fn default(&self) -> Board<T, N> {
        self.load(&self.default_pos)
    }

    pub fn load(&self, pos: &str) -> Board<T, N> {
        let mut board = self.init();
        board.load(pos);
        board
    }
}

pub trait GameTemplate {
    fn create<T : BitInt, const N: usize>() -> Game<T, N>;
}

#[derive(Copy, Clone, Debug)]
pub enum GameState {
    Win(Team),
    Draw,
    Ongoing
}

/// `GameRules` handles managing game specific processing.
pub trait GameRules<T : BitInt, const N: usize> {
    fn load(&self, board: &mut Board<T, N>, pos: &str);
    fn save(&self, board: &mut Board<T, N>) -> String;

    fn is_legal(&self, board: &mut Board<T, N>) -> bool;
    fn game_state(&self, board: &mut Board<T, N>, legal_actions: &[Action]) -> GameState;
    fn gen_zobrist(&self, board: &mut Board<T, N>, seed: u64) -> ZobristTable;
    fn hash(&self, board: &mut Board<T, N>, table: &ZobristTable) -> u64;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Team {
    White,
    Black
}

impl Team {
    pub fn next(self) -> Team {
        match self {
            Team::White => Team::Black,
            Team::Black => Team::White
        }
    }

    pub fn index(self) -> usize {
        match self {
            Team::White => 0,
            Team::Black => 1
        }
    }
}

#[derive(Clone)]
pub struct Board<'a, T : BitInt, const N: usize> {
    pub game: &'a Game<T, N>,
    pub state: BoardState<T, N>,
    pub history: Vec<ActionRecord>
}

#[derive(Clone)]
pub struct BoardState<T : BitInt, const N: usize> {
    pub moving_team: Team,
    pub first_move: BitBoard<T>,
    pub white: BitBoard<T>,
    pub black: BitBoard<T>,
    pub pieces: [ BitBoard<T>; N ]
}

impl<T : BitInt, const N: usize> BoardState<T, N> {
    pub fn new() -> Self {
        Self {
            moving_team: Team::White,
            black: BitBoard::empty(),
            white: BitBoard::empty(),
            first_move: BitBoard::empty(),
            pieces: [ BitBoard::empty(); N ]
        }
    }

    #[inline(always)]
    pub fn team(&self, team: Team) -> BitBoard<T> {
        match team { 
            Team::White => self.white,
            Team::Black => self.black
        }
    }

    #[inline(always)]
    pub fn team_to_move(&self) -> BitBoard<T> {
        self.team(self.moving_team)
    }

    #[inline(always)]
    pub fn opposite_team(&self) -> BitBoard<T> {
        self.team(self.moving_team.next())
    }

    pub fn piece_at(&self, square: u8) -> Option<usize> {
        let at = BitBoard::index(square);
        for piece in 0..self.pieces.len() {
            if self.pieces[piece].and(at).is_set() {
                return Some(piece);
            }
        }
        None
    }

}

impl<'a, T : BitInt, const N: usize> Board<'a, T, N> {
    pub fn new(game: &'a Game<T, N>) -> Self {
        Self {
            game,
            state: BoardState::new(),
            history: vec![]
        }
    }

    pub fn load(&mut self, pos: &str) {
        self.game.rules.load(self, pos);

        for index in 0..self.game.pieces.len() {
            self.game.pieces[index].rules.load(self, index);
        }
    }

    pub fn load_pieces(&mut self, pos: &str) {
        for (y, row) in pos.split("/").enumerate() {
            let y = y as u8;
            let mut x: u8 = 0;

            for char in row.chars() {
                if let Some(skip) = char.to_digit(10) {
                    x += skip as u8;
                    continue;
                }

                let matched_piece = self.game.pieces.iter().enumerate()
                    .find(|el| el.1.symbol.to_lowercase() == char.to_string().to_lowercase());

                if let Some((index, _)) = matched_piece {
                    let piece = BitBoard::coords(x, y, self.game.bounds);
                    let is_black = char.is_lowercase();

                    self.state.first_move = self.state.first_move.or(piece);

                    self.state.pieces[index] = self.state.pieces[index].or(piece);

                    if is_black {
                        self.state.black = self.state.black.or(piece);
                    } else {
                        self.state.white = self.state.white.or(piece);
                    }
                }

                x += 1;
            }
        }
    }

    pub fn list_actions(&mut self) -> Vec<Action> {
        let mut actions: Vec<Action> = Vec::with_capacity(40);
        for piece_type in 0..self.game.pieces.len() {
            let mut piece_actions = self.game.pieces[piece_type].rules.list_actions(self, piece_type);
            actions.append(&mut piece_actions);
        }
        actions
    }
    
    pub fn list_legal_actions(&mut self) -> Vec<Action> {
        let mut actions = vec![];
        for action in self.list_actions() {
            let state = self.play(action);
            let is_legal = self.game.rules.is_legal(self);
            self.restore(state);
            
            if is_legal {
                actions.push(action);
            }
        }
        actions
    }

    pub fn list_captures(&mut self, mask: BitBoard<T>) -> BitBoard<T> {
        let mut captures = BitBoard::empty();
        for piece_type in 0..self.game.pieces.len() {
            let piece_mask = self.game.pieces[piece_type].rules.capture_mask(self, piece_type, mask);
            captures = piece_mask.or(captures);
        }
        captures
    }

    pub fn game_state(&mut self, actions: &[Action]) -> GameState {
        self.game.rules.game_state(self, actions)
    }

    pub fn piece_at(&self, square: u8) -> Option<usize> {
        self.state.piece_at(square)
    }

    pub fn display_action(&mut self, action: Action) -> Vec<String> {
        let piece_index = self.piece_at(action.from).expect("Found displayed piece");
        self.game.pieces[piece_index as usize].rules.display_action(self, action)
    }

    pub fn display_uci_action(&mut self, action: Action) -> String {
        let piece_index = self.piece_at(action.from).expect("Found displayed piece");
        self.game.pieces[piece_index as usize].rules.display_uci_action(self, action)
    }

    pub fn find_action(&mut self, action: &str) -> Action {
        let actions = self.list_actions();
        return actions.iter().find(|el| self.display_action(**el).contains(&action.to_string())).map(|el| *el).expect("Could not find action"); 
    }
    
    pub fn play_action(&mut self, action: &str) -> BoardState<T, N> {
        let action = self.find_action(action);
        self.play(action)
    }

    pub fn play_null(&mut self) -> BoardState<T, N> {
        let state = self.state.clone();

        self.state.moving_team = self.state.moving_team.next();
        self.history.push(ActionRecord::Null());
        state
    }

    pub fn play(&mut self, action: Action) -> BoardState<T, N> {
        let state = self.state.clone();

        let piece_index = self.piece_at(action.from).expect("Found piece making move");
        self.game.pieces[piece_index as usize].rules.make_move(self, action);
        self.state.moving_team = state.moving_team.next();

        self.history.push(ActionRecord::Action(action));
        state
    }

    pub fn restore(&mut self, state: BoardState<T, N>) {
        self.state = state;
        self.history.pop();

    }
}