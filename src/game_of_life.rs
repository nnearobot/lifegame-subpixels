use rand::Rng;

pub struct GameOfLife {
    grid: Vec<bool>,
    width: usize,
    height: usize,
}

impl GameOfLife {
    /// Create a new game state with all cells dead.
    pub fn new(width: usize, height: usize) -> Self {
        let grid = vec![false; width * height];
        GameOfLife { grid, width, height }
    }

    /// Randomize the grid.
    pub fn randomize(&mut self) {
        let mut rng = rand::rng();
        for cell in &mut self.grid {
            *cell = rng.random_bool(0.5);
        }
    }

    /// Update the grid using the standard Game of Life rules.
    pub fn update(&mut self) {
        let mut next = self.grid.clone();
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = self.index(x, y);
                let live_neighbors = self.live_neighbor_count(x, y);
                next[idx] = match (self.grid[idx], live_neighbors) {
                    (true, 2) | (true, 3) => true, // Stays alive
                    (false, 3) => true,            // Becomes alive
                    _ => false,                    // Otherwise, dead
                }
            }
        }
        self.grid = next;
    }

    pub fn pixel_color(&self, i: usize) -> [u8; 4] {
        if self.grid[i] {
            [0xFF, 0xFF, 0xFF, 0xFF] // white pixel
        } else {
            [0x00, 0x00, 0x00, 0xFF] // black pixel
        }
    }

    /// Helper to convert 2D coordinates to 1D index.
    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Count live neighbors with wrap-around (toroidal grid).
    fn live_neighbor_count(&self, x: usize, y: usize) -> u8 {
        let mut count = 0;
        // Check neighbors in a 3x3 block around (x, y)
        for dy in [-1, 0, 1].iter().cloned() {
            for dx in [-1, 0, 1].iter().cloned() {
                if dx == 0 && dy == 0 {
                    continue; // Skip the cell itself
                }
                let nx = (x as isize + dx + self.width as isize) % self.width as isize;
                let ny = (y as isize + dy + self.height as isize) % self.height as isize;
                let idx = self.index(nx as usize, ny as usize);
                if self.grid[idx] {
                    count += 1;
                }
            }
        }
        count
    }
}
