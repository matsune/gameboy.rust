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

pub fn get_ram_banks_with_bank_size(hex: u8) -> (u8, u16) {
    match hex {
        0x00 => (0, 0),
        0x01 => (1, 2048),
        0x02 => (1, 0x2000),
        0x03 => (4, 0x2000),
        0x04 => (16, 0x2000),
        0x05 => (8, 0x2000),
        _ => (0, 0),
    }
}
