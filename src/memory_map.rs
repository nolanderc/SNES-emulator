use crate::snes_header::*;

#[macro_use]
mod registers;
use registers::RegisterMap;

use hardware_registers::HardwareRegisters;

#[macro_use]
mod macros;

/// Maps different memory adresses to memory storages in the CPU
pub struct MemoryMap<'a> {
    rom: &'a [u8],
    wram: WorkRam,
    sram: SaveRam,
    hardware_registers: HardwareRegisters,
}

define_memory_access! {
    hardware_registers = [
        0x2100 => ScreenDisplayRegister   ( screen_display   ),
        0x2140 => ApuIoRegister0          ( apu_io0          ),
        0x2141 => ApuIoRegister1          ( apu_io1          ),
        0x2142 => ApuIoRegister2          ( apu_io2          ),
        0x2143 => ApuIoRegister3          ( apu_io3          ),
        0x4200 => InterruptEnableRegister ( interrupt_enable ),
        0x420c => HdmaEnableRegister      ( hdma_enable      ),
        0x420b => DmaEnableRegister       ( dma_enable       )
    ]
    other {
        Rom(usize)
    }
    get(memory) {
        Rom(index) => memory.rom[index]
    }
    get_mut(memory) {
        Rom(_) => panic!("Attempted write to ROM!")
    }
}

impl WorkRam {
    pub fn new() -> WorkRam {
        WorkRam {
            data: [0; 128 * 1024],
        }
    }
}

const WRAM_SIZE: usize = 128 * 1024;

/// 128 KB of Work RAM (WRAM)
struct WorkRam {
    data: [u8; WRAM_SIZE],
}

/// Save RAM, stores saves files on the cartridge
struct SaveRam;

impl<'a> MemoryMap<'a> {
    pub fn new(rom: &'a [u8]) -> Self {
        MemoryMap {
            rom,
            wram: WorkRam::new(),
            sram: SaveRam,
            hardware_registers: HardwareRegisters::default(),
        }
    }

