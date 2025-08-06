
use crate::{bitboard::{BitBoard, BitInt, Edges}, game::{action::{make_chess_move, Action}, AttackLookup, Board, Game, MagicEntry, PieceMagics}};

use super::{ray_attacks, repeat, slider::{Slider, SliderMoves}};

fn try_make_table<T : BitInt, S : SliderMoves, const N: usize>(
    game: &mut Game<T, N>, 
    entry: MagicEntry<T>,
    piece_index: usize,
    index: usize
) -> Option<Vec<BitBoard<T>>> {
    let mut table = vec![ BitBoard::<T>::default(); (1 << (64 - entry.shift)) as usize ];
    let mut blockers = BitBoard::<T>::default();
    loop {
        let moves = Slider::<S>::list_moves(game, piece_index, index, blockers);
        let table_entry = &mut table[magic_index(entry, blockers)];
        if table_entry.empty() {
            // Write to empty slot
            *table_entry = moves;
        } else if *table_entry != moves {
            // Having two different move sets in the same slot is a hash collision
            return None;
        }

        blockers.0 = blockers.0.wrapping_sub(&entry.mask.0) & entry.mask.0;
        if blockers.empty() {
            // Finished enumerating all blocker configurations
            break;
        }
    }

    Some(table)
}

#[inline(never)]
fn magic_index<T : BitInt>(entry: MagicEntry<T>, blockers: BitBoard<T>) -> usize {
    let blockers = blockers.and(entry.mask);
    let hash = blockers.0.wrapping_mul(&entry.magic);
    let index = hash >> entry.shift;
    index.to_usize().expect("Must be usize")
}

#[inline(never)]
fn magic_moves<T: BitInt, const N: usize>(
    board: &mut Board<T, N>,
    pos: usize,
    piece_index: usize,
    blockers: BitBoard<T>,
    team: BitBoard<T>,
) -> BitBoard<T> {
    let entry = get_magic_entry(board, piece_index, pos);
    let magic_ind = magic_index(entry, blockers);
    let raw_moves = get_raw_magic_moves(board, piece_index, pos, magic_ind);
    let legal_moves = filter_friendly(raw_moves, team);
    legal_moves
}

#[inline(never)]
fn get_magic_entry<T: BitInt, const N: usize>(
    board: &Board<T, N>,
    piece_index: usize,
    pos: usize,
) -> MagicEntry<T> {
    board.game.magics[piece_index][pos]
}

#[inline(never)]
fn get_raw_magic_moves<T: BitInt, const N: usize>(
    board: &Board<T, N>,
    piece_index: usize,
    pos: usize,
    magic_ind: usize,
) -> BitBoard<T> {
    board.game.lookup[piece_index][pos][magic_ind]
}

#[inline(always)]
fn filter_friendly<T: BitInt>(moves: BitBoard<T>, team: BitBoard<T>) -> BitBoard<T> {
    moves.and_not(team)
}

#[derive(Copy, Clone)]
pub struct Magic<S : SliderMoves>(pub S);

impl <S : SliderMoves> Magic<S> {
    pub fn process<T: BitInt, const N: usize>(&self, game: &mut Game<T, N>, piece_index: usize) {
        let slider = Slider(self.0);
        slider.process(game, piece_index);
        
        // We need raycasting to check if magics are valid

        let edges = game.edges[0];

        let mut lookup: AttackLookup<T> = vec![];
        let mut magics: PieceMagics<T> = vec![];

        for index in 0..64 {
            let pos = BitBoard::index(index);
            let rays = self.0.rays(pos, &edges);
            let mut relevant_blockers = BitBoard::default();

            for ray in rays {
                for edge in [edges.bottom, edges.left, edges.right, edges.top] {
                    if ray.and(edge).count() != 1 { continue; }

                    relevant_blockers = relevant_blockers.or(ray.and_not(edge));
                    break;
                }  
            }
            
            loop {
                let magic = fastrand::u64(..) & fastrand::u64(..) & fastrand::u64(..);

                let entry = MagicEntry {
                    mask: relevant_blockers,
                    magic: T::from(magic).expect("Must work"),
                    shift: 64 - relevant_blockers.count() as usize
                };
    
                let table = try_make_table::<T, S, N>(game, entry, piece_index, index as usize);

                if let Some(table) = table {
                    lookup.push(table);
                    magics.push(entry);

                    break;
                }
            }
        }

        game.lookup[piece_index] = lookup;
        game.magics[piece_index] = magics;

        println!("Done for {}", piece_index);
    }

    pub fn attacks<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, piece_index: usize, mask: BitBoard<T>) -> bool {
        let team = board.state.team_to_move();
        let blockers = board.state.black.or(board.state.white);
    
        board.state.pieces[piece_index]
            .and(team)
            .iter()
            .any(|slider| {
                let moves = magic_moves(board, slider as usize, piece_index, blockers, team);

                moves.and(mask).set()
            })
    }

    #[inline(never)]
    pub fn add_actions<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, actions: &mut Vec<Action>, piece_index: usize) {
        let team = board.state.team_to_move();
        let blockers = board.state.black.or(board.state.white);
        let piece = piece_index as u8;
    
        for slider in board.state.pieces[piece_index].and(team).iter() {
            let moves = magic_moves(board, slider as usize, piece_index, blockers, team);

            for to in moves.iter() {
                actions.push(Action::from(slider as u16, to as u16, piece));
            }
        }
    }
}