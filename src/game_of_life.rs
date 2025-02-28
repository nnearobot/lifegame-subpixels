use rand::Rng;

pub struct GameOfLife {
    grid: Vec<bool>,
    width: usize,
    height: usize,
}

impl GameOfLife {
    /// Create a new game state with all cells dead.
    pub fn new(width: usize, height: usize) -> Self {
        let grid = new_grid(width, height);
        GameOfLife { grid, width, height }
    }

    /// Randomize the grid.
    pub fn randomize(&mut self) {
        self.grid = new_grid(self.width, self.height);
    }

    /// Update the grid using the standard Game of Life rules.
    pub fn update(&mut self) {
        self.grid = next(&self.grid, self.width, self.height);
    }

    pub fn pixel_color(&self, i: usize) -> [u8; 4] {
        if self.grid[i] {
            [0, 255, 255, 255] // white pixel
        } else {
            [0, 0, 0, 255] // black pixel
        }
    }
}


pub struct GameOfLifeSubpixels {
    grid: Vec<bool>,
    width: usize,
    height: usize,
}

impl GameOfLifeSubpixels {
    /// Create a new game state with all cells dead.
    pub fn new(width: usize, height: usize) -> Self {
        let grid = new_grid(width * 3, height);
        GameOfLifeSubpixels { grid, width: width * 3, height }
    }

    /// Randomize the grid.
    pub fn randomize(&mut self) {
        self.grid = new_grid(self.width, self.height);
    }

    /// Update the grid using the standard Game of Life rules.
    pub fn update(&mut self) {
        self.grid = next(&self.grid, self.width, self.height);
    }

    pub fn pixel_color(&self, i: usize) -> [u8; 4] {
        let r = if self.grid[3 * i + 0] { 255 } else { 0 };
        let g = if self.grid[3 * i + 1] { 255 } else { 0 };
        let b = if self.grid[3 * i + 2] { 255 } else { 0 };
        [r, g, b, 255]
    }
}


fn next(grid: &Vec<bool>, width: usize, height: usize) -> Vec<bool> {
    let mut next = grid.clone();
    for y in 0..height {
        for x in 0..width {
            let idx = index(width, x, y);
            let live_neighbors = live_neighbor_count(grid, width, height, x, y);
            next[idx] = match (grid[idx], live_neighbors) {
                (true, 2) | (true, 3) => true, // Stays alive
                (false, 3) => true,            // Becomes alive
                _ => false,                    // Otherwise, dead
            }
        }
    }
    next
}

/// Helper to convert 2D coordinates to 1D index.
fn index(width: usize, x: usize, y: usize) -> usize {
    y * width + x
}

/// Count live neighbors with wrap-around (toroidal grid).
fn live_neighbor_count(grid: &Vec<bool>, width: usize, height: usize, x: usize, y: usize) -> u8 {
    let mut count = 0;
    // Check neighbors in a 3x3 block around (x, y)
    for dy in [-1, 0, 1].iter().cloned() {
        for dx in [-1, 0, 1].iter().cloned() {
            if dx == 0 && dy == 0 {
                continue; // Skip the cell itself
            }
            let nx = (x as isize + dx + width as isize) % width as isize;
            let ny = (y as isize + dy + height as isize) % height as isize;
            let idx = index(width, nx as usize, ny as usize);
            if grid[idx] {
                count += 1;
            }
        }
    }
    count
}

/// Randomize the grid.
pub fn new_grid(width: usize, height: usize) -> Vec<bool> {
    let mut grid = vec![false; width * height];
    let mut rng = rand::rng();
    for cell in grid.iter_mut() {
        *cell = rng.random_bool(0.5);
    }
    grid
}
