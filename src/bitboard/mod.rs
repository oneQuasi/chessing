pub mod sizedint;

use num::{PrimInt, Unsigned};

pub trait BitInt:
    PrimInt + Unsigned {}

impl<T> BitInt for T where
    T: PrimInt + Unsigned
{}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitBoard<T: BitInt>(pub T);

#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub rows: u16,
    pub cols: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct Edges<T: BitInt> {
    pub right: BitBoard<T>,
    pub left: BitBoard<T>,
    pub top: BitBoard<T>,
    pub bottom: BitBoard<T>,
    pub all: BitBoard<T>,
}

pub struct BitPositions<T: BitInt>(T);

impl<T: BitInt> BitPositions<T> {
    pub fn new(bits: T) -> Self {
        BitPositions(bits)
    }
}

impl<T: BitInt> Iterator for BitPositions<T> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == T::zero() {
            None
        } else {
            let index = self.0.trailing_zeros();
            self.0 = self.0 & (self.0 - T::one());
            Some(index)
        }
    }
}

impl Bounds {
    pub fn new(rows: u16, cols: u16) -> Bounds {
        Bounds { rows, cols }
    }
}

impl<T: BitInt> BitBoard<T> {
    pub fn or(self, board: BitBoard<T>) -> BitBoard<T> {
        BitBoard(self.0 | board.0)
    }

    pub fn xor(self, board: BitBoard<T>) -> BitBoard<T> {
        BitBoard(self.0 ^ board.0)
    }

    pub fn and(self, board: BitBoard<T>) -> BitBoard<T> {
        BitBoard(self.0 & board.0)
    }

    pub fn and_not(self, board: BitBoard<T>) -> BitBoard<T> {
        self.and(board.not())
    }

    pub fn not(self) -> BitBoard<T> {
        BitBoard(!self.0)
    }

    pub fn gt(self, board: BitBoard<T>) -> bool {
        self.0 > board.0
    }

    pub fn lt(self, board: BitBoard<T>) -> bool {
        self.0 < board.0
    }

    pub fn right(self, tiles: usize) -> BitBoard<T> {
        BitBoard(self.0 << tiles)
    }

    pub fn left(self, tiles: usize) -> BitBoard<T> {
        BitBoard(self.0 >> tiles)
    }

    pub fn up(self, tiles: usize) -> BitBoard<T> {
        BitBoard(self.0 << (8 * tiles))
    }

    pub fn down(self, tiles: usize) -> BitBoard<T> {
        BitBoard(self.0 >> (8 * tiles))
    }

    pub fn display(self, bounds: Bounds) {
        let Bounds { rows, cols } = bounds;
        let total_bits = T::zero().count_zeros(); // We’ll use this to cap max size
        assert!(
            (rows as u32) * (cols as u32) <= total_bits,
            "BitBoard only supports up to {} bits",
            total_bits
        );

        for r in 0..rows {
            for c in 0..cols {
                let index = (rows - 1 - r) * cols + c;
                let bit = (self.0 >> (index as usize)) & T::one();
                print!("{}", if bit == T::one() { "⬜" } else { "⬛" });
            }
            println!();
        }
    }

    pub fn edges_left(bounds: Bounds, depth: u16) -> BitBoard<T> {
        let Bounds { rows, cols } = bounds;
        if rows < 3 || cols < 3 || depth == 0 {
            return BitBoard(T::zero());
        }

        let mut mask = T::zero();
        for row in 0..rows {
            for depth in 0..depth.min(cols) {
                let idx = row * cols + depth;
                mask = mask | (T::one() << (idx as usize));
            }
        }
        BitBoard(mask)
    }

    pub fn edges_right(bounds: Bounds, depth: u16) -> BitBoard<T> {
        let Bounds { rows, cols } = bounds;
        if rows < 3 || cols < 3 || depth == 0 {
            return BitBoard(T::zero());
        }

        let mut mask = T::zero();
        for row in 0..rows {
            for depth in 0..depth.min(cols) {
                let idx = row * cols + (cols - 1 - depth);
                mask = mask | (T::one() << (idx as usize));
            }
        }
        BitBoard(mask)
    }

    pub fn edges_down(bounds: Bounds, depth: u16) -> BitBoard<T> {
        let Bounds { rows, cols } = bounds;
        if rows < 3 || cols < 3 || depth == 0 {
            return BitBoard(T::zero());
        }

        let mut mask = T::zero();
        for col in 0..cols {
            for depth in 0..depth.min(rows) {
                let idx = depth * cols + col;
                mask = mask | (T::one() << (idx as usize));
            }
        }
        BitBoard(mask)
    }

    pub fn edges_up(bounds: Bounds, depth: u16) -> BitBoard<T> {
        let Bounds { rows, cols } = bounds;
        if rows < 3 || cols < 3 || depth == 0 {
            return BitBoard(T::zero());
        }

        let mut mask = T::zero();
        for col in 0..cols {
            for depth in 0..depth.min(rows) {
                let idx = (rows - 1 - depth) * cols + col;
                mask = mask | (T::one() << (idx as usize));
            }
        }
        BitBoard(mask)
    }

    pub fn edges(bounds: Bounds, depth: u16) -> Edges<T> {
        let left = Self::edges_left(bounds, depth);
        let right = Self::edges_right(bounds, depth);
        let top = Self::edges_up(bounds, depth);
        let bottom = Self::edges_down(bounds, depth);
        let all = left.or(right).or(top).or(bottom);

        Edges {
            left,
            right,
            top,
            bottom,
            all,
        }
    }

    pub fn empty() -> BitBoard<T> {
        BitBoard(T::zero())
    }

    pub fn index(index: u16) -> BitBoard<T> {
        BitBoard(T::one() << index.into())
    }

    pub fn count(self) -> u32 {
        self.0.count_ones()
    }

    pub fn between(start: usize, end: usize) -> BitBoard<T> {
        if start > end {
            return Self::between(end, start);
        }

        if end - start <= 1 {
            return BitBoard(T::zero());
        }

        let len = end - start - 1;
        let mask = (T::one() << len) - T::one();
        BitBoard(mask << (start + 1))
    }

    pub fn between_inclusive(start: usize, end: usize) -> BitBoard<T> {
        if start > end {
            return Self::between_inclusive(end, start);
        }

        let len = end - start + 1;
        let mask = (T::one() << len) - T::one();
        BitBoard(mask << start)
    }

    pub fn coords(x: u16, y: u16, bounds: Bounds) -> BitBoard<T> {
        assert!(x < bounds.cols, "x coordinate out of bounds");
        assert!(y < bounds.rows, "y coordinate out of bounds");

        // Calculate the index of the bit
        let index = (bounds.rows - 1 - y) * bounds.cols + x;

        Self::index(index)
    }

    pub fn iter(self) -> BitPositions<T> {
        BitPositions::new(self.0)
    }

    pub fn is_set(self) -> bool {
        self.0 != T::zero()
    }

    pub fn is_empty(self) -> bool {
        self.0 == T::zero()
    }

    pub fn bitscan_forward(self) -> u32 {
        assert!(self.is_set(), "BitBoard must be set to BitScan");
        self.0.trailing_zeros()
    }

    pub fn bitscan_backward(self) -> u32 {
        assert!(self.is_set(), "BitBoard must be set to BitScan");
        T::zero().count_zeros() - 1 - self.0.leading_zeros()
    }
}
