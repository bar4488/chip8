use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::EventPump;
use std::time::Duration;

pub struct Game {
    canvas: WindowCanvas,
    pub event_pump: EventPump,
}

impl Game {
    pub fn initialize() -> Game {
        // initializing graphics
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        println!("initialized sdl");
        let window = video_subsystem
            .window("rust-sdl2 demo", 640, 320)
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();
        canvas.present();
        let event_pump = sdl_context.event_pump().unwrap();
        Game {
            canvas: canvas,
            event_pump: event_pump,
        }
    }

    pub fn get_events(&mut self) -> sdl2::event::EventPollIterator {
        self.event_pump.poll_iter()
    }

    #[allow(unused_must_use)]
    pub fn draw(&mut self, gfx: &[bool; 64 * 32]) {
        self.canvas.set_draw_color(Color::RGB(0, 255, 255));
        self.canvas.clear();
        // The rest of the game loop goes here...
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        for x in 0..=63 {
            for y in 0..=31 {
                if gfx[64 * y + x] {
                    self.canvas
                        .fill_rect(Rect::new(x as i32 * 10, y as i32 * 10, 10, 10));
                }
            }
        }
        self.canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
