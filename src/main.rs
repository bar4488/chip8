mod chip8;
use chip8::Chip8;

mod game;
use game::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub fn main() {
    let mut game = Game::initialize();
    let mut chip = Chip8::new();

    println!("entering loop");
    chip.test_drawing();
    'running: loop {
        for event in game.get_events() {
            match event {
                sdl2::event::Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        //chip.cycle();
        game.draw(&chip.gfx);
    }
    println!("exited loop");
}
