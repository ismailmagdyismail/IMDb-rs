use crate::core::checksum::check_sum::CheckSum;

pub struct ParityChecksum;

impl CheckSum for ParityChecksum {
    fn calculate(&self, bytes: &[u8]) -> u32 {
        return calculate_parity_bit(bytes) as u32;
    }
}

pub fn calculate_parity_bit(bytes: &[u8]) -> u8 {
    let mut ones_count = 0;
    for byte in bytes {
        ones_count += byte.count_ones();
    }
    ((ones_count & 1) == 0) as u8
}

#[cfg(test)]
mod test {
    use crate::core::checksum::{
        check_sum::CheckSum,
        parity::{ParityChecksum, calculate_parity_bit},
    };

    #[test]
    fn test_odd_count() {
        let bytes: &[u8] = &[1u32.to_le_bytes(), 1u32.to_le_bytes(), 1u32.to_le_bytes()].concat();
        let res = calculate_parity_bit(bytes);
        assert_eq!(res, 0);
    }

    #[test]
    fn test_even_count() {
        let bytes: &[u8] = &[1u32.to_le_bytes(), 1u32.to_le_bytes()].concat();
        let res = calculate_parity_bit(bytes);
        assert_eq!(res, 1);
    }

    #[test]
    fn test_as_u32() {
        let parity = ParityChecksum;
        let expected = [(1u32.to_le_bytes(), 0), (3u32.to_le_bytes(), 1)];
        for entry in expected {
            let res = parity.calculate(&entry.0);
            assert_eq!(res, entry.1);
        }
    }
}
