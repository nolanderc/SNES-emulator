use crate::{memory_map::*, *};

mod registers;
use registers::*;

/// Rioch 5A22 CPU, executes uses 65C816 assembly
pub struct Cpu {
    native_interrupts: InterruptVector,
    emulation_interrupts: InterruptVector,
    registers: CpuRegisters,
}

impl Cpu {
    pub(crate) fn new(memory: &MemoryMap) -> Self {
        let SnesHeader {
            native_interrupts,
            emulation_interrupts,
            ..
        } = memory.get_snes_header();

        Cpu {
            native_interrupts,
            emulation_interrupts,
            registers: CpuRegisters::default(),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.registers.program_counter = self.emulation_interrupts.reset;
        self.registers.program_bank = 0;
        self.registers.stack_pointer = 0x0100;
        self.registers.index_x = 0;
        self.registers.index_y = 0;
        self.registers.accumulator = 0;
        self.registers.emulation = true;
        self.registers.processor_status.set_accumulator(true);
        self.registers.processor_status.set_index(true);
        self.registers.processor_status.set_decimal(false);
        self.registers.processor_status.set_irq(true);
        self.registers.processor_status.set_carry(true);
    }

    pub(crate) fn tick(&mut self, memory: &mut MemoryMap) {
        let instruction = self.fetch_instruction(memory);
        self.advance(instruction.size());
        self.execute(instruction, memory);
    }

    // ======================== //
    // Get instruction argument //
    // ======================== //

    /// Fetch the next instruction pointed to by the program counter and bank.
    fn fetch_instruction(&self, memory: &MemoryMap) -> Instruction {
        let opcode = self.get_instruction_arg(memory, 0);

        use Instruction::*;
        match opcode {
            0x18 => ClearCarry,

            0x4c => Jump(self.get_arg_absolute(memory)),
            0x5c => Jump(self.get_arg_absolute_long(memory)),

            0x78 => DisableInterruptRequests,

            0x9c => StoreZero(self.get_arg_absolute(memory)),

            0x8d => StoreAccumulator(self.get_arg_absolute(memory)),

            0xa9 if self.registers.processor_status.get_accumulator() => {
                LoadAccumulator(self.get_arg_immediate_8bit(memory))
            }
            0xa9 => LoadAccumulator(self.get_arg_immediate_16bit(memory)),

            0xc2 => ResetStatusFlags(self.get_instruction_arg(memory, 1)),
            0xfb => ExchangeCarryEmulator,

            _ => unimplemented!("opcode not implemented: {:x}", opcode),
        }
    }

    fn get_instruction_arg(&self, memory: &MemoryMap, delta: u16) -> u8 {
        memory.get_byte(
            self.registers.program_bank,
            self.registers.program_counter + delta,
        )
    }

    fn get_arg_absolute(&self, memory: &MemoryMap) -> Address {
        let low = self.get_instruction_arg(memory, 1);
        let high = self.get_instruction_arg(memory, 2);
        let addr = u16::from_le_bytes([low, high]);

        Address::Absolute { addr }
    }

    fn get_arg_absolute_long(&self, memory: &MemoryMap) -> Address {
        let low = self.get_instruction_arg(memory, 1);
        let high = self.get_instruction_arg(memory, 2);
        let addr = u16::from_le_bytes([low, high]);

        let bank = self.get_instruction_arg(memory, 3);

        Address::AbsoluteLong { bank, addr }
    }

    fn get_arg_immediate_8bit(&self, memory: &MemoryMap) -> Address {
        let data = self.get_instruction_arg(memory, 1);
        Address::Immediate8 { data }
    }

    fn get_arg_immediate_16bit(&self, memory: &MemoryMap) -> Address {
        let low = self.get_instruction_arg(memory, 1);
        let high = self.get_instruction_arg(memory, 2);
        let data = u16::from_le_bytes([low, high]);
        Address::Immediate16 { data }
    }
}

// Execute instruction
impl Cpu {
    /// Execute an instruction on the processor
    fn execute(&mut self, instruction: Instruction, memory: &mut MemoryMap) {
        log::trace!("Executing instruction: {:x?}", instruction);

        use Instruction::*;
        match instruction {
            Jump(address) => self.jump(address),

            DisableInterruptRequests => self.registers.processor_status.set_irq(false),

            ClearCarry => self.registers.processor_status.set_carry(false),

            ExchangeCarryEmulator => {
                let carry = self.registers.processor_status.get_carry();
                self.registers.emulation = carry;
            }

            ResetStatusFlags(mask) => {
                log::trace!("REP mask: {:08b}", mask);
                let p = &mut self.registers.processor_status;
                p.0 = !((!p.0) | mask);
            }

            LoadAccumulator(address) => self.load_accumulator(memory, address),

            StoreAccumulator(address) => self.store_accumulator(memory, address),
            StoreZero(address) => self.store(0, memory, address),
        }
    }

    /// Advance the program counter
    fn advance(&mut self, delta: u8) {
        self.registers.program_counter += u16::from(delta);
    }

    /// Converts an instruction's address argument to an absolute raw address.
    fn raw_address(&self, address: Address) -> (u8, u16) {
        use Address::*;
        match address {
            Absolute { addr } => (self.registers.data_bank, addr),
            AbsoluteLong { bank, addr } => (bank, addr),

            Immediate8 { .. } | Immediate16 { .. } => {
                panic!("Attempted to get address of immediate instruction")
            }

            _ => unimplemented!("raw address: {:?}", address),
        }
    }

