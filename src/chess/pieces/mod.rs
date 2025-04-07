pub mod pawn;
pub mod knight;
pub mod sliders;
pub mod king;

use king::{create_king, KingProcess};
use knight::{create_knight, KnightProcess};
use pawn::{create_pawn, PawnProcess};
use sliders::{bishop::{create_bishop, BishopProcess}, queen::{create_queen, QueenProcess}, rook::{create_rook, RookProcess}};

use crate::game::piece::Piece;