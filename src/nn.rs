#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NNDigit {
    n: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NNDigits {
    digits: Vec<NNDigit>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NNHalfDigit {
    n: u16,
}

impl NNDigits {
    pub fn new(digits: &[NNDigit]) -> Self {
        Self {
            digits: digits.to_vec(),
        }
    }

    pub fn zero() -> Self {
        Self {
            digits: vec![NNDigit::new(0)],
        }
    }

    /* Decodes character string b into the result, where character string is ordered
    from most to least significant. */
    pub fn decode(b: &[u8]) -> Self {
        let mut digits = Vec::<NNDigit>::new();

        let mut current_digit = NNDigit::new(0);
        let mut bit_offset = 0;
        for byte in b.iter().rev() {
            if bit_offset >= u32::BITS {
                digits.push(current_digit);
                bit_offset = 0;
                current_digit = NNDigit::new(0);
            }
            current_digit.n |= (*byte as u32) << bit_offset;
            bit_offset += 8;
        }

        digits.push(current_digit);

        Self { digits }
    }

    /* Encodes digits into character string result, where character string is ordered
    from most to least significant. */
    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::<u8>::new();
        result.reserve(self.digits.len() * (u32::BITS / u8::BITS) as usize);

        for digit in self.digits.iter().rev() {
            let mut n = digit.n;
            for _ in 0..(u32::BITS / u8::BITS) {
                // Get uppermost byte of n
                let top_byte = n & ((u8::MAX as u32) << (u32::BITS - u8::BITS));

                // move the uppermost byte down to the least significant byte
                let byte = ((top_byte >> (u32::BITS - u8::BITS)) % (u8::MAX as u32 + 1)) as u8;

                // Move the next byte in n to the top
                n <<= u8::BITS;

                result.push(byte);
            }
        }

        result
    }

    /* Assigns self = other */
    pub fn assign(&mut self, other: &Self) {
        self.digits = other.digits.clone();
    }

    /* Assigns self = 0 */
    pub fn assign_zero(&mut self)  {
        self.digits = vec![NNDigit::new(0)];
    }

    /* Assigns self = 2^exp */
    pub fn assign_2_exp(&mut self, exp: u32) {
        let offset = (exp / u32::BITS) as usize;
        self.digits = vec![NNDigit::new(0); offset + 1];
        self.digits[offset] = NNDigit::new(1 << (exp % u32::BITS));
    }
}

impl Default for NNDigits {
    fn default() -> Self {
        Self { digits: vec![NNDigit::default()] }
    }
}

impl NNDigit {
    pub fn new(n: u32) -> Self {
        Self { n }
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
        let correct_digits = NNDigits::new(&[NNDigit::new(84281096), NNDigit::new(16909060)]);
        let digits = NNDigits::decode(&bytes);

        assert_eq!(digits.cmp(&correct_digits), Ordering::Equal);
    }

    #[test]
    pub fn test_decode2() {
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let correct_digits = NNDigits::new(&[
            NNDigit::new(117967114),
            NNDigit::new(50595078),
            NNDigit::new(258),
        ]);
        let digits = NNDigits::decode(&bytes);

        assert_eq!(digits.cmp(&correct_digits), Ordering::Equal);
    }

    #[test]
    pub fn test_decode3() {
        let bytes = [1];
        let correct_digits = NNDigits::new(&[NNDigit::new(1)]);
        let digits = NNDigits::decode(&bytes);

        assert_eq!(digits.cmp(&correct_digits), Ordering::Equal);
    }

    #[test]
    pub fn test_encode1() {
        let original_digits = NNDigits::new(&[
            NNDigit::new(117967114),
            NNDigit::new(50595078),
            NNDigit::new(258),
        ]);

        let encoded_bytes = original_digits.encode();
        let decoded_digits = NNDigits::decode(&encoded_bytes);

        assert_eq!(original_digits.cmp(&decoded_digits), Ordering::Equal);
    }

    #[test]
    pub fn test_encode2() {
        let original_digits = NNDigits::new(&[NNDigit::new(84281096), NNDigit::new(16909060)]);

        let encoded_bytes = original_digits.encode();
        let decoded_digits = NNDigits::decode(&encoded_bytes);

        assert_eq!(original_digits.cmp(&decoded_digits), Ordering::Equal);
    }

    #[test]
    pub fn test_assign_2_exp() {
        let mut num = NNDigits::default();
        num.assign_2_exp(123);

        let correct_digits = NNDigits::new(&[
            NNDigit::new(0),
            NNDigit::new(0),
            NNDigit::new(0),
            NNDigit::new(134217728),
        ]);

        assert_eq!(num.cmp(&correct_digits), Ordering::Equal);
    }
}
