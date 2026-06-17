use std::time::{Duration, Instant};

use chipster::input::KEY_MAPPINGS;
use chipster::machine::Machine;
use macroquad::prelude::*;

const CHIP8_WIDTH: u32 = 64;
const CHIP8_HEIGHT: u32 = 32;
const WINDOW_SCALE: u32 = 10;

fn window_conf() -> Conf {
    Conf {
        window_title: "CHIP-8 Emulator".to_owned(),
        window_height: (CHIP8_HEIGHT * WINDOW_SCALE) as i32,
        window_width: (CHIP8_WIDTH * WINDOW_SCALE) as i32,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut m = Machine::new();
    if let Err(e) = m.load_rom("./demos/Trip8 Demo (2008) [Revival Studios].ch8") {
        eprintln!("Failed to load ROM: {}", e);
        return;
    }

    let cp_frame = 700 / 60;
    let mut last_tick = Instant::now();
    let time_delta = Duration::from_secs_f64(1.0 / 60.0);

    loop {
        for &(macroquad_key, key_idx) in &KEY_MAPPINGS {
            if is_key_down(macroquad_key) {
                m.keydown(key_idx);
            } else {
                m.keyup(key_idx);
            }
        }

        for _ in 0..cp_frame {
            m.cycle();
        }

        if last_tick.elapsed() >= time_delta {
            m.update_timers();
            last_tick = Instant::now();
        }

        clear_background(BLACK);

        let cell_width = screen_width() / CHIP8_WIDTH as f32;
        let cell_height = screen_height() / CHIP8_HEIGHT as f32;
        for y in 0..32 {
            for x in 0..64 {
                let pixel_idx = y * 64 + x;
                if m.display[pixel_idx] {
                    draw_rectangle(
                        x as f32 * cell_width,
                        y as f32 * cell_height,
                        cell_width,
                        cell_height,
                        WHITE,
                    );
                }
            }
        }

        print!("\rfps: {}", macroquad::time::get_fps());
        next_frame().await
    }
}
