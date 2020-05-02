mod chip8;
use chip8::Chip8;

mod graphics;
use graphics::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub fn main() {
    let mut chip = Chip8::new();
    let mut game = Game::initialize();

    println!("entering loop");
    'running: loop {
        for event in game.get_events() {
            match event {
                sdl2::event::Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {

                }
            }
        }
        chip.cycle();
        game.draw(&chip.gfx);
    }
    println!("exited loop");
}
