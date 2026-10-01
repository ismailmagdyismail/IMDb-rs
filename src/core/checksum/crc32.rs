use crc::{CRC_32_ISO_HDLC, Crc};

use crate::core::checksum::check_sum::CheckSum;

pub struct Crc32CheckSum {
    crc_calculator: Crc<u32>,
}

impl Crc32CheckSum {
    pub fn new() -> Crc32CheckSum {
        let crc = Crc::<u32>::new(&CRC_32_ISO_HDLC);
        return Crc32CheckSum {
            crc_calculator: crc,
        };
    }
}

impl CheckSum for Crc32CheckSum {
    fn calculate(&self, bytes: &[u8]) -> u32 {
        self.crc_calculator.checksum(bytes)
    }
}

#[cfg(test)]
mod test {
    use crate::core::checksum::{check_sum::CheckSum, crc32::Crc32CheckSum};

    #[test]
    fn test_crc_determinism() {
        let crc_calc = Crc32CheckSum::new();
        let bytes = [1u32.to_le_bytes(), 2u32.to_le_bytes()].concat();
        let v1 = crc_calc.calculate(&bytes);
        let v2 = crc_calc.calculate(&bytes);
        assert_eq!(v1, v2)
    }
}
