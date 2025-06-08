#[macro_export]
macro_rules! round_up {
    ($val:expr, $align:expr) => {
        (($val as usize + $align as usize - 1) & !($align as usize - 1))        
    };
}

#[macro_export]
macro_rules! round_down {
    ($val:expr, $align:expr) => {
        ($val as usize & !($align as usize - 1))
    };
}

#[macro_export]
macro_rules! sign_extend {
    ($val:expr, $bits:expr) => {
        (($val as i64) << (64 - $bits)) >> (64 - $bits)
    };
}

#[macro_export]
macro_rules! zero_extend {
    ($val:expr, $bits:expr) => {
        ($val as u64) & ((1u64 << $bits) - 1)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_up() {
        assert_eq!(round_up!(5, 4), 8);
        assert_eq!(round_up!(8, 4), 8);
        assert_eq!(round_up!(9, 4), 12);
    }

    #[test]
    fn test_round_down() {
        assert_eq!(round_down!(5, 4), 4);
        assert_eq!(round_down!(8, 4), 8);
        assert_eq!(round_down!(9, 4), 8);
    }

    #[test]
    fn test_sign_extend() {
        assert_eq!(sign_extend!(0b00000001, 1), -1);
        assert_eq!(sign_extend!(0b11111111, 8), -1);
        assert_eq!(sign_extend!(0b00000000_00000000_00000000_00000001, 32), 1);
        assert_eq!(sign_extend!(0b11111111_11111111_11111111_11111111, 32), -1);
        assert_eq!(sign_extend!(0b100000000011111111000, 21), -1046536);
    }

    #[test]
    fn test_zero_extend() {
        assert_eq!(zero_extend!(0b00000001, 1), 1);
        assert_eq!(zero_extend!(0b11111111, 8), 255);
        assert_eq!(zero_extend!(0b00000000_00000000_00000000_00000001, 32), 1);
        assert_eq!(zero_extend!(0b11111111_11111111_11111111_11111111, 32), 4294967295);
    }
}