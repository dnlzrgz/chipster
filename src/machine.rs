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
            pc: 0x200, // CHIP-8 programs usually start a 0x200
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fontset_is_loaded_at_the_correct_location_in_memory() {
        let m = Machine::new();
        assert_eq!(&m.memory[0..FONTSET.len()], &FONTSET);
    }
}
