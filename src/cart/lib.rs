pub fn get_rom_banks(hex: u8) -> u16 {
    match hex {
        0x00 => 2,
        0x01 => 4,
        0x02 => 8,
        0x03 => 16,
        0x04 => 32,
        0x05 => 64,
        0x06 => 128,
        0x07 => 256,
        0x08 => 512,
        _ => 0,
    }
}

pub fn get_ram_banks(hex: u8) -> u8 {
    match hex {
        0x00 => 0,
        0x01 | 0x02 => 1,
        0x03 => 4,
        0x04 => 16,
        0x05 => 8,
        _ => 0,
    }
}
