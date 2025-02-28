use std::sync::Arc;

use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};


mod game_of_life;
use game_of_life::GameOfLifeSubpixels;

const WIDTH: u32 = 100;
const HEIGHT: u32 = 100;

struct Application {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    game: GameOfLifeSubpixels,
}

impl Application {
    fn new() -> Self {
        let game = GameOfLifeSubpixels::new(WIDTH as usize, HEIGHT as usize);
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
                Err(_err) => {
                    event_loop.exit();
                    None
                }
            }
        };
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
                    let rgba = self.game.pixel_color(i);
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
