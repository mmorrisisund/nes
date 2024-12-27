pub trait Bus {
    fn reset(&mut self);

    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);

    fn read_u16(&self, address: u16) -> u16 {
        let lo = self.read(address) as u16;
        let hi = self.read(address + 1) as u16;

        (hi << 8) | lo
    }

    fn write_u16(&mut self, address: u16, data: u16) {
        let lo = data as u8;
        let hi = (data >> 8) as u8;

        self.write(address, lo);
        self.write(address + 1, hi);
    }
}

// TODO: Add visibility modifiers?
macro_rules! mem_mapped_registers {
    (enum $name:ident { $($field:ident = $field_val:expr,)*}) => {
        #[derive(Debug)]
        enum $name {
            $($field = $field_val,)*
        }

        impl TryFrom<u16> for $name {
            type Error = ();

            fn try_from(value: u16) -> Result<Self, Self::Error> {
                match value {
                    $(x if x == $name::$field as u16 => Ok($name::$field),)*
                    _ => Err(()),
                }
            }
        }
    };
}
