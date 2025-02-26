use std::sync::Arc;

use pixels::{Pixels, SurfaceTexture};
use rand::Rng;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

const WIDTH: u32 = 99;
const HEIGHT: u32 = 99;

struct GameOfLife {
    grid: Vec<bool>,
    width: usize,
    height: usize,
}

impl GameOfLife {
    /// Create a new game state with all cells dead.
    fn new(width: usize, height: usize) -> Self {
        let grid = vec![false; width * height];
        GameOfLife { grid, width, height }
    }

    /// Randomize the grid.
    fn randomize(&mut self) {
        let mut rng = rand::rng();
        for cell in &mut self.grid {
            *cell = rng.random_bool(0.5);
        }
    }

    /// Update the grid using the standard Game of Life rules.
    fn update(&mut self) {
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


struct Application {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    game: GameOfLife,
}

impl Application {
    fn new() -> Self {
        let game = GameOfLife::new(WIDTH as usize, HEIGHT as usize);
        Self {
            window: None,
            pixels: None,
            game,
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = {
            let size: LogicalSize<f64> = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
            Arc::new(
                event_loop
                .create_window(
                    WindowAttributes::default()
                    .with_title("Life on subpixels")
                    .with_inner_size(size)
                    .with_min_inner_size(size)
                ).unwrap()
            )
        };
        self.window = Some(window.clone());

        self.pixels = {
            let window_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            match Pixels::new(WIDTH, HEIGHT, surface_texture) {
                Ok(pixels) => {
                    window.request_redraw();
                    Some(pixels)
                }
                Err(err) => {
                    event_loop.exit();
                    None
                }
            }
        };

        self.game.randomize();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                let window = self.window.as_ref().expect("redraw request without a window");
                window.pre_present_notify();

                let pixels = self.pixels.as_mut().expect("redraw request without a pixels buffer");

                self.game.update();

                let frame = pixels.frame_mut();
                for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                    let cell = self.game.grid[i];
                    let rgba = if cell {
                        [0xFF, 0xFF, 0xFF, 0xFF] // white pixel
                    } else {
                        [0x00, 0x00, 0x00, 0xFF] // black pixel
                    };
                    pixel.copy_from_slice(&rgba);
                }

                if pixels.render().is_err() {
                    event_loop.exit();
                    return;
                }

                window.request_redraw();
            }
            _ => (),
        }
    }
}

fn main() -> Result<(), impl std::error::Error> {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Application::new();
    event_loop.run_app(&mut app)
}