    /// Returns the data pointed to by an address
    fn get_data(&self, memory: &MemoryMap, address: Address, wide: bool) -> u16 {
        use Address::*;
        match address {
            Immediate8 { data } => data as u16,
            Immediate16 { data } => data,
            _ => {
                let (bank, addr) = self.raw_address(address);
                let low = memory.get_byte(bank, addr);

                if wide {
                    let high = memory.get_byte(bank, addr + 1);
                    u16::from_le_bytes([low, high])
                } else {
                    low as u16
                }
            }
        }
    }

    // ====================== //
    // Implement instructions //
    // ====================== //

    /// Jump to the target address.
    fn jump(&mut self, address: Address) {
        let (bank, addr) = self.raw_address(address);

        self.registers.program_bank = bank;
        self.registers.program_counter = addr;
    }

    fn load_accumulator(&mut self, memory: &MemoryMap, address: Address) {
        let wide = !self.registers.processor_status.get_accumulator();
        self.registers.accumulator = self.get_data(memory, address, wide);
    }

    fn store_accumulator(&mut self, memory: &mut MemoryMap, address: Address) {
        let (bank, addr) = self.raw_address(address);

        let low = self.registers.accumulator & 0xff;
        memory.set_byte(bank, addr, low as u8);

        let wide = !self.registers.processor_status.get_accumulator();
        if wide {
            let high = (self.registers.accumulator & 0xff00) >> 8;
            memory.set_byte(bank, addr + 1, high as u8);
        }
    }

    /// Store a byte in program memory.
    /// Only works if the address points to writable memory (aka, not rom).
    fn store(&mut self, value: u8, memory: &mut MemoryMap, address: Address) {
        let (bank, addr) = self.raw_address(address);

        memory.set_byte(bank, addr, value);
    }
}

#[derive(Debug)]
enum Address {
    /// DBR | addr
    Absolute {
        addr: u16,
    },

    /// PBR | (X+offset)
    AbsoluteIndexedIndirect {
        offset: u16,
    },

    /// DBR | (X+offset)
    AbsoluteIndexed {
        offset: u16,
    },

    /// DBR | (Y+offset)
    AbsoluteIndexedY {
        offset: u16,
    },

    /// 0x00 | addr
    AbsoluteIndirect {
        addr: u16,
    },

    /// bank | addr
    AbsoluteLong {
        bank: u8,
        addr: u16,
    },

    /// bank | (X+addr)
    AbsoluteLongIndexed {
        bank: u8,
        addr: u16,
    },

    /// DBR | A
    Accumulator,

    /// Source Address:      src_bank | X
    /// Destination Address: dst_bank | Y
    BlockMove {
        src_bank: u8,
        dst_bank: u8,
    },

    /// Also known as Indirect X addressing.
    /// DBR | (D+X+offset)
    DirectIndexedIndirect {
        offset: u8,
    },

    /// 0x00 | (D+X+offset)
    DirectIndexed {
        offset: u8,
    },

    /// 0x00 | (D+Y+offset)
    DirectIndexedY {
        offset: u8,
    },

    /// Also known as Indirect Y addressing.
    /// (DBR | (D+offset)) + Y
    DirectIndirectIndexed {
        offset: u8,
    },

    /// (0x00 | (D+offset)) + Y
    DirectIndirectLongIndexed {
        offset: u8,
    },

    /// 0x00 | (D+offset)
    DirectIndirectLong {
        offset: u8,
    },

    /// DBR | (D+offset)
    DirectIndirect {
        offset: u8,
    },

    /// 0x00 | (D+offset)
    Direct {
        offset: u8,
    },

    /// The address is the data
    /// See: http://6502.org/tutorials/65c816opcodes.html#5.14
    Immediate8 {
        data: u8,
    },
    Immediate16 {
        data: u16,
    },

    /// Varies by instructien
    Implied,

    /// PBR | (PC+offset)
    ProgramCounterRelativeLong {
        offset: i16,
    },

    /// PBR | (PC+offset)
    ProgramCounterRelative {
        offset: i8,
    },

    /// 0x00 | ???
    Stack,

    /// 0x00 | (S+offset}
    StackRelative {
        offset: u8,
    },

    /// (DBR | (S+offset}) + Y
    StackRelativeIndirectIndexed {
        offset: u8,
    },
}

#[derive(Debug)]
enum Instruction {
    // ===== //
    // Flags //
    // ===== //

    /// SEI, Disable interrupt requests
    DisableInterruptRequests,

    /// CLC, clear carry bit
    ClearCarry,

    /// XCE, exchange carry and emulator bit
    ExchangeCarryEmulator,

    /// REP, reset status flags
    ResetStatusFlags(u8),

    /// JMP, jump to address
    Jump(Address),

    // ========== //
    // Load/Store //
    // ========== //
    
    /// LDA, load accumulator from memory
    LoadAccumulator(Address),

    /// STA, store accumulator in memory
    StoreAccumulator(Address),

    /// STZ, store zero in memory
    StoreZero(Address),
}

impl Address {
    pub fn arg_size(&self) -> u8 {
        use Address::*;
        match self {
            Absolute { .. } => 2,
            AbsoluteLong { .. } => 3,
            Immediate8 { .. } => 1,
            Immediate16 { .. } => 2,

            _ => unimplemented!("arg_size: {:?}", self),
        }
    }
}

impl Instruction {
    pub fn size(&self) -> u8 {
        use Instruction::*;

        match self {
            Jump(addr) 
                | StoreZero(addr) 
                | LoadAccumulator(addr) 
                | StoreAccumulator(addr) 
                => 1 + addr.arg_size(),

            ResetStatusFlags(_) => 2,

            DisableInterruptRequests | ClearCarry | ExchangeCarryEmulator => 1,
        }
    }
}
