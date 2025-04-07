/// If changed to `u64`, traditional chess is sped up by ~40%, but supporting variants is very important to this project.
pub type IntType = u128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitBoard(pub IntType);

#[derive(Clone, Copy, Debug)]
pub struct Bounds {
    pub rows: u16,
    pub cols: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct Edges {
    pub right: BitBoard,
    pub left: BitBoard,
    pub top: BitBoard,
    pub bottom: BitBoard,
    pub all: BitBoard
}

pub struct BitPositions(IntType);

impl BitPositions {
    pub fn new(bits: IntType) -> Self {
        BitPositions(bits)
    }
}

impl Iterator for BitPositions {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            None
        } else {
            let index = self.0.trailing_zeros();
            self.0 &= self.0 - 1; // Clear the lowest set bit
            Some(index)
        }
    }
}

impl Bounds {
    pub fn new(rows: u16, cols: u16) -> Bounds {
        Bounds { rows, cols }
    }
}

impl BitBoard {
    pub fn or(self, board: BitBoard) -> BitBoard {
        BitBoard(self.0 | board.0)
    }

    pub fn xor(self, board: BitBoard) -> BitBoard {
        BitBoard(self.0 ^ board.0)
    }

    pub fn and(self, board: BitBoard) -> BitBoard {
        BitBoard(self.0 & board.0)
    }

    pub fn and_not(self, board: BitBoard) -> BitBoard {
        self.and(board.not())
    }

    pub fn not(self) -> BitBoard {
        BitBoard(!self.0)
    }

    pub fn gt(self, board: BitBoard) -> bool {
        self.0 > board.0
    }

    pub fn lt(self, board: BitBoard) -> bool {
        self.0 < board.0
    }

    pub fn right(self, tiles: u16) -> BitBoard {
        BitBoard(self.0 << tiles)
    }

    pub fn left(self, tiles: u16) -> BitBoard {
        BitBoard(self.0 >> tiles)
    }

    pub fn up(self, tiles: u16) -> BitBoard {
        BitBoard(self.0 << 8 * tiles)
    }

    pub fn down(self, tiles: u16) -> BitBoard {
        BitBoard(self.0 >> 8 * tiles)
    }

    pub fn display(self, bounds: Bounds) {
        let Bounds { rows, cols } = bounds;
        assert!(rows * cols <= 128, "BitBoard only supports up to 128 bits");

        for r in 0..rows {
            for c in 0..cols {
                let index = (rows - 1 - r) * cols + c; // Flip vertically for correct display
                let bit = (self.0 >> index) & 1;
                print!("{}", if bit == 1 { "⬜" } else { "⬛" });
            }
            println!(); // Newline after each row
        }
    }

    pub fn edges_left(bounds: Bounds, depth: u16) -> BitBoard {
        let Bounds { rows, cols } = bounds;
        if rows < 3 || cols < 3 || depth == 0 {
            return BitBoard(0);
        }
    
        let mut mask: IntType = 0;
        for row in 0..rows {
            for d in 0..depth.min(cols as u16) {
                let idx = row as u32 * cols as u32 + d as u32;
                mask |= (1 as IntType) << idx;
            }
        }
        BitBoard(mask)
    }
    
    pub fn edges_right(bounds: Bounds, depth: u16) -> BitBoard {
        let Bounds { rows, cols } = bounds;
        if rows < 3 || cols < 3 || depth == 0 {
            return BitBoard(0);
        }
    
        let mut mask: IntType = 0;
        for row in 0..rows {
            for d in 0..depth.min(cols as u16) {
                let idx = row as u32 * cols as u32 + (cols as u32 - 1 - d as u32);
                mask |= (1 as IntType) << idx;
            }
        }
        BitBoard(mask)
    }
    
    pub fn edges_down(bounds: Bounds, depth: u16) -> BitBoard {
        let Bounds { rows, cols } = bounds;
        if rows < 3 || cols < 3 || depth == 0 {
            return BitBoard(0);
        }
    
        let mut mask: IntType = 0;
        for col in 0..cols {
            for d in 0..depth.min(rows as u16) {
                let idx = (d as u32) * cols as u32 + col as u32;
                mask |= (1 as IntType) << idx;
            }
        }
        BitBoard(mask)
    }
    
    pub fn edges_up(bounds: Bounds, depth: u16) -> BitBoard {
        let Bounds { rows, cols } = bounds;
        if rows < 3 || cols < 3 || depth == 0 {
            return BitBoard(0);
        }
    
        let mut mask: IntType = 0;
        for col in 0..cols {
            for d in 0..depth.min(rows as u16) {
                let idx = (rows as u32 - 1 - d as u32) * cols as u32 + col as u32;
                mask |= (1 as IntType) << idx;
            }
        }
        BitBoard(mask)
    }

    pub fn edges(bounds: Bounds, depth: u16) -> Edges {
        let left = BitBoard::edges_left(bounds, depth);
        let right = BitBoard::edges_right(bounds, depth);
        let top = BitBoard::edges_up(bounds, depth);
        let bottom = BitBoard::edges_down(bounds, depth);
        let all = left.or(right).or(top).or(bottom);

        Edges {
            left,
            right,
            top,
            bottom,
            all
        }
    }

    pub fn empty() -> BitBoard {
        BitBoard(0)
    }

    pub fn index(index: u16) -> BitBoard {
        BitBoard((1 as IntType) << index)
    }

    pub fn count(self) -> u32 {
        self.0.count_ones()
    }

    pub fn between(start: u16, end: u16) -> BitBoard {
        if start > end {
            return BitBoard::between(end, start);
        }
    
        if end - start <= 1 {
            return BitBoard(0); // No bits between
        }
    
        let len = end - start - 1;
        let mask = ((1 as IntType) << len) - 1;
        BitBoard(mask << (start + 1))
    }

    pub fn between_inclusive(start: u16, end: u16) -> BitBoard {
        if start > end {
            return BitBoard::between_inclusive(end, start);
        }
    
        let len = end - start + 1;
        let mask = ((1 as IntType) << len) - 1;
    
        BitBoard(mask << start)
    }

    pub fn coords(x: u16, y: u16, bounds: Bounds) -> BitBoard {
        // Assert that the coordinates are within bounds
        assert!(x < bounds.cols, "x coordinate out of bounds");
        assert!(y < bounds.rows, "y coordinate out of bounds");

        // Calculate the index of the bit
        let index = (bounds.rows - 1 - y) * bounds.cols + x; // Flip y-axis for correct indexing

        BitBoard::index(index)
    }

    pub fn iter(self) -> BitPositions {
        BitPositions::new(self.0)
    }

    pub fn is_set(self) -> bool {
        self.0 > 0
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn bitscan_forward(self) -> u32 {
        assert!(self.is_set(), "BitBoard must be set to BitScan");
    
        // Find the position of the first set bit using `trailing_zeros()`
        let index = self.0.trailing_zeros(); // This gives the position of the first set bit in u32
        index  // Return the result as a u16 (cast to u16)
    }

    pub fn bitscan_backward(self) -> u32 {
        assert!(self.is_set(), "BitBoard must be set to BitScan");
    
        // Find the position of the first set bit using `trailing_zeros()`
        let index = self.0.leading_zeros(); // This gives the position of the first set bit in u32
        IntType::BITS - 1 - index  // Return the result as a u16 (cast to u16)
    }
}
