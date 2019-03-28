
macro_rules! get_access {
    { 
        matching_value = $value:expr,
        store = $store:expr,
        hardware_registers = [$($reg:ident),*]
        $($tt:tt)*
    } => {
        match $value { 
            $(MemoryAccess::$reg => get_access!($reg, $store) ),*
                $($tt)*
        }
    };

    ( $reg:ident, $store:expr ) => {
        RegisterMap::<hardware_registers::$reg>::get($store)
    }
}

macro_rules! get_mut_access {
    {
        matching_value = $value:expr,
        store = $store:expr,
        hardware_registers = [$($reg:ident),*] 
        $($tt:tt)*
    } => {
        match $value { 
            $(MemoryAccess::$reg => get_mut_access!($reg, $store) ),*
                $($tt)*
        }
    };

    ( $reg:ident, $store:expr ) => {
        RegisterMap::<hardware_registers::$reg>::get_mut($store)
    }
}

macro_rules! impl_memory_access {
    {
        get($get_self:ident) {
            hardware_registers = [ $($get_reg:ident),* ]
            $($get_tt:tt)*
        }
        get_mut($mut_self:ident) {
            hardware_registers = [ $($mut_reg:ident),* ]
            $($mut_tt:tt)*
        }
    } => {
        impl<'a> MemoryMap<'a> {
            fn access_byte(&self, access: MemoryAccess) -> u8 {
                let $get_self = self;
                use MemoryAccess::*;
                get_access! {
                    matching_value = access,
                    store = &self.hardware_registers,
                    hardware_registers = [ $($get_reg),* ],
                    $($get_tt)*
                }
            }

            fn access_byte_mut(&mut self, access: MemoryAccess) -> &mut u8 {
                let $mut_self = self;
                use MemoryAccess::*;
                get_mut_access! {
                    matching_value = access,
                    store = &mut $mut_self.hardware_registers,
                    hardware_registers = [ $($mut_reg),* ],
                    $($mut_tt)*
                }
            }
        }
    }
}

macro_rules! impl_memory_mapping {
    {
        $($addr:expr => { $($tt:tt)* }),*
    } => {
        impl<'a> MemoryMap<'a> {
            fn get_hardware_register(addr: u16) -> MemoryAccess {
                match addr {
                    $(
                        $addr => { $($tt)* }
                    ),*
                    _ => unimplemented!("get_hardware_register({:x})", addr)
                }
            }
        }
    }
}

macro_rules! define_memory_access {
    {
        hardware_registers = [ $($addr:expr => $reg:ident ($name:ident)),* ]
        other { $($tt:tt)* }
        get($get_self:ident) { $($get_tt:tt)* }
        get_mut($mut_self:ident) { $($mut_tt:tt)* }
    } => {
        enum MemoryAccess {
            $($reg,)*
            $($tt)*
        }

        impl_memory_access! {
            get($get_self) {
                hardware_registers = [ $($reg),* ]
                $($get_tt)*
            }
            get_mut($mut_self) {
                hardware_registers = [ $($reg),* ]
                $($mut_tt)*
            }
        }

        impl_memory_mapping! {
            $(
                $addr => {MemoryAccess::$reg}
            ),*
        }

        mod hardware_registers {
            use crate::memory_map::registers::*;

            define_registers! {
                HardwareRegisters {
                    $(
                        $name: $reg
                    ),*
                }
            }
        }
    }
}
