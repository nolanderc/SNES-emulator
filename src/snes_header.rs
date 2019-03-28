
#[derive(Debug)]
pub struct SnesHeader<'a> {
    pub title: &'a str,
    pub makeup: RomMakeup,
    pub kind: RomKind,

    /// The logarithmic size of the ROM in kB. #bytes = `1024 << rom_size`
    pub rom_size: u8,

    /// The logarithmic size of the SRAM in kB. #bytes = `1024 << rom_size`
    pub sram_size: u8,

    pub creator_id: u8,
    pub version: u8,
    pub checksum_complement: u8,
    pub checksum: u8,

    pub native_interrupts: InterruptVector,
    pub emulation_interrupts: InterruptVector
}

#[derive(Debug)]
pub struct InterruptVector {
    /// Co-processor enable
    pub cop: u16,
    pub brk: u16,
    pub abort: u16,

    /// Non-maskable interrupt, called when v-blank begins
    pub nmi: u16,

    /// Reset vector, execution begins via this vector.
    pub reset: u16,

    /// Interrupt request. Can be set to be called at a certain spot in the horizontal refresh cycle
    pub irq: u16,
}

#[derive(Debug)]
pub enum RomMakeup {
    LoRom
}

#[derive(Debug)]
pub enum RomKind {
    Rom,
    Ram,
    Sram,
    Dsp1,
    Fx
}


impl<'a> SnesHeader<'a> {
    pub fn from_bytes(bytes: &'a[u8]) -> Self {
        let native_start = 0x24;
        let native_end = 0x2f;
        let emulation_start = 0x34;
        let emulation_end = 0x3f;

        SnesHeader {
            title: std::str::from_utf8(&bytes[0..21]).unwrap(),
            makeup: RomMakeup::from_byte(bytes[21]),
            kind: RomKind::Rom,
            rom_size: bytes[23],
            sram_size: bytes[24],
            creator_id: bytes[25],
            version: bytes[27],
            checksum_complement: bytes[28],
            checksum: bytes[29],
            native_interrupts: InterruptVector::from_bytes(&bytes[native_start..=native_end]),
            emulation_interrupts: InterruptVector::from_bytes(&bytes[emulation_start..=emulation_end]),
        }
    }
}

impl RomMakeup {
    pub fn from_byte(byte: u8) -> RomMakeup {
        match byte {
            0x20 => RomMakeup::LoRom,
            _ => unimplemented!()
        }
    }
}

impl InterruptVector {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        macro_rules! offset_of {
            ($offset:expr) => {
               u16::from_le_bytes([bytes[2 * $offset], bytes[2 * $offset + 1]]) 
            };
        }

        InterruptVector {
            cop  : offset_of!(0),
            brk  : offset_of!(1),
            abort: offset_of!(2),
            nmi  : offset_of!(3),
            reset: offset_of!(4),
            irq  : offset_of!(5),
        }
    }
}