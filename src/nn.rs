#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NNDigit {
    n: u32,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NNHalfDigit {
    n: u16,
}

impl NNDigit {
    pub fn new(n: u32) -> Self {
        Self { n }
    }

    /* Decodes character string b into the result, where character string is ordered
    from most to least significant. */
    pub fn decode(b: &[u8]) -> Vec<Self> {
        let mut result = Vec::<Self>::new();

        let mut current_digit = Self::new(0);
        let mut bit_offset = 0;
        for byte in b.iter().rev() {
            if bit_offset >= u32::BITS {
                result.push(current_digit);
                bit_offset = 0;
                current_digit = Self::new(0);
            }
            current_digit.n |= (*byte as u32) << bit_offset;
            bit_offset += 8;
        }

        result.push(current_digit);

        result
    }
}

impl Default for NNDigit {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::*;
    #[test]
    pub fn test_decode1() {
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8];
        let correct_digits = vec![NNDigit::new(84281096), NNDigit::new(16909060)];
        let digits = NNDigit::decode(&bytes);

        assert_eq!(digits.cmp(&correct_digits), Ordering::Equal);
    }

    #[test]
    pub fn test_decode2() {
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let correct_digits = vec![
            NNDigit::new(117967114),
            NNDigit::new(50595078),
            NNDigit::new(258),
        ];
        let digits = NNDigit::decode(&bytes);

        assert_eq!(digits.cmp(&correct_digits), Ordering::Equal);
    }

    #[test]
    pub fn test_decode3() {
        let bytes = [1];
        let correct_digits = vec![NNDigit::new(1)];
        let digits = NNDigit::decode(&bytes);

        assert_eq!(digits.cmp(&correct_digits), Ordering::Equal);
    }
}
