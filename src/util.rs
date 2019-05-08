pub fn get_msb(n: u16) -> u8 {
    (n >> 8) as u8
}

pub fn get_lsb(n: u16) -> u8 {
    (n & 0xff) as u8
}

pub fn is_bit_on(n: u8, pos: u8) -> bool {
    assert!(pos < 8, "Bit out of bounds");
    (n & (1 << pos)) != 0
}

pub fn set_bit(n: &mut u8, pos: u8, b: bool) {
    assert!(pos < 8, "Bit out of bounds");
    *n = if b { *n | (1 << pos) } else { !(1 << pos) & *n };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_msb_lsb() {
        assert_eq!(get_msb(0b_11010010_00101101), 0b_11010010);
        assert_eq!(get_lsb(0b_11010010_00101101), 0b_00101101);
    }

    #[test]
    fn test_get_bit() {
        assert_eq!(is_bit_on(0b_1000_0000, 7), true);
        assert_eq!(is_bit_on(0b_1000_0000, 6), false);
        assert_eq!(is_bit_on(0b_0010_0000, 5), true);
    }

    #[test]
    fn test_set_bit() {
        let mut a = 0b_1000_0000;
        set_bit(&mut a, 7, false);
        assert_eq!(a, 0b_0000_0000);
        set_bit(&mut a, 6, true);
        assert_eq!(a, 0b_0100_0000);
        set_bit(&mut a, 6, false);
        set_bit(&mut a, 1, true);
        assert_eq!(a, 0b_0000_0010);
    }
}
