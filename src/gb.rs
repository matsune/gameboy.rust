use crate::cpu::CPU;

#[derive(Debug)]
pub struct Gameboy {
    cpu: CPU,
}

impl Gameboy {
    pub fn new() -> Self {
        Self { cpu: CPU::new() }
    }
}
