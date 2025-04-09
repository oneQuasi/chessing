use rustc_hash::FxHashMap as HashMap;

use action::{Action, HistoryState, HistoryUpdate};
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

pub struct Board<'a, T : BitInt> {
    pub game: &'a Game<T>,
    pub state: BoardState<T>,
    pub piece_map: HashMap<String, usize>,
    /// A cache of important piece indexes guaranteed by the game processor.
    pub required_pieces: Vec<usize>,
    pub edges: Vec<Edges<T>>,
    pub lookup: PieceLookup<T>
}

pub struct Game<T : BitInt> {
    pub processor: Box<dyn GameProcessor<T>>,
    pub pieces: Vec<Piece<T>>,
    pub bounds: Bounds,
    pub default_pos: String
}

impl<T : BitInt> Game<T> {
    pub fn init(&self) -> Board<T> {
        Board::new(self)
    }

    pub fn default(&self) -> Board<T> {
        self.load(&self.default_pos)
    }

    pub fn load(&self, pos: &str) -> Board<T> {
        let mut board = self.init();
        board.load(pos);
        board
    }
}

pub trait GameTemplate {
    fn create<T : BitInt>() -> Game<T>;
}

#[derive(Copy, Clone, Debug)]
pub enum GameState {
    Win(Team),
    Draw,
    Ongoing
}

/// `GameProcessor` handles managing game specific processing.
pub trait GameProcessor<T : BitInt> {
    fn load(&self, board: &mut Board<T>, pos: &str);
    fn is_legal(&self, board: &mut Board<T>) -> bool;
    fn game_state(&self, board: &mut Board<T>, legal_actions: &[Action]) -> GameState;
    fn gen_zobrist(&self, board: &mut Board<T>) -> ZobristTable;
    fn hash(&self, board: &mut Board<T>, table: &ZobristTable) -> u64;
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

#[derive(Debug, Clone)]
pub struct BoardState<T : BitInt> {
    pub moving_team: Team,

    // Pieces which haven't moved are set as `first_move`

    pub first_move: BitBoard<T>,

    // Only two player games are supported.

    pub white: BitBoard<T>,
    pub black: BitBoard<T>,
    pub pieces: Vec<BitBoard<T>>,

    pub mailbox: Vec<u8>,

    pub history: Vec<Action>
}

impl<T : BitInt> BoardState<T> {
    pub fn new() -> Self {
        Self {
            moving_team: Team::White,
            black: BitBoard::empty(),
            white: BitBoard::empty(),
            first_move: BitBoard::empty(),
            pieces: vec![BitBoard::empty(); 6],
            history: vec![],
            mailbox: vec![]
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

    pub fn restore(&mut self, state: HistoryState<T>) {
        self.moving_team = self.moving_team.next();
        self.history.pop();
        for change in state.0 {
            match change {
                HistoryUpdate::Mailbox(index, value) => {
                    self.mailbox[index as usize] = value;
                }
                HistoryUpdate::Piece(index, board) => {
                    self.pieces[index as usize] = board;
                }
                HistoryUpdate::White(board) => {
                    self.white = board;
                }
                HistoryUpdate::Black(board) => {
                    self.black = board;
                }
                HistoryUpdate::FirstMove(board) => {
                    self.first_move = board;
                }
            }
        }
    }
}

impl<'a, T : BitInt> Board<'a, T> {
    pub fn new(game: &'a Game<T>) -> Self {
        Self {
            game,
            state: BoardState::new(),
            piece_map: HashMap::default(),
            required_pieces: vec![],
            edges: vec![
                BitBoard::edges(game.bounds, 1),
                BitBoard::edges(game.bounds, 2)
            ],
            lookup: vec![ vec![]; 8 ]
        }
    }

    /// Since variants are supported as a first-class feature, the index of a given piece type might not be fixed.
    /// `piece_map` allows for easy access of other pieces based on names, avoiding conflicts.
    #[inline(always)]
    pub fn find_piece(&self, name: &str) -> Option<usize> {
        self.piece_map.get(name).copied()
    }

    pub fn load(&mut self, pos: &str) {
        for _ in 0..(self.game.bounds.rows * self.game.bounds.cols) {
            self.state.mailbox.push(0);
        }

        for (index, piece) in self.game.pieces.iter().enumerate() {
            self.piece_map.insert(piece.name.clone(), index);
        }

        self.game.processor.load(self, pos);

        for index in 0..self.game.pieces.len() {
            self.game.pieces[index].processor.process(self, index);
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
                    self.state.mailbox[piece.bitscan_forward() as usize] = index as u8 + 1;

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
            let mut piece_actions = self.game.pieces[piece_type].processor.list_actions(self, piece_type);
            actions.append(&mut piece_actions);
        }
        actions
    }
    
    pub fn list_legal_actions(&mut self) -> Vec<Action> {
        let mut actions = vec![];
        for action in self.list_actions() {
            let history = self.play(action);
            let is_legal = self.game.processor.is_legal(self);
            self.state.restore(history);
            
            if is_legal {
                actions.push(action);
            }
        }
        actions
    }

    pub fn list_captures(&mut self, mask: BitBoard<T>) -> BitBoard<T> {
        let mut captures = BitBoard::empty();
        for piece_type in 0..self.game.pieces.len() {
            let piece_mask = self.game.pieces[piece_type].processor.capture_mask(self, piece_type, mask);
            captures = piece_mask.or(captures);
        }
        captures
    }

    pub fn game_state(&mut self, actions: &[Action]) -> GameState {
        self.game.processor.game_state(self, actions)
    }

    pub fn display_action(&mut self, action: Action) -> Vec<String> {
        let piece_index = self.state.mailbox[action.from as usize] - 1;
        self.game.pieces[piece_index as usize].processor.display_action(self, action)
    }

    pub fn display_uci_action(&mut self, action: Action) -> String {
        let piece_index = self.state.mailbox[action.from as usize] - 1;
        self.game.pieces[piece_index as usize].processor.display_uci_action(self, action)
    }

    pub fn find_action(&mut self, action: &str) -> Action {
        let actions = self.list_actions();
        return actions.iter().find(|el| self.display_action(**el).contains(&action.to_string())).map(|el| *el).expect("Could not find action"); 
    }

    pub fn play_action(&mut self, action: &str) -> HistoryState<T> {
        let act = self.find_action(action);
        self.play(act)
    }
    

    pub fn play(&mut self, action: Action) -> HistoryState<T> {
        let piece_index = self.state.mailbox[action.from as usize] - 1;
        let state = self.game.pieces[piece_index as usize].processor.make_move(self, action);
        self.state.moving_team = self.state.moving_team.next();

        self.state.history.push(action);
        state
    }
}