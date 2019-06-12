// TODO:

use crate::memory::Memory;

#[derive(Default)]
pub struct Serial {}

impl Memory for Serial {
    fn read(&self, _address: u16) -> u8 {
        0
    }
    fn write(&mut self, _address: u16, _value: u8) {}
}
