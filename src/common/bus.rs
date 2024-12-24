pub trait Bus {
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
