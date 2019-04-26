use crate::reg::Registers;

#[derive(Debug)]
pub struct CPU {
    registers: Registers,
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: Registers::default(),
        }
    }
}
