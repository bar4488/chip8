extern crate rand;

use rand::Rng;
use std::time::Instant;

const PROGRAM_START_LOCATION: usize = 40 * 10;


#[allow(dead_code)]
pub struct Chip8 {
    // the currently proccessed instruction code
    opcode: u16,
    // registers v0 to vf, vf s used as a flag register
    v: [u8; 16],
    // stores memory addresses
    index: u16,
    //store the currently executing address.
    pc: u16,
    // points to the topmost level of the stack.
    sp: u8,
    // decremented at a rate of 60hz
    delay_timer: u8,
    // decrements at a rate of 60 hz, when not 0, a buzzer will sound
    sound_timer: u8,

    // variables that save the time in milliseconds since the delta or sound timer were set.
    delay_set_time: Option<Instant>,
    sound_set_time: Option<Instant>,

    // stores the address that the interpreter should return
    // to when finished with a subroutine
    stack: [u16; 16],
    // chip8 programs start at location 0x200 (5012) or 0x600(1536)
    memory: [u8; 4096],
    // 64*32 pixel display, top left is (0,0) bottom right is (63, 31)
    pub gfx: [bool; 64 * 32],

    //keyboard has 16 keys, the array is used to indicate the state of each key
    keyboard: [bool; 16],
}

#[allow(dead_code)]
impl Chip8 {
    pub fn new() -> Chip8 {
        let mut rng = rand::thread_rng();
        Chip8 {
            opcode: 0,
            v: [0; 16],
            delay_timer: 0,
            index: 0,
            pc: 0,
            sound_timer: 0,
            sp: 0,
            stack: [0u16; 16],
            gfx: [false; 64 * 32],
            memory: [rng.gen_range(0, 255); 4096],
            keyboard: [false; 16],
            delay_set_time: None,
            sound_set_time: None,
        }
    }

    pub fn test_drawing(&mut self) {
        let sprite = [0b00011111, 0b00010101, 0b00011011, 0b00010101, 0b00011111];
        self.index = 0x10;
        self.memory[0x10..0x15].clone_from_slice(&sprite);
        self.opcode = 0xd015;
        self.v[0] = 0x5;
        self.v[1] = 0x3;
        self.process_opcode();
    }

    pub fn load_instructions(&mut self, instructions: &[u8]) {
        let program_memory = &mut self.memory[PROGRAM_START_LOCATION..];
        program_memory.clone_from_slice(&instructions);
    }

    pub fn get_sound_timer(&self) -> u8{

        if let Some(elapsed) = self.sound_set_time {
            let delta = (elapsed.elapsed().as_millis() * 60 / 1000) as u8;
            if delta > self.sound_timer {
                0
            }
            else {
                self.sound_timer - delta
            }
        }
        else {
            0
        }
    }

    pub fn get_delay_timer(&self) -> u8{
        if let Some(elapsed) = self.delay_set_time {
            let delta = (elapsed.elapsed().as_millis() * 60 / 1000) as u8;
            if delta > self.delay_timer {
                0
            }
            else {
                self.delay_timer - delta
            }
        }
        else {
            0
        }
    }

    pub fn cycle(&mut self) {
        // read current opcode from memory to self.opcode
        self.opcode =
            ((self.memory[self.pc as usize] as u16) << 8) + self.memory[self.pc as usize + 1] as u16;
        // process opcode
        self.process_opcode();
    }

