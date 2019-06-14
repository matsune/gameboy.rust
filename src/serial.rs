use crate::memory::Memory;

#[derive(Default)]
pub struct Serial {
    data: u8,
    control: u8,
}

impl Memory for Serial {
    fn read(&self, a: u16) -> u8 {
        match a {
            0xff01 => self.data,
            0xff02 => self.control,
            _ => panic!(),
        }
    }

    fn write(&mut self, a: u16, v: u8) {
        match a {
            0xff01 => self.data = v,
            0xff02 => self.control = v,
            _ => panic!(),
        }
    }
}
