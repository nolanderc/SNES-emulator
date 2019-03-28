
#![allow(dead_code)]

use log::*;

mod snes_header;
use snes_header::*;

mod cpu;
use cpu::*;

mod memory_map;
use memory_map::*;

/// Emulated Super Nintendo Entertainment System
pub struct Snes<'a> {
    core: Cpu,
    video: Video,
    sound: Sound,
    memory: MemoryMap<'a>,
}

/// Picture Processing Unit (ppu).
#[derive(Default)]
struct Video {
    memory: VideoRam
}

/// Sound Controller Chip: 8-bit Sony SPC700
#[derive(Default)]
struct Sound {
    memory: SoundRam
}

/// Video RAM: 64 KB (VRAM)
#[derive(Default)]
struct VideoRam;

/// Sound RAM: 512 kilobit (SRAM)
#[derive(Default)]
struct SoundRam;



impl<'a> Snes<'a> {
    pub fn start(mut rom: &'a[u8]) {
        let smc_header_size = rom.len() % 1024;
        info!("SMC header size: {}", smc_header_size);

        // if the header's length is not 512, it's malformed
        assert!(smc_header_size == 0 || smc_header_size == 512);

        // Cut the SMC header
        if smc_header_size == 512 {
            rom = &rom[512..];
        }

        let memory = MemoryMap::new(rom);

        let snes = Snes {
            core: Cpu::new(&memory),
            video: Video::default(),
            sound: Sound::default(),
            memory
        };

        snes.run();
    }

    fn run(mut self) {
        self.core.reset();

        loop {
            self.core.tick(&mut self.memory)
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_test_rom() {
        let _ = simple_logger::init();

        let rom = include_bytes!("../asm/test.smc");
        let _snes = Snes::start(rom);
    }

    #[test]
    fn load_super_mario_world_rom() {
        let _ = simple_logger::init();

        let rom = include_bytes!("../rom/Super Mario World (U) [!].smc");
        let _snes = Snes::start(rom);
    }
}
