
#[derive(Default)]
pub struct CpuRegisters {
    pub accumulator: u16,
    pub index_x: u16,
    pub index_y: u16,
    pub stack_pointer: u16,

    pub data_bank: u8,
    pub direct_page: u16,

    pub program_bank: u8,
    pub program_counter: u16,

    pub processor_status: ProcessorStatus,

    pub emulation: bool
}

macro_rules! impl_register {
    (register $register:ty {type=$type:ty; $(($offset:expr, $get:ident, $set:ident);)+}) => {
        impl $register {
            $(
                pub fn $get(&self) -> bool {
                    (self.0 & (1 << $offset)) != 0
                }

                pub fn $set(&mut self, state: bool) {
                    self.0 |= (state as $type) << $offset;
                }
            )+
        }
    }
}


#[derive(Default)]
pub struct ProcessorStatus(pub u8);

impl_register! (
    register ProcessorStatus {
        type = u8;

        (0,  get_carry,        set_carry);
        (1,  get_zero,         set_zero);
        (2,  get_irq,          set_irq);
        (3,  get_decimal,      set_decimal);
        (4,  get_index,        set_index);
        (5,  get_accumulator,  set_accumulator);
        (6,  get_overflow,     set_overflow);
        (7,  get_negative,     set_negative);
    }
);


