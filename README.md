# `chessing`

## Overview

`chessing` is a move generation library designed to generate moves of chess-compatible games.

Games currently implemented:
- [Chess](https://en.wikipedia.org/wiki/Chess)

Games to be implemented:
- [FRC Chess](https://en.wikipedia.org/wiki/Fischer_random_chess)
- [Alternative Chess Results (AlphaZero)](https://arxiv.org/abs/2009.04374)
    - No-castling
    - No-castling (10)
    - Pawn One Square
    - Stalemate = Win
    - Torpedo
    - Semi-Torpedo
    - Pawn-Back
    - Pawn-Sideways
    - Self-Capture
- [Shogi](https://en.wikipedia.org/wiki/Shogi)
- [Ataxx](https://en.wikipedia.org/wiki/Ataxx)

Additionally, other chess-compatible games can be implemented within this framework by implementing `GameRules` and `PieceRules`.

## Quickstart

```rs
// Start Position
let chess = Chess::create::<u64>();
let mut board = chess.default();

// Perft 
assert_eq!(board.perft(5), 4865609);  

// Generate Actions
for action in board.list_actions() {
    let history = board.play(action);
    let is_legal = board.game.processor.is_legal(&mut board);
    board.restore(history);

    println!("{}", board.display_uci_action(action));
}
```

## Goals

`chessing` supports chess-compatible games, which with respect to this project means:
- Zero sum games (games with two opposing players).
- Games that can be represented as pieces that can act on squares.

`chessing` does not aim to support every game, but to support a subset of games related to chess, and allow for them to be implemented far more easily.

`chessing` isn't meant to be the fastest for move generation, but should still perform decently. If you only want to support Chess and FRC Chess, without caring for other variants, use [cozy-chess](https://github.com/analog-hors/cozy-chess), which is about five times faster in my testing.

`chessing` supports arbitrarily sized boards. You can define your own type compatible implementing `num::PrimInt` and `num::Unsigned`, or you can use the builtin `SizedInt` type.

## Implementation

Implementing a Game requires processing distinct logic for pieces and games.

### PieceRules

```rs
pub trait PieceRules<T: BitInt, const N: usize> {
    fn process(&self, board: &mut Board<T, N>, piece_index: usize) {}
    
    fn list_actions(&self, board: &mut Board<T, N>, piece_index: usize) -> Vec<Action>;
    fn make_move(&self, board: &mut Board<T, N>, action: Action);

    /// Only useful for chess; allows us to optimize checks
    fn capture_mask(&self, board: &mut Board<T, N>, piece_index: usize, mask: BitBoard<T>) -> BitBoard<T> {
        BitBoard::empty()
    }
}
```

`PieceRules` is how you can define pieces and piece behaviors.

- `process` is called after a board is setup, and provides a chance to cache piece moves or otherwise process the board.
- `list_actions` lists actions that can be made with the piece.
- `make_move` defines how the board changes when you make the move, and returns a `HistoryState` to restore changed BitBoards.
- `capture_mask` allows for efficiently testing if a piece can see `mask` without needing to generate a list of actions.

Some pieces require other piece types to generate specific moves. For instance, kings depend on rooks for castling moves. To handle this, kings check if a rook piece is in the game before attempting to see if they can castle like so:

```rs
let rook_ind = board.find_piece("rook");
```

### GameRules

```rs
pub trait GameRules<T: BitInt, const N: usize> {
    fn is_legal(&self, board: &mut Board<T>) -> bool;
    fn load(&self, board: &mut Board<T, N>, pos: &str);

    fn game_state(&self, board: &mut Board<T, N>, legal_actions: &[Action]) -> GameState;
}
```

`GameRules` is how you define full game behaviors.

- `is_legal` checks if a board position after a move is made is legal. For instance in Chess, a position is illegal if after a side makes a move, that team's king is under attack.
- `load` allows for constructing board positions from a string, say a FEN in chess.
- `game_state` determines whether a game is winning for a team, drawn, or ongoing. This does not handle repetitions.