    fn process_opcode(&mut self) {
        println!("processing opcode: {:#x?}", self.opcode);
        self.pc += 2;
        match self.opcode {
            0x00e0 => {
                self.gfx = [false; 64 * 32];
                println!("clear display");
            }
            0x00ee => {
                self.pc = self.stack[(self.sp - 1) as usize];
                self.sp -= 1;
                println!("return from subroutine");
            }
            0x0000..=0x0fff => {
                self.pc = self.opcode & 0x0fff;
                println!(
                    "jump to old machine code routine at {}",
                    self.opcode & 0x0fff
                );
            }
            0x1000..=0x1fff => {
                self.pc = self.opcode & 0x0fff;
                println!("jump to addr {}", self.opcode & 0x0fff);
            }
            0x2000..=0x2fff => {
                // initialize a new function routine
                self.sp += 1;
                self.stack[(self.sp - 1) as usize] = self.pc;
                self.pc = self.opcode & 0x0fff;
                println!("call subroutine at addr {}", self.opcode & 0x0fff);
            }
            0x3000..=0x3fff => {
                if self.v[((self.opcode & 0x0f00) >> 8) as usize] == (self.opcode & 0x00ff) as u8 {
                    self.pc += 2;
                }
                println!(
                    "skip if v{} = {}",
                    (self.opcode & 0x0f00) >> 8,
                    self.opcode & 0x00ff
                );
            }
            0x4000..=0x4fff => {
                if self.v[((self.opcode & 0x0f00) >> 8) as usize] != (self.opcode & 0x00ff) as u8 {
                    self.pc += 2;
                }
                println!(
                    "skip if v{}({}) != {}",
                    (self.opcode & 0x0f00) >> 8,
                    self.v[((self.opcode & 0x0f00) >> 8) as usize],
                    self.opcode & 0x00ff
                );
            }
            0x5000..=0x5fff => {
                if self.v[((self.opcode & 0x0f00) >> 8) as usize]
                    == self.v[((self.opcode & 0x00f0) >> 4) as usize]
                {
                    self.pc += 2;
                }
                println!(
                    "skip if v{} = v{}",
                    (self.opcode & 0x0f00) >> 8,
                    (self.opcode & 0x00f0) >> 4
                );
            }
            0x6000..=0x6fff => {
                self.v[((self.opcode & 0x0f00) >> 8) as usize] = (self.opcode & 0x00ff) as u8;
                println!("set v{} to {}", self.opcode & 0x0f00, self.opcode & 0x00ff);
            }
            0x7000..=0x7fff => {
                self.v[((self.opcode & 0x0f00) >> 8) as usize].wrapping_add((self.opcode & 0x00ff) as u8);
                println!("set v{} to {}", self.opcode & 0x0f00, self.opcode & 0x00ff);
            }
            0x8000..=0x8fff => {
                let x = ((self.opcode & 0x0f00) >> 8) as usize;
                let y = ((self.opcode & 0x00f0) >> 4) as usize;
                let n = self.opcode & 0x000f;
                match n {
                    0 => self.v[x] = self.v[y],
                    1 => self.v[x] = self.v[x] | self.v[y],
                    2 => self.v[x] = self.v[x] & self.v[y],
                    3 => self.v[x] = self.v[x] ^ self.v[y],
                    4 => {
                        let (result, has_overflown) = self.v[x].overflowing_add(self.v[y]);
                        self.v[x] = result;
                        self.v[0xf] = match has_overflown {
                            true => 1,
                            false => 0,
                        };
                    }
                    5 => {
                        self.v[0xf] = match self.v[x] > self.v[y] {
                            true => 1,
                            false => 0,
                        };
                        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
                    }
                    6 => {
                        self.v[0xf] = self.v[x] & 0x0001;
                        self.v[x] = self.v[x] >> 2;
                    }
                    7 => {
                        self.v[0xf] = match self.v[y] > self.v[x] {
                            true => 1,
                            false => 0,
                        };
                        self.v[x] = self.v[y].wrapping_sub(self.v[x]);
                    }
                    0xe => {
                        self.v[0xf] = match self.v[x] & 0x0008 != 0 {
                            true => 1,
                            false => 0,
                        };
                        self.v[x] = self.v[x] << 1;
                    }
                    _ => {}
                }
            }
            0x9000..=0x9fff => {
                let x = ((self.opcode & 0x0f00) >> 8) as usize;
                let y = ((self.opcode & 0x00f0) >> 4) as usize;
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            0xa000..=0xafff => {
                self.index = self.opcode & 0x0fff;
            }
            0xb000..=0xbfff => {
                self.pc = (self.v[0] as u16) + (self.opcode & 0x0fff);
            }
            0xc000..=0xcfff => {
                let mut rng = rand::thread_rng();
                self.v[((self.opcode & 0x0f00) >> 8) as usize] =
                    (rng.gen_range(0, 256) & self.opcode & 0x00ff) as u8;
            }
            0xd000..=0xdfff => {
                let x = self.v[((self.opcode & 0x0f00) >> 8) as usize];
                let y = self.v[((self.opcode & 0x00f0) >> 4) as usize];
                let n = self.opcode & 0x000f;

                let end_slice = (self.index + n) as usize;
                let sprite = &self.memory[(self.index as usize)..end_slice];
                for xx in 0..8 {
                    for yy in 0..sprite.len() {
                        let gx = (x as usize + xx) % 64;
                        let gy = (y as usize + yy) % 32;

                        let pixel = (sprite[yy] & (1 << xx)) != 0;
                        if self.gfx[gy * 64 + gx] {
                            self.v[0xf] = 1;
                        }
                        self.gfx[gy * 64 + gx] = self.gfx[gy * 64 + gx] ^ pixel;
                        println!("drawing at x: {}, y: {}", gx, gy);
                    }
                }
            }
            0xe000..=0xefff => {
                let x = (self.opcode & 0x0f00 >> 8) as usize;
                match self.opcode & 0x00ff {
                    0x9e => {
                        if self.keyboard[self.v[x] as usize] {
                            self.pc += 2;
                        }
                    }
                    0xa1 => {
                        if !self.keyboard[self.v[x] as usize] {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        //panic!("unimplemented opcode: {}", self.opcode);
                    }
                }
            }
            0xf000..=0xffff => {
                let x = (self.opcode & 0x0f00 >> 8) as usize;
                match self.opcode & 0x00ff {
                    0x07 => {
                        self.v[x] = self.get_delay_timer();
                    }
                    0x0a => {
                        // wait for a key press
                        // TODO: add logic to suspend program execution while no key is pressed
                        if self.keyboard.contains(&true) {
                            self.v[x] = self.keyboard.iter().position(|&x| x).unwrap() as u8;
                        } else {
                            // return the program counter so it will stay on the current instruction if theres no key pressed
                            self.pc -= 0;
                        }
                    }
                    0x15 => {
                        self.delay_timer = self.v[x];
                        self.delay_set_time = Some(Instant::now());
                    }
                    0x18 => {
                        self.sound_timer = self.v[x];
                        self.sound_set_time = Some(Instant::now());
                    }
                    0x1e => {
                        self.index += self.v[x] as u16;
                    }
                    0x29 => {
                        self.index = self.v[x] as u16 * 40;
                    }
                    0x33 => {
                        let hundrents = self.v[x] / 100;
                        let tens = (self.v[x] / 10) % 10;
                        let ones = self.v[x] % 10;

                        self.memory[self.index as usize] = hundrents;
                        self.memory[self.index as usize + 1] = tens;
                        self.memory[self.index as usize + 2] = ones;
                    }
                    0x55 => {
                        let i = self.index as usize;
                        self.memory[i..=(i + x)].clone_from_slice(&self.v[0..=x]);
                    }
                    0x65 => {
                        let i = self.index as usize;
                        self.v[0..=x].clone_from_slice(&self.memory[i..=(i + x)]);
                    }
                    _ => {
                        //panic!("unimplemented opcode: {}", self.opcode);
                    }
                }
            }
        }
    }
}

impl std::fmt::Debug for Chip8 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.v)
    }
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jump_opcode() {
        let mut chip = Chip8::new();

        // assert SYS addr opcode is working
        chip.opcode = 0x1111;
        chip.process_opcode();
        assert_eq!(chip.pc, 0x0111);

        // assert JP addr opcode is working
        chip.opcode = 0x0134;
        chip.process_opcode();
        assert_eq!(chip.pc, 0x0134);
    }

    #[test]
    fn test__subroutine() {
        let mut chip = Chip8::new();

        // test calling a subroutine
        chip.pc = 0x0111;
        let previous_pc = chip.pc;
        chip.opcode = 0x2123;
        chip.process_opcode();
        assert_eq!(chip.pc, 0x0123);
        assert_eq!(chip.sp, 1);
        assert_eq!(chip.stack[(chip.sp - 1) as usize], previous_pc + 2);

        // test returning from a subroutine
        chip.opcode = 0x00ee;
        chip.pc = 0x0111;
        chip.process_opcode();
        assert_eq!(chip.pc, 0x0111 + 2);
        assert_eq!(chip.sp, 0);
    }

    #[test]
    fn test_skip_equal_vxb() {
        let mut chip = Chip8::new();

        // test skip equal
        chip.pc = 0x0111;
        let previous_pc = chip.pc;
        chip.opcode = 0x3244;
        chip.process_opcode();
        assert_eq!(chip.pc, previous_pc + 2);

        chip.v[2] = 0x44;
        chip.pc = 0x0111;
        chip.process_opcode();
        assert_eq!(chip.pc, previous_pc + 4);
    }

    #[test]
    fn test_skip_ne_vxb() {
        let mut chip = Chip8::new();

        // test skip not equal
        chip.pc = 0x0111;
        let previous_pc = chip.pc;
        chip.v[2] = 0x11;
        chip.opcode = 0x4244;
        chip.process_opcode();
        assert_eq!(chip.pc, previous_pc + 4);

        chip.pc = 0x0111;
        let previous_pc = chip.pc;
        chip.v[2] = 0x44;
        chip.process_opcode();
        assert_eq!(chip.pc, previous_pc + 2);
    }

    #[test]
    fn test_skip_eq_vxvy() {
        let mut chip = Chip8::new();
        chip.pc = 0x0111;

        // test skip equal
        chip.pc = 0x0111;
        let previous_pc = chip.pc;
        chip.v[2] = 0x11;
        chip.v[3] = 0x11;
        chip.opcode = 0x5230;
        chip.process_opcode();
        assert_eq!(chip.pc, previous_pc + 4);

        chip.pc = 0x0111;
        let previous_pc = chip.pc;
        chip.v[2] = 0xff;
        chip.v[3] = 0x11;
        chip.opcode = 0x5230;
        chip.process_opcode();
        assert_eq!(chip.pc, previous_pc + 2);
    }

    #[test]
    fn test_skip_ne_vxvy() {
        let mut chip = Chip8::new();
        chip.pc = 0x0111;

        // test skip equal
        let previous_pc = chip.pc;
        chip.v[2] = 0x11;
        chip.v[3] = 0x11;
        chip.opcode = 0x9230;
        chip.process_opcode();
        assert_eq!(chip.pc, previous_pc + 2);

        let previous_pc = chip.pc;
        chip.v[2] = 0xff;
        chip.v[3] = 0x11;
        chip.opcode = 0x9230;
        chip.process_opcode();
        assert_eq!(chip.pc, previous_pc + 4);
    }

    #[test]
    fn test_set_vxkk() {
        let mut chip = Chip8::new();
        chip.pc = 0x0111;

        // test skip equal
        chip.v[2] = 0x3;
        chip.opcode = 0x6240;
        chip.process_opcode();
        assert_eq!(chip.v[2], 0x40);
    }

    #[test]
    fn test_add_vxkk() {
        let mut chip = Chip8::new();
        chip.pc = 0x0111;

        // test skip equal
        chip.v[2] = 0x3;
        let previous_v2 = chip.v[2];
        chip.opcode = 0x7240;
        chip.process_opcode();
        assert_eq!(chip.v[2], previous_v2 + 0x40);
    }

    #[test]
    fn test_vxvy() {
        let mut chip = Chip8::new();
        let mut rng = rand::thread_rng();
        let tries = 10;
        chip.pc = 0x0111;

        // test set vx vy

        for _ in 0..tries {
            chip.v[4] = rng.gen_range(0, 255);
            chip.v[2] = rng.gen_range(0, 255);
            chip.opcode = 0x8240;
            chip.process_opcode();
            assert_eq!(chip.v[2], chip.v[4]);
        }

        // test or vx vy
        for _ in 0..tries {
            chip.v[4] = rng.gen_range(0, 255);
            chip.v[2] = rng.gen_range(0, 255);
            let previous_vx = chip.v[2];
            chip.opcode = 0x8241;
            chip.process_opcode();
            assert_eq!(chip.v[2], previous_vx | chip.v[4]);
        }

        // test and vx vy
        for _ in 0..tries {
            chip.v[4] = rng.gen_range(0, 255);
            chip.v[2] = rng.gen_range(0, 255);
            let previous_vx = chip.v[2];
            chip.opcode = 0x8242;
            chip.process_opcode();
            assert_eq!(chip.v[2], previous_vx & chip.v[4]);
        }

        // test and vx vy
        for _ in 0..tries {
            chip.v[4] = rng.gen_range(0, 255);
            chip.v[2] = rng.gen_range(0, 255);
            let previous_vx = chip.v[2];
            chip.opcode = 0x8243;
            chip.process_opcode();
            assert_eq!(chip.v[2], previous_vx ^ chip.v[4]);
        }

        // test vx add vy
        for _ in 0..tries {
            chip.v[4] = rng.gen_range(0, 255);
            chip.v[2] = rng.gen_range(0, 255);
            let previous_vx = chip.v[2];
            chip.opcode = 0x8244;
            chip.process_opcode();
            let (res, carry) = previous_vx.overflowing_add(chip.v[4]);
            assert_eq!(chip.v[2], res);
            assert_eq!(
                chip.v[0xf],
                match carry {
                    true => 1,
                    false => 0,
                }
            );
        }

        // test vx sub vy
        for _ in 0..tries {
            chip.v[4] = rng.gen_range(0, 255);
            chip.v[2] = rng.gen_range(0, 255);
            chip.opcode = 0x8245;
            let vf = match chip.v[2] > chip.v[4] {
                true => 1,
                false => 0,
            };
            let res = chip.v[2].wrapping_sub(chip.v[4]);
            chip.process_opcode();
            assert_eq!(chip.v[2], res);
            assert_eq!(chip.v[0xf], vf);
        }

        // test and vx vy
        for _ in 0..tries {
            chip.v[4] = rng.gen_range(0, 255);
            chip.v[2] = rng.gen_range(0, 255);
            chip.opcode = 0x8245;
            let vf = match chip.v[2] > chip.v[4] {
                true => 1,
                false => 0,
            };
            let res = chip.v[2].wrapping_sub(chip.v[4]);
            chip.process_opcode();
            assert_eq!(chip.v[2], res);
            assert_eq!(chip.v[0xf], vf);
        }
    }
}
