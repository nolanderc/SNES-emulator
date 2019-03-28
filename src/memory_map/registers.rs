//! See https://wiki.superfamicom.org/registers#toc-83

pub trait RegisterMap<T> {
    /// Get the value of the register
    fn get(&self) -> u8;

    /// Get a mutable borrow of the register's value
    fn get_mut(&mut self) -> &mut u8;
}

macro_rules! impl_register_map {
    ( $struct:ident { type: $reg:ty, name: $name:ident } ) => {
        impl RegisterMap<$reg> for $struct {
            fn get(&self) -> u8 {
                self.$name.0
            }

            fn get_mut(&mut self) -> &mut u8 {
                &mut self.$name.0
            }
        }
    }
}

macro_rules! define_registers {
    { $struct:ident {$($name:ident: $reg:ident),*} } => {
        #[derive(Default)]
        pub struct $struct {
            $(
                pub $name: $reg
            ),*
        }

        $(
            #[derive(Default)] 
            pub struct $reg(pub u8);

            impl_register_map!( $struct { type: $reg, name: $name });
        )*
    }
}


