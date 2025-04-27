
use crate::{bitboard::{BitBoard, BitInt, Edges}, game::{action::{make_chess_move, Action}, Board, Game, MagicEntry}};

use super::{ray_attacks, repeat, slider::{Slider, SliderMoves}};

#[derive(Copy, Clone)]
pub struct Magic<S : SliderMoves>(pub S);

impl <S : SliderMoves> Magic<S> {
    pub fn process<T: BitInt, const N: usize>(&self, game: &mut Game<T, N>, piece_index: usize) {
        let slider = Slider(self.0);
        slider.process(game, piece_index);
        
        // We need raycasting to check if magics are valid

        let edges = game.edges[0];
        game.lookup[piece_index] = vec![];

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

            if index != 0 { continue; }
            
            let entry = MagicEntry {
                mask: relevant_blockers,
                magic: 64,
                index_bits: relevant_blockers.count()
            };

            let mut table = vec![ BitBoard::<T>::default(); (1 << entry.index_bits) as usize ];
            let mut blockers = BitBoard::<T>::default();
            
            
        }
    }

    pub fn attacks<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, piece_index: usize, mask: BitBoard<T>) -> bool {
        false
    }

    pub fn actions<T: BitInt, const N: usize>(&self, board: &mut Board<T, N>, piece_index: usize) -> Vec<Action> {
        vec![]
    }
}