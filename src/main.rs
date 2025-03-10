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

const WIDTH: u32 = 100;
const HEIGHT: u32 = 100;
const LIFE_DENSITY_DEFAULT: f64 = 0.08;
const SPEED_DEFAULT: u64 = 3;
const SLEEP_TIME_LIST: [u64; 7] = [500, 300, 150, 80, 50, 25, 0];
const SUBPIXELS: bool = true;

struct Application {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    life_density: f64,
    speed: u64,
    game: GameOfLife,
    run: bool,
    request_redraw: bool,
    close_requested: bool,
}

impl Application {
    fn new() -> Self {
        let game = GameOfLife::new(SUBPIXELS, WIDTH as usize, HEIGHT as usize, LIFE_DENSITY_DEFAULT);
        Self {
            window: None,
            pixels: None,
            life_density: LIFE_DENSITY_DEFAULT,
            speed: SPEED_DEFAULT,
            game,
            run: false,
            request_redraw: true,
            close_requested: false,
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
                        false => self.request_redraw = true,
                        _ => (),
                    }
                },
                Key::Named(NamedKey::ArrowUp) => {
                    if self.life_density < 0.5 {
                        self.life_density += 0.01;
                        if self.life_density > 0.5 {
                            self.life_density = 0.5;
                        }
                        self.game.randomize(self.life_density);
                        self.request_redraw = true;
                    }
                },
                Key::Named(NamedKey::ArrowDown) => {
                    if self.life_density > 0.01 {
                        self.life_density -= 0.01;
                        if self.life_density < 0.01 {
                            self.life_density = 0.01;
                        }
                        self.game.randomize(self.life_density);
                        self.request_redraw = true;
                    }
                },
                Key::Named(NamedKey::ArrowRight) => {
                    if self.speed < SLEEP_TIME_LIST.len() as u64 - 1 {
                        self.speed += 1;
                    }
                },
                Key::Named(NamedKey::ArrowLeft) => {
                    if self.speed > 0 {
                        self.speed -= 1;
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

                self.game.update();

                let frame = pixels.frame_mut();
                for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                    let rgba = self.game.pixel_color(i);
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
