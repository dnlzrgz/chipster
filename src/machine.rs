use std::{
    fs::{self},
    io::{self},
    path::Path,
};

use tinyrand::{Rand, StdRand};

const ROM_START_ADDR: usize = 0x200;

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Machine {
    // 4 KiB (0x1000) of addressable memory (bytes).
    memory: [u8; 4096],
    // 16 general-purpose 8-bit registers.
    // Vf is usually used as a flag.
    v: [u8; 16],
    // Index register.
    i: u16,
    // Program counter.
    pc: u16,
    // Call stack. Classical CHIP-8 has up to 16 levels.
    stack: [u16; 16],
    // Stack pointer.
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    pub display: [bool; 64 * 32],
    keypad: [bool; 16],
    rand: StdRand,
}

impl Default for Machine {
    fn default() -> Self {
        Machine::new()
    }
}

impl Machine {
    pub fn new() -> Self {
        let mut machine = Machine {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: ROM_START_ADDR as u16,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: [false; 64 * 32],
            keypad: [false; 16],
            rand: StdRand::default(),
        };

        machine.load_fontset();
        machine
    }

    fn load_fontset(&mut self) {
        self.memory[0..FONTSET.len()].copy_from_slice(&FONTSET);
    }

    pub fn load_rom<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let bytes = fs::read(path)?;
        self.load_rom_bytes(&bytes)
    }

    fn load_rom_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        if bytes.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "The selected ROM is empty",
            ));
        }

        let max_allowed_space = self.memory.len().saturating_sub(ROM_START_ADDR);
        if bytes.len() > max_allowed_space {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Rom size ({} bytes) exceeds available memory ({} bytes)",
                    bytes.len(),
                    max_allowed_space,
                ),
            ));
        }

        self.memory[ROM_START_ADDR..ROM_START_ADDR + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    pub fn cycle(&mut self) {
        let opcode = self.fetch_opcode();
        self.execute(opcode)
    }

    fn fetch_opcode(&self) -> u16 {
        // TODO: Check if return Result is better.
        let hi = self.memory[self.pc as usize] as u16;
        let lo = self.memory[self.pc as usize + 1] as u16;

        (hi << 8) | lo
    }

    fn execute(&mut self, opcode: u16) {
        self.pc += 2;

        let c = ((opcode & 0xF000) >> 12) as usize; // Opcode type (highest nibble)
        let x = ((opcode & 0x0F00) >> 8) as usize; // First register
        let y = ((opcode & 0x00F0) >> 4) as usize; // Second register
        let d = (opcode & 0x000F) as usize; // Lowest nibble
        let kk = (opcode & 0x00FF) as u8; // Constant (8-bit)
        let nnn = opcode & 0x0FFF; // Memory address (12-bit)

        match c {
            0x0 => match opcode {
                0xE0 => self.op_00e0(),
                0xEE => self.op_00ee(),
                _ => {}
            },
            0x1 => self.op_1nnn(nnn),
            0x2 => self.op_2nnn(nnn),
            0x3 => self.op_3xkk(x, kk),
            0x4 => self.op_4xkk(x, kk),
            0x5 if d == 0 => self.op_5xy0(x, y),
            0x6 => self.op_6xkk(x, kk),
            0x7 => self.op_7xkk(x, kk),
            0x8 => match d {
                0x0 => self.op_8xy0(x, y),
                0x1 => self.op_8xy1(x, y),
                0x2 => self.op_8xy2(x, y),
                0x3 => self.op_8xy3(x, y),
                0x4 => self.op_8xy4(x, y),
                0x5 => self.op_8xy5(x, y),
                0x6 => self.op_8xy6(x, y),
                0x7 => self.op_8xy7(x, y),
                0xE => self.op_8xye(x, y),
                _ => {}
            },
            0x9 if d == 0 => self.op_9xy0(x, y),
            0xA => self.op_annn(nnn),
            0xB => self.op_bnnn(nnn),
            0xC => self.op_cxkk(x, kk),
            0xD => self.op_dxyn(x, y, d),
            0xE => match kk {
                0x9E => self.op_ex9e(x),
                0xA1 => self.op_exa1(x),
                _ => {}
            },
            0xF => match kk {
                0x07 => self.op_fx07(x),
                0x0A => self.op_fx0a(x),
                0x15 => self.op_fx15(x),
                0x18 => self.op_fx18(x),
                0x1E => self.op_fx1e(x),
                0x29 => self.op_fx29(x),
                0x33 => self.op_fx33(x),
                0x55 => self.op_fx55(x),
                0x65 => self.op_fx65(x),
                _ => {}
            },
            _ => {}
        }
    }

    // Clears the screen.
    fn op_00e0(&mut self) {
        self.display.fill(false);
    }

    // Returns from a subroutine.
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    // Jumps to address nnn.
    fn op_1nnn(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    // Calls subroutine at nnn.
    fn op_2nnn(&mut self, nnn: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = nnn
    }

    // Skip nex instruction is Vx == kk
    fn op_3xkk(&mut self, x: usize, kk: u8) {
        if self.v[x] == kk {
            self.pc += 2;
        }
    }

    // Skips the next instruction if Vx != kk
    fn op_4xkk(&mut self, x: usize, kk: u8) {
        if self.v[x] != kk {
            self.pc += 2;
        }
    }

    // Skips the next instruction if Vx == Vy
    fn op_5xy0(&mut self, x: usize, y: usize) {
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }

    // Sets Vx to nn
    fn op_6xkk(&mut self, x: usize, kk: u8) {
        self.v[x] = kk
    }

    // Adds nn to Vx (carry flag is not changed)
    fn op_7xkk(&mut self, x: usize, kk: u8) {
        self.v[x] = self.v[x].wrapping_add(kk);
    }

    // Sets Vx = Vy
    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
    }

    // Sets Vx = Vx OR Vy
    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.v[x] |= self.v[y];
    }

    // Sets Vx = Vx AND Vy
    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.v[x] &= self.v[y]
    }

    // Sets Vx = Vx XOR Vy
    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.v[x] ^= self.v[y]
    }

    // Sets Vx = Vx + Vy and Vf as the carry
    fn op_8xy4(&mut self, x: usize, y: usize) {
        let (sum, carry) = self.v[x].overflowing_add(self.v[y]);
        self.v[0xF] = carry as u8;
        self.v[x] = sum;
    }

    // Sets Vx = Vx - Vy and Vf as the !borrow
    fn op_8xy5(&mut self, x: usize, y: usize) {
        let (diff, borrow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[0xF] = (!borrow) as u8;
        self.v[x] = diff;
    }

    // Sets Vx = Vx SHR 1 and Vf to the shifted out bit
    fn op_8xy6(&mut self, x: usize, _y: usize) {
        self.v[0xF] = self.v[x] & 0x01;
        self.v[x] >>= 1;
    }

    // Sets Vx = Vy - Vx and Vf as the !borrow
    fn op_8xy7(&mut self, x: usize, y: usize) {
        let (result, borrow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[0xF] = (!borrow) as u8;
        self.v[x] = result;
    }

    // Sets Vx = Vx SHL 1 and Vf to the shifted out bit
    fn op_8xye(&mut self, x: usize, _y: usize) {
        self.v[0xF] = (self.v[x] & 0x80) >> 7;
        self.v[x] <<= 1;
    }

    // Skips the next instruction if Vx != Vy
    fn op_9xy0(&mut self, x: usize, y: usize) {
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }

    // Sets the index register to nnn
    fn op_annn(&mut self, nnn: u16) {
        self.i = nnn;
    }

    // Jumps to address nnn + V0
    fn op_bnnn(&mut self, nnn: u16) {
        self.pc = nnn + (self.v[0] as u16);
    }

    // Sets Vx = random byte AND kk
    fn op_cxkk(&mut self, x: usize, kk: u8) {
        let rand_byte = (self.rand.next_u64() & 0xFF) as u8;
        self.v[x] = rand_byte & kk;
    }

    // Draws a sprite at coordinates (Vx, Vy)
    fn op_dxyn(&mut self, x: usize, y: usize, d: usize) {
        let start_x = self.v[x] as usize % 64;
        let start_y = self.v[y] as usize % 32;

        self.v[0xF] = 0;

        for row in 0..d {
            let sprite_byte = self.memory[(self.i as usize) + row];
            let current_y = start_y + row;
            if current_y >= 32 {
                break;
            }

            for col in 0..8 {
                let current_x = start_x + col;
                if current_x >= 64 {
                    break;
                }

                if (sprite_byte & (0x80 >> col)) != 0 {
                    let pixel_idx = current_y * 64 + current_x;

                    if self.display[pixel_idx] {
                        self.v[0xF] = 1;
                    }

                    self.display[pixel_idx] ^= true;
                }
            }
        }
    }

    // Skips next instruction if key stored in Vx is pressed
    fn op_ex9e(&mut self, x: usize) {
        let key = self.v[x] as usize;
        if key < 16 && self.keypad[key] {
            self.pc += 2;
        }
    }

    // Skips next instruction if key stored in Vx is NOT pressed
    fn op_exa1(&mut self, x: usize) {
        let key = self.v[x] as usize;
        if key < 16 && !self.keypad[key] {
            self.pc += 2;
        }
    }

    // Sets Vx = delay timer value
    fn op_fx07(&mut self, x: usize) {
        self.v[x] = self.delay_timer;
    }

    // Acknowledge a keypress, store it in Vx (blocking)
    fn op_fx0a(&mut self, x: usize) {
        let mut key_pressed = false;
        for i in 0..16 {
            if self.keypad[i] {
                self.v[x] = i as u8;
                key_pressed = true;
                break;
            }
        }

        if !key_pressed {
            self.pc -= 2;
        }
    }

    // Sets delay timer = Vx
    fn op_fx15(&mut self, x: usize) {
        self.delay_timer = self.v[x];
    }

    // Sets sound timer = Vx
    fn op_fx18(&mut self, x: usize) {
        self.sound_timer = self.v[x];
    }

    // Adds Vx to I
    fn op_fx1e(&mut self, x: usize) {
        self.i = self.i.wrapping_add(self.v[x] as u16);
    }

    // Sets I = location of sprite for digit stored in Vx
    fn op_fx29(&mut self, x: usize) {
        self.i = (self.v[x] as u16) * 5;
    }

    // Stores BCD representation of Vx in memory locations I, I+1, and I+2
    fn op_fx33(&mut self, x: usize) {
        let value = self.v[x];
        self.memory[self.i as usize] = value / 100;
        self.memory[(self.i + 1) as usize] = (value / 10) % 10;
        self.memory[(self.i + 2) as usize] = value % 10;
    }

    // Stores V0 through Vx in memory starting at i (index register)
    fn op_fx55(&mut self, x: usize) {
        for index in 0..=x {
            self.memory[(self.i as usize) + index] = self.v[index];
        }
    }

    // Fills V0 through Vx with values from memory starting at i (index register)
    fn op_fx65(&mut self, x: usize) {
        for index in 0..=x {
            self.v[index] = self.memory[(self.i as usize) + index];
        }
    }

    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn keydown(&mut self, key: usize) {
        if key < 16 {
            self.keypad[key] = true;
        }
    }

    pub fn keyup(&mut self, key: usize) {
        if key < 16 {
            self.keypad[key] = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fontset_is_loaded_at_start() {
        let m = Machine::new();
        assert_eq!(&m.memory[0..FONTSET.len()], &FONTSET);
    }

    #[test]
    fn load_rom_into_memory() {
        let mut m = Machine::new();
        let dummy_rom = vec![0x12, 0x34, 0x56, 0x78];

        let result = m.load_rom_bytes(&dummy_rom);
        assert!(result.is_ok());

        let end = ROM_START_ADDR + dummy_rom.len();
        assert_eq!(&m.memory[ROM_START_ADDR..end], &dummy_rom);
    }

    #[test]
    fn returns_err_from_empty_rom() {
        let mut m = Machine::new();
        let empty_rom: Vec<u8> = vec![];
        let result = m.load_rom_bytes(&empty_rom);
        assert!(result.is_err());
    }

    #[test]
    fn returns_err_for_oversized_rom() {
        let mut m = Machine::new();
        let max_allowed_space = m.memory.len() - ROM_START_ADDR;
        let oversized_rom = vec![0u8; max_allowed_space + 1];

        let result = m.load_rom_bytes(&oversized_rom);
        assert!(result.is_err());
    }
}
