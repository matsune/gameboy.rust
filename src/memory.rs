use crate::util::{get_lsb, get_msb};

pub trait Memory {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);

    fn read_word(&self, address: u16) -> u16 {
        let lsb = self.read(address);
        let msb = self.read(address + 1);
        u16::from(msb) << 8 | u16::from(lsb)
    }

    fn write_word(&mut self, address: u16, value: u16) {
        self.write(address, get_lsb(value));
        self.write(address + 1, get_msb(value));
    }
}
