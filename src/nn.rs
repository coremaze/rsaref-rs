use std::cmp::Ordering;

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

    pub fn set_digit_count(&mut self, digit_count: usize) {
        if digit_count < self.digits.len() {
            self.digits.drain(digit_count..);
        } else {
            let needed_digits = digit_count - self.digits.len();
            for _ in 0..needed_digits {
                self.digits.push(NNDigit::new(0));
            }
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
    pub fn assign_zero(&mut self) {
        self.digits = vec![NNDigit::new(0); self.digits.len()];
    }

    /* Assigns self = 2^exp */
    pub fn assign_2_exp(&mut self, exp: u32) {
        let offset = (exp / u32::BITS) as usize;
        self.digits = vec![NNDigit::new(0); offset + 1];
        self.digits[offset] = NNDigit::new(1 << (exp % u32::BITS));
    }

    /* Computes result = self + other */
    pub fn add(&self, other: &Self) -> Self {
        assert!(
            self.digits.len() == other.digits.len(),
            "add operation requires operands to be the same length"
        );
        let mut carry = 0;
        let mut result_digits = Vec::<NNDigit>::with_capacity(self.digits.len());

        for (n1, n2) in self.digits.iter().zip(other.digits.iter()) {
            // n1 + carry
            let ai_n = match n1.n.checked_add(carry) {
                None => {
                    // Overflowed
                    n2.n
                }
                Some(n1_plus_carry) => {
                    // Did not overflow
                    match n1_plus_carry.checked_add(n2.n) {
                        None => {
                            // Overflowed
                            carry = 1;
                            n1_plus_carry.wrapping_add(n2.n)
                        }
                        Some(n1_plus_carry_plus_n2) => {
                            // No overflow
                            carry = 0;
                            n1_plus_carry_plus_n2
                        }
                    }
                }
            };

            result_digits.push(NNDigit::new(ai_n));
        }

        assert!(self.digits.len() == result_digits.len());

        Self {
            digits: result_digits,
        }
    }

    pub fn sub(&self, other: &Self) -> Self {
        assert!(
            self.digits.len() == other.digits.len(),
            "sub operation requires operands to be the same length"
        );
        let mut borrow = 0;
        let mut result_digits = Vec::<NNDigit>::with_capacity(self.digits.len());

        for (n1, n2) in self.digits.iter().zip(other.digits.iter()) {
            // n1 - borrow
            let ai_n = match n1.n.checked_sub(borrow) {
                None => {
                    // Underflowed
                    u32::MAX - n2.n
                }
                Some(n1_minus_borrow) => {
                    // Did not underflow
                    match n1_minus_borrow.checked_sub(n2.n) {
                        None => {
                            // Underflowed
                            borrow = 1;
                            n1_minus_borrow.wrapping_sub(n2.n)
                        }
                        Some(n1_minus_borrow_minus_n2) => {
                            // No underflow
                            borrow = 0;
                            n1_minus_borrow_minus_n2
                        }
                    }
                }
            };

            result_digits.push(NNDigit::new(ai_n));
        }

        assert!(self.digits.len() == result_digits.len());

        Self {
            digits: result_digits,
        }
    }

    pub fn mult(&self, other: &Self) -> Self {
        assert!(
            self.digits.len() == other.digits.len(),
            "mult operation requires operands to be the same length"
        );

        let mut accumulator = NNDigits::zero();
        accumulator.set_digit_count(self.digits.len() * 2);

        for (i1, d1) in self.digits.iter().enumerate() {
            for (i2, d2) in other.digits.iter().enumerate() {
                let mut mul: u64 = d1.n as u64 * d2.n as u64;
                let lower_digit_n = (mul & u32::MAX as u64) as u32;
                mul >>= u32::BITS;
                let higher_digit_n = (mul & u32::MAX as u64) as u32;

                let digit_shift = i1 + i2;

                let mut add_digits = NNDigits::zero();
                add_digits.set_digit_count(accumulator.digits.len());
                add_digits.digits[digit_shift].n = lower_digit_n;
                add_digits.digits[digit_shift + 1].n = higher_digit_n;

                accumulator = accumulator.add(&add_digits);
            }
        }

        accumulator.set_digit_count(self.digits.len());

        accumulator
    }

    pub fn lshift(&self, bits: usize) -> Self {
        assert!(bits < u32::BITS as usize);

        let mut result = NNDigits::zero();
        result.set_digit_count(self.digits.len());

        let mut shifted: u32 = 0;
        for (source, dest) in self.digits.iter().zip(result.digits.iter_mut()) {
            dest.n = (source.n << bits) | shifted;
            shifted = source.n >> (u32::BITS - bits as u32);
        }

        result
    }

    pub fn rshift(&self, bits: usize) -> Self {
        assert!(bits < u32::BITS as usize);

        let mut result = NNDigits::zero();
        result.set_digit_count(self.digits.len());

        let mut shifted: u32 = 0;
        for (source, dest) in self.digits.iter().zip(result.digits.iter_mut()).rev() {
            dest.n = (source.n >> bits) | shifted;
            shifted = source.n << (u32::BITS - bits as u32);
        }

        result
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        assert!(self.digits.len() == other.digits.len());
        for (d1, d2) in self.digits.iter().zip(other.digits.iter()).rev() {
            if d1 > d2 {
                return Ordering::Greater;
            }
            if d1 < d2 {
                return Ordering::Less;
            }
        }

        Ordering::Equal
    }

    pub fn divmod(&self, other: &Self) -> (Self, Self) {
        assert!(self.digits.len() == other.digits.len());
        let mut zero = Self::zero();
        zero.set_digit_count(self.digits.len());
        assert!(other.compare(&zero) != Ordering::Equal);

        let mut divisor = self.clone();
        divisor.set_digit_count(divisor.digits.len() * 2);

        let mut dividend = other.clone();
        dividend.set_digit_count(dividend.digits.len() * 2);

        let mut quotient = zero.clone();
        quotient.set_digit_count(quotient.digits.len() * 2);

        for i in (0..self.digits.len() * u32::BITS as usize).rev() {
            let mut nn_bit = NNDigits::zero();
            nn_bit.assign_2_exp(i as u32);
            nn_bit.set_digit_count(divisor.digits.len());

            let quotient_high = quotient.clone().add(&nn_bit);

            let try_mult_high = quotient_high.mult(&dividend);
            if try_mult_high.compare(&divisor).is_le() {
                quotient = quotient_high;
            }
        }

        let mut modulus = divisor.sub(&quotient.mult(&dividend));
        modulus.set_digit_count(self.digits.len());
        quotient.set_digit_count(self.digits.len());

        (quotient, modulus)
    }
}

impl Default for NNDigits {
    fn default() -> Self {
        Self {
            digits: vec![NNDigit::default()],
        }
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

    #[test]
    pub fn test_add1() {
        let operand1 = NNDigits::new(&[NNDigit::new(12345), NNDigit::new(54321)]);
        let operand2 = NNDigits::new(&[NNDigit::new(5555555), NNDigit::new(9999999)]);
        let correct_result = NNDigits::new(&[NNDigit::new(5567900), NNDigit::new(10054320)]);
        let result = operand1.add(&operand2);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_add2() {
        let operand1 = NNDigits::new(&[
            NNDigit::new(0xFFFFFFFF),
            NNDigit::new(0xFFFFFFFF),
            NNDigit::new(0),
        ]);
        let operand2 = NNDigits::new(&[NNDigit::new(1), NNDigit::new(0), NNDigit::new(0)]);
        let correct_result = NNDigits::new(&[NNDigit::new(0), NNDigit::new(0), NNDigit::new(1)]);
        let result = operand1.add(&operand2);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_add3() {
        let operand1 = NNDigits::new(&[NNDigit::new(0xFFFFFFFF), NNDigit::new(1)]);
        let operand2 = NNDigits::new(&[NNDigit::new(0xFFFFFFFF), NNDigit::new(0)]);
        let correct_result = NNDigits::new(&[NNDigit::new(4294967294), NNDigit::new(2)]);
        let result = operand1.add(&operand2);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_add4() {
        let operand1 = NNDigits::new(&[NNDigit::new(1), NNDigit::new(1)]);
        let operand2 = NNDigits::new(&[NNDigit::new(0xFFFFFFFF), NNDigit::new(0xFFFFFFFF)]);
        let correct_result = NNDigits::new(&[NNDigit::new(0), NNDigit::new(1)]);
        let result = operand1.add(&operand2);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_set_digit_count1() {
        let mut operand = NNDigits::new(&[NNDigit::new(123), NNDigit::new(321)]);
        let correct_result = NNDigits::new(&[
            NNDigit::new(123),
            NNDigit::new(321),
            NNDigit::new(0),
            NNDigit::new(0),
        ]);
        operand.set_digit_count(4);
        assert_eq!(operand.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_set_digit_count2() {
        let mut operand = NNDigits::new(&[NNDigit::new(123), NNDigit::new(321)]);
        let correct_result = NNDigits::new(&[NNDigit::new(123)]);
        operand.set_digit_count(1);
        assert_eq!(operand.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_sub1() {
        let operand1 = NNDigits::new(&[NNDigit::new(1), NNDigit::new(1)]);
        let operand2 = NNDigits::new(&[NNDigit::new(0xFFFFFFFF), NNDigit::new(0xFFFFFFFF)]);
        let correct_result = NNDigits::new(&[NNDigit::new(2), NNDigit::new(1)]);
        let result = operand1.sub(&operand2);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_sub2() {
        let operand1 = NNDigits::new(&[NNDigit::new(12345), NNDigit::new(12)]);
        let operand2 = NNDigits::new(&[NNDigit::new(100), NNDigit::new(1)]);
        let correct_result = NNDigits::new(&[NNDigit::new(12245), NNDigit::new(11)]);
        let result = operand1.sub(&operand2);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_sub3() {
        let operand1 = NNDigits::new(&[NNDigit::new(12345), NNDigit::new(0xFFFFFFFF)]);
        let operand2 = NNDigits::new(&[NNDigit::new(0xFFFFFFFF), NNDigit::new(1)]);
        let correct_result = NNDigits::new(&[NNDigit::new(12346), NNDigit::new(4294967293)]);
        let result = operand1.sub(&operand2);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_sub4() {
        let operand1 = NNDigits::new(&[NNDigit::new(1), NNDigit::new(0)]);
        let operand2 = NNDigits::new(&[NNDigit::new(0xFFFFFFFF), NNDigit::new(0xFFFFFFFF)]);
        let correct_result = NNDigits::new(&[NNDigit::new(2), NNDigit::new(0)]);
        let result = operand1.sub(&operand2);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_mult1() {
        let operand1 = NNDigits::new(&[NNDigit::new(123)]);
        let operand2 = NNDigits::new(&[NNDigit::new(456)]);
        let correct_result = NNDigits::new(&[NNDigit::new(56088)]);
        let result = operand1.mult(&operand2);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_mult2() {
        let operand1 = NNDigits::new(&[
            NNDigit::new(1),
            NNDigit::new(0xFF),
            NNDigit::new(123),
            NNDigit::new(0),
            NNDigit::new(0),
            NNDigit::new(0),
        ]);
        let operand2 = NNDigits::new(&[
            NNDigit::new(0xFFFFFFFF),
            NNDigit::new(0xFFFFFFFF),
            NNDigit::new(0x666),
            NNDigit::new(0),
            NNDigit::new(0),
            NNDigit::new(0),
        ]);
        let correct_result = NNDigits::new(&[
            NNDigit::new(4294967295),
            NNDigit::new(4294967040),
            NNDigit::new(1515),
            NNDigit::new(417945),
            NNDigit::new(201597),
            NNDigit::new(0),
        ]);
        let result = operand1.mult(&operand2);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_mult3() {
        let operand1 = NNDigits::new(&[
            NNDigit::new(1),
            NNDigit::new(0),
            NNDigit::new(0xF0F0F0F0),
            NNDigit::new(0),
            NNDigit::new(0),
            NNDigit::new(0),
        ]);
        let operand2 = NNDigits::new(&[
            NNDigit::new(0xFFFFFFFF),
            NNDigit::new(0xFFFFFFFF),
            NNDigit::new(0x10),
            NNDigit::new(0),
            NNDigit::new(0),
            NNDigit::new(0),
        ]);
        let correct_result = NNDigits::new(&[
            NNDigit::new(4294967295),
            NNDigit::new(4294967295),
            NNDigit::new(252645152),
            NNDigit::new(4294967295),
            NNDigit::new(4294967279),
            NNDigit::new(15),
        ]);
        let result = operand1.mult(&operand2);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_mult4() {
        let operand1 = NNDigits::new(&[
            NNDigit::new(0xFFFFFFFF),
            NNDigit::new(0xFFFFFFFF),
            NNDigit::new(0xFFFFFFFF),
            NNDigit::new(0),
            NNDigit::new(0),
            NNDigit::new(0),
        ]);
        let operand2 = NNDigits::new(&[
            NNDigit::new(0xFFFFFFFF),
            NNDigit::new(0xFFFFFFFF),
            NNDigit::new(0xFFFFFFFF),
            NNDigit::new(0),
            NNDigit::new(0),
            NNDigit::new(0),
        ]);
        let correct_result = NNDigits::new(&[
            NNDigit::new(1),
            NNDigit::new(0),
            NNDigit::new(0),
            NNDigit::new(4294967294),
            NNDigit::new(4294967295),
            NNDigit::new(4294967295),
        ]);
        let result = operand1.mult(&operand2);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_lshift1() {
        let operand = NNDigits::new(&[NNDigit::new(1)]);
        let bits = 3;
        let correct_result = NNDigits::new(&[NNDigit::new(8)]);
        let result = operand.lshift(bits);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_lshift2() {
        let operand = NNDigits::new(&[NNDigit::new(1)]);
        let bits = 31;
        let correct_result = NNDigits::new(&[NNDigit::new(2147483648)]);
        let result = operand.lshift(bits);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_lshift3() {
        let operand = NNDigits::new(&[NNDigit::new(123), NNDigit::new(456)]);
        let bits = 16;
        let correct_result = NNDigits::new(&[NNDigit::new(8060928), NNDigit::new(29884416)]);
        let result = operand.lshift(bits);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_lshift4() {
        let operand = NNDigits::new(&[NNDigit::new(0x80000000), NNDigit::new(0)]);
        let bits = 1;
        let correct_result = NNDigits::new(&[NNDigit::new(0), NNDigit::new(1)]);
        let result = operand.lshift(bits);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_rshift1() {
        let operand = NNDigits::new(&[NNDigit::new(123)]);
        let bits = 4;
        let correct_result = NNDigits::new(&[NNDigit::new(7)]);
        let result = operand.rshift(bits);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_rshift2() {
        let operand = NNDigits::new(&[NNDigit::new(123), NNDigit::new(456), NNDigit::new(789)]);
        let bits = 15;
        let correct_result = NNDigits::new(&[
            NNDigit::new(59768832),
            NNDigit::new(103415808),
            NNDigit::new(0),
        ]);
        let result = operand.rshift(bits);
        assert_eq!(result.cmp(&correct_result), Ordering::Equal);
    }

    #[test]
    pub fn test_divmod1() {
        let divisor = NNDigits::new(&[NNDigit::new(123)]);
        let dividend = NNDigits::new(&[NNDigit::new(2)]);
        let correct_quotient = NNDigits::new(&[NNDigit::new(61)]);
        let correct_mod = NNDigits::new(&[NNDigit::new(1)]);

        let (quotient, modulus) = divisor.divmod(&dividend);
        println!("{quotient:?}");
        println!("{modulus:?}");
        assert_eq!(quotient.cmp(&correct_quotient), Ordering::Equal);
        assert_eq!(modulus.cmp(&correct_mod), Ordering::Equal);
    }

    #[test]
    pub fn test_divmod2() {
        let divisor = NNDigits::new(&[NNDigit::new(0), NNDigit::new(1)]);
        let dividend = NNDigits::new(&[NNDigit::new(4), NNDigit::new(0)]);
        let correct_quotient = NNDigits::new(&[NNDigit::new(1073741824), NNDigit::new(0)]);
        let correct_mod = NNDigits::new(&[NNDigit::new(0), NNDigit::new(0)]);

        let (quotient, modulus) = divisor.divmod(&dividend);
        assert_eq!(quotient.cmp(&correct_quotient), Ordering::Equal);
        assert_eq!(modulus.cmp(&correct_mod), Ordering::Equal);
    }

    #[test]
    pub fn test_divmod3() {
        let divisor = NNDigits::new(&[
            NNDigit::new(123),
            NNDigit::new(456),
            NNDigit::new(789),
            NNDigit::new(123),
            NNDigit::new(456),
            NNDigit::new(789),
        ]);
        let dividend = NNDigits::new(&[
            NNDigit::new(1),
            NNDigit::new(2),
            NNDigit::new(3),
            NNDigit::new(0),
            NNDigit::new(0),
            NNDigit::new(0),
        ]);
        let correct_quotient = NNDigits::new(&[
            NNDigit::new(2386093233),
            NNDigit::new(2863311499),
            NNDigit::new(4294967272),
            NNDigit::new(262),
            NNDigit::new(0),
            NNDigit::new(0),
        ]);
        let correct_mod = NNDigits::new(&[
            NNDigit::new(1908874186),
            NNDigit::new(954437082),
            NNDigit::new(2),
            NNDigit::new(0),
            NNDigit::new(0),
            NNDigit::new(0),
        ]);

        let (quotient, modulus) = divisor.divmod(&dividend);
        assert_eq!(quotient.cmp(&correct_quotient), Ordering::Equal);
        assert_eq!(modulus.cmp(&correct_mod), Ordering::Equal);
    }
}