    pub fn get_snes_header(&self) -> SnesHeader<'a> {
        self.get_lorom_header()
    }

    pub fn get_byte(&self, bank: u8, addr: u16) -> u8 {
        let access = self.get_memory_access_lorom(bank, addr);
        self.access_byte(access)
    }

    pub fn set_byte(&mut self, bank: u8, addr: u16, value: u8) {
        let access = self.get_memory_access_lorom(bank, addr);
        *self.access_byte_mut(access) = value;
    }

    /*
    fn access_byte(&self, access: MemoryAccess) -> u8 {
        macro_rules! get_access {
            { hardware_registers = [$($reg:ident),*] $($tt:tt)* } => {
                match access {
                    $(MemoryAccess::$reg => get_access!($reg) ),*
                    $($tt)*
                }
            };

            ( $reg:ident ) => {
                RegisterMap::<hardware_registers::$reg>::get(&self.hardware_registers)
            }
        };

        use MemoryAccess::*;
        get_access! {
            hardware_registers = [
                InterruptEnableRegister,
                HdmaEnableRegister
            ],

            Rom(index) => self.rom[index],
        }
    }

    fn access_byte_mut(&mut self, access: MemoryAccess) -> &mut u8 {
        macro_rules! get_mut_access {
            { hardware_registers = [$($reg:ident),*] $($tt:tt)* } => {
                match access {
                    $(MemoryAccess::$reg => get_mut_access!($reg) ),*
                    $($tt)*
                }
            };

            ( $reg:ident ) => {
                RegisterMap::<hardware_registers::$reg>::get_mut(&mut self.hardware_registers)
            }
        };

        use MemoryAccess::*;
        get_mut_access! {
            hardware_registers = [
                InterruptEnableRegister,
                HdmaEnableRegister
            ],

            Rom(_) => panic!("Attempted write to ROM!"),
        }
    }
    */

    fn get_lorom_header(&self) -> SnesHeader<'a> {
        let start = 0x7fc0;
        let end = 0x7fff;
        SnesHeader::from_bytes(&self.rom[start..=end])
    }

    // ============== //
    // LoROM mappings //
    // ============== //

    fn get_memory_access_lorom(&self, bank: u8, addr: u16) -> MemoryAccess {
        match bank {
            0x00..=0x3F => self.get_bank_00_3f(bank, addr),
            0x40..=0x6F => self.get_bank_40_6f(bank, addr),
            0x70..=0x7D => self.get_bank_70_7d(bank, addr),
            0x7E => self.get_bank_7e(addr),
            0x7F => self.get_bank_7f(addr),
            0x80..=0xBF => self.get_bank_00_3f(bank - 0x80, addr),
            0xC0..=0xEF => self.get_bank_40_6f(bank - 0xC0 + 0x40, addr),
            0xF0..=0xFD => self.get_bank_70_7d(bank - 0xF0 + 0x70, addr),
            0xFE..=0xFF => self.get_bank_fe_ff(bank, addr),
        }
    }

    fn get_bank_00_3f(&self, bank: u8, addr: u16) -> MemoryAccess {
        match addr {
            // LowRAM, shadowed from bank $7E
            0x0000..=0x1FFF => self.get_bank_7e(addr),

            // Unused
            0x2000..=0x20FF => unimplemented!(),

            // PPU1, APU, hardware registers
            0x2100..=0x21FF => Self::get_hardware_register(addr),

            // Unused
            0x2200..=0x2FFF => unimplemented!(),

            // DSP, SuperFX, hardware registers (I couldn't find any source)
            0x3000..=0x3FFF => unimplemented!(),

            // Old Style Joypad Registers
            0x4000..=0x40FF => Self::get_hardware_register(addr),

            // Unused
            0x4100..=0x41FF => unimplemented!(),

            // DMA, PPU2, hardware registers
            0x4200..=0x44FF => Self::get_hardware_register(addr),

            // Unused
            0x4500..=0x5FFF => unimplemented!(),

            // RESERVED (enhancement chips memory)
            0x6000..=0x7FFF => unimplemented!(),

            // LoROM (000000-1FFFFF)
            0x8000..=0xFFFF => {
                let rom = u32::from(bank) * 0x8000 + u32::from(addr) - 0x8000;
                MemoryAccess::Rom(rom as usize)
            }
        }
    }

    fn get_bank_40_6f(&self, bank: u8, addr: u16) -> MemoryAccess {
        match addr {
            // Unused if the chip is not MAD-1
            0x0000..=0x7FFF => unimplemented!(),

            // LoROM (200000-37FFFF)
            0x8000..=0xFFFF => unimplemented!(),
        }
    }

    fn get_bank_70_7d(&self, bank: u8, addr: u16) -> MemoryAccess {
        match addr {
            // Cartridge SRAM
            0x0000..=0x7FFF => unimplemented!(),

            // LoROM (380000-3EFFFF)
            0x8000..=0xFFFF => unimplemented!(),
        }
    }

    fn get_bank_7e(&self, addr: u16) -> MemoryAccess {
        match addr {
            // LowRAM (WRAM)
            0x0000..=0x1FFF => unimplemented!(),

            // HighRAM (WRAM)
            0x2000..=0x7FFF => unimplemented!(),

            // Extended RAM (WRAM)
            0x8000..=0xFFFF => unimplemented!(),
        }
    }

    fn get_bank_7f(&self, addr: u16) -> MemoryAccess {
        match addr {
            // Extended RAM (WRAM)
            0x0000..=0xFFFF => unimplemented!(),
        }
    }

    fn get_bank_fe_ff(&self, bank: u8, addr: u16) -> MemoryAccess {
        match addr {
            // Cartridge SRAM - 64 Kilobytes (512 KB total)
            0x0000..=0x7FFF => unimplemented!(),

            // LoROM (3F0000-3FFFFF)
            0x8000..=0xFFFF => unimplemented!(),
        }
    }
}
