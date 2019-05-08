use crate::memory::Memory;

#[derive(Default)]
pub struct Serial {}

impl Memory for Serial {
    fn read(&self, address: u16) -> u8 {
        0
    }
    fn write(&mut self, address: u16, value: u8) {}
}
