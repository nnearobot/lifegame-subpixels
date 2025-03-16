use rand::Rng;

pub struct GameOfLife {
    grid: Vec<bool>,
    width: usize,
    height: usize,
    subpixels: bool,
}

impl GameOfLife {
    /// Create a new game state with all cells dead.
    pub fn new(subpixels: bool, mut width: usize, height: usize, life_density: f64) -> Self {
        if subpixels {
            width *= 3;
        }
        let grid = Self::new_grid(width, height, life_density);
        GameOfLife { subpixels, grid, width, height }
    }

    /// Randomize the grid.
    pub fn randomize(&mut self, life_density: f64) {
        self.grid = Self::new_grid(self.width, self.height, life_density);
    }

    /// Update the grid using the standard Game of Life rules.
    pub fn update(&mut self) {
        self.grid = self.next();
    }

    pub fn pixel_color(&mut self, x: usize, y: usize) -> [u8; 4] {
        match self.subpixels {
            true => {
                let i = self.index(x * 3, y);
                let r = if self.grid[i + 0] { 255 } else { 0 };
                let g = if self.grid[i + 1] { 255 } else { 0 };
                let b = if self.grid[i + 2] { 255 } else { 0 };
                [r, g, b, 255]
            },
            false => {
                let i = self.index(x, y);
                if self.grid[i] {
                    [255, 255, 255, 255]
                } else {
                    [0, 0, 0, 255]
                }
            },
        }
    }

    fn next(&mut self) -> Vec<bool> {
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
        next
    }

    /// Helper to convert 2D coordinates to 1D index.
    fn index(&mut self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Count live neighbors with wrap-around (toroidal grid).
    fn live_neighbor_count(&mut self, x: usize, y: usize) -> u8 {
        let mut count = 0;
        // Check neighbors in a 3x3 block around (x, y)
        for dy in [-1, 0, 1].iter().cloned() {
            for dx in [-1, 0, 1].iter().cloned() {
                // Skip the cell itself
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = x as isize + dx as isize;
                let ny = y as isize + dy as isize;

                // skip the cell that is out of bounds
                if nx < 0 || nx >= self.width as isize {
                    continue;
                }
                if ny < 0 || ny >= self.height as isize {
                    continue;
                }

                let idx = self.index(nx as usize, ny as usize);
                if self.grid[idx] {
                    count += 1;
                }
            }
        }
        count
    }

    fn new_grid(width: usize, height: usize, life_density: f64) -> Vec<bool> {
        let mut grid = vec![false; width * height];
        let mut rng = rand::rng();
        for cell in grid.iter_mut() {
            *cell = rng.random_bool(life_density);
        }
        grid
    }
}

