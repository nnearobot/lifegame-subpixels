use std::sync::Arc;
use std::thread;
use std::time::Duration;

use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{WindowEvent, KeyEvent, ElementState};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

mod game_of_life;
use game_of_life::GameOfLife;

const WIDTH: u32 = 50; // 50 min
const HEIGHT: u32 = 50; // 50 min
const LIFE_DENSITY_DEFAULT: f64 = 0.08; // 8%
const SPEED_DEFAULT: u64 = 0;
const SUBPIXELS: bool = true;

const SLEEP_TIME_LIST: [u64; 7] = [500, 300, 150, 80, 50, 25, 0];
const LIFE_DENSITY_MIN: f64 = 0.02;
const LIFE_DENSITY_MAX: f64 = 0.50;
const LIFE_DENSITY_STEP: f64 = 0.02;

const BORDER: u32 = 80;
const INDICATOR: u32 = 3;

struct Application {
    window: Option<Arc<Window>>,
    window_size: LogicalSize<u32>,
    border: u32,
    indicator: u32,
    pixels: Option<Pixels<'static>>,
    life_density: f64,
    speed: u64,
    game: GameOfLife,
    game_size: LogicalSize<u32>,
    run: bool,
    request_redraw: bool,
    close_requested: bool,
}

impl Application {
    fn new() -> Self {
        let width = WIDTH.max(50);
        let height = HEIGHT.max(50);
        let game = GameOfLife::new(SUBPIXELS, width as usize, height as usize, LIFE_DENSITY_DEFAULT);
        let game_size: LogicalSize<u32> = LogicalSize::new(width, height);
        let window_size: LogicalSize<u32> = LogicalSize::new(width + 2 * BORDER, height + 2 * BORDER + 2 * (INDICATOR * 2));
        Self {
            window: None,
            window_size,
            border: BORDER,
            indicator: INDICATOR,
            pixels: None,
            life_density: LIFE_DENSITY_DEFAULT,
            speed: SPEED_DEFAULT,
            game,
            game_size,
            run: false,
            request_redraw: true,
            close_requested: false,
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = {
            Arc::new(
                event_loop
                .create_window(
                    WindowAttributes::default()
                    .with_title("Life on subpixels")
                    .with_inner_size(self.window_size)
                    .with_min_inner_size(self.window_size)
                ).unwrap()
            )
        };
        self.window = Some(window.clone());

        self.pixels = {
            let window_size = window.inner_size();
            let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, window.clone());
            match Pixels::new(self.window_size.width, self.window_size.height, surface_texture) {
                Ok(pixels) => {
                    window.request_redraw();
                    Some(pixels)
                }
                Err(_err) => {
                    self.close_requested = true;
                    None
                }
            }
        };
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            },
            WindowEvent::KeyboardInput {
                event: KeyEvent { logical_key: key, state: ElementState::Pressed, .. },
                ..
            } => match key.as_ref() {
                Key::Character("r") => {
                    self.game.randomize(self.life_density);
                    self.request_redraw = true;
                },
                Key::Named(NamedKey::Enter) => {
                    self.run = !self.run;
                },
                Key::Named(NamedKey::Space) => {
                    match self.run {
                        false => {
                            self.game.update();
                            self.request_redraw = true
                        },
                        _ => (),
                    }
                },
                Key::Named(NamedKey::ArrowUp) => {
                    if self.life_density < LIFE_DENSITY_MAX {
                        self.life_density += LIFE_DENSITY_STEP;
                        if self.life_density > LIFE_DENSITY_MAX {
                            self.life_density = LIFE_DENSITY_MAX;
                        }
                        self.game.randomize(self.life_density);
                        self.request_redraw = true;
                    }
                },
                Key::Named(NamedKey::ArrowDown) => {
                    if self.life_density > LIFE_DENSITY_MIN {
                        self.life_density -= LIFE_DENSITY_STEP;
                        if self.life_density < LIFE_DENSITY_MIN {
                            self.life_density = LIFE_DENSITY_MIN;
                        }
                        self.game.randomize(self.life_density);
                        self.request_redraw = true;
                    }
                },
                Key::Named(NamedKey::ArrowRight) => {
                    if self.speed < SLEEP_TIME_LIST.len() as u64 - 1 {
                        self.speed += 1;
                        if !self.run {
                            self.request_redraw = true;
                        }
                    }
                },
                Key::Named(NamedKey::ArrowLeft) => {
                    if self.speed > 0 {
                        self.speed -= 1;
                        if !self.run {
                            self.request_redraw = true;
                        }
                    }
                },
                Key::Named(NamedKey::Escape) => {
                    self.close_requested = true;
                },
                _ => (),
            },
            WindowEvent::RedrawRequested => {
                let window = self.window.as_ref().expect("redraw request without a window");
                window.pre_present_notify();

                let pixels = self.pixels.as_mut().expect("redraw request without a pixels buffer");

                if self.run {
                    self.game.update();
                }

                let frame = pixels.frame_mut();
                for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                    let x = (i as u32) % self.window_size.width;
                    let y = (i as u32) / self.window_size.width;
                    let control_panel = 2 * self.indicator * 2;
                    let mut rgba: [u8; 4] = [0, 0, 0, 255]; // default color

                    // borders always black
                    if x < self.border || x >= self.border + self.game_size.width as u32 {
                        rgba = [0, 0, 0, 255];
                    } else if y < self.border || y >= self.border + control_panel + self.game_size.height as u32 {
                        rgba = [0, 0, 0, 255];

                    // init life density indicator
                    } else if y < self.border + self.indicator {
                        rgba = [255, 255, 255, 255];
                        let life_density_width = (self.game_size.width as f64 / LIFE_DENSITY_MAX  as f64 * self.life_density) as u32;
                        if x < self.border + life_density_width {
                            rgba = [0, 255, 255, 255];
                        }

                    // current speed indicator
                    } else if y >= self.border + self.indicator * 2 && y < self.border + self.indicator * 2 + self.indicator {
                        rgba = [255, 255, 255, 255];
                        let speed_width = (self.game_size.width as f64 / SLEEP_TIME_LIST.len() as f64 * (self.speed as f64 + 1.0)) as u32;
                        if x < self.border + speed_width {
                            rgba = [255, 0, 255, 255];
                        }

                    // game grid
                    } else if y >= self.border + control_panel {
                        let game_x = x - self.border;
                        let game_y = y - self.border - control_panel;
                        rgba = self.game.pixel_color(game_x as usize, game_y as usize);
                    }

                    pixel.copy_from_slice(&rgba);
                }

                if pixels.render().is_err() {
                    event_loop.exit();
                    return;
                }
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.run && !self.close_requested {
            thread::sleep(Duration::from_millis(SLEEP_TIME_LIST[self.speed as usize]));
            self.window.as_ref().unwrap().request_redraw();
        }

        match self.run {
            true => event_loop.set_control_flow(ControlFlow::Poll),
            false => event_loop.set_control_flow(ControlFlow::Wait),
        };

        if !self.run && self.request_redraw {
            self.window.as_ref().unwrap().request_redraw();
            self.request_redraw = false;
        }

        if self.close_requested {
            event_loop.exit();
        }
    }

}

fn main() -> Result<(), impl std::error::Error> {
    let event_loop = EventLoop::new().unwrap();

    let mut app = Application::new();
    event_loop.run_app(&mut app)
}
