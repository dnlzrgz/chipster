use std::{
    fs::{self},
    io::{self},
    path::Path,
};

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
    keypad: [bool; 16],
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
            keypad: [false; 16],
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
