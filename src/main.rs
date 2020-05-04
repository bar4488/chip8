mod chip8;
use chip8::Chip8;

mod game;
use game::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::io::Read;
use std::fs::File;

pub fn main() {
    let program: Vec<u8> = load_chip8_program();
    let mut game = Game::initialize();
    let mut chip = Chip8::load(program);

    println!("entering loop");
    //chip.test_drawing();
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
        chip.cycle();
        game.draw(&chip.gfx);
    }
    println!("exited loop");
}

fn load_chip8_program() -> Vec<u8> {
    let mut filename = String::new();
    std::io::stdin().read_line(&mut filename).expect("Error reading input");
    println!("loading program {}...", &filename);

    let mut f = File::open(&filename.trim()).expect("no file found");
    let metadata = std::fs::metadata(&filename.trim()).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read(&mut buffer).expect("buffer overflow");

    buffer
}
