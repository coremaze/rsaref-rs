use crate::RSAError;
use md5::{Digest, Md5};

const RANDOM_BYTES_NEEDED: usize = 256;

#[derive(Debug)]
pub struct RandomStruct {
    bytes_needed: usize,
    state: [u8; 16],
    output_available: usize,
    output: [u8; 16],
}

impl RandomStruct {
    pub fn new() -> Self {
        Self {
            bytes_needed: RANDOM_BYTES_NEEDED,
            state: [0u8; 16],
            output_available: 0,
            output: [0u8; 16],
        }
    }

    pub fn random_init(&mut self) {
        self.bytes_needed = RANDOM_BYTES_NEEDED;
        self.state.fill(0);
        self.output_available = 0;
    }

    pub fn random_update(&mut self, block: &[u8]) {
        let mut context = Md5::new();
        context.update(block);
        let digest: [u8; 16] = context.finalize().into();

        /* add digest to state */
        let mut x: u32 = 0;
        for (state_byte, digest_byte) in self.state.iter_mut().zip(digest) {
            x += *state_byte as u32 + digest_byte as u32;
            *state_byte = (x & 0xFF) as u8;
            x >>= 8;
        }

        self.bytes_needed = self.bytes_needed.saturating_sub(block.len());
    }

    pub fn get_random_bytes_needed(&self) -> usize {
        self.bytes_needed
    }

    pub fn generate_bytes(&mut self, mut block_len: usize) -> Result<Vec<u8>, RSAError> {
        if self.bytes_needed != 0 {
            return Err(RSAError::NeedRandom);
        }

        let mut available: usize = self.output_available;

        let mut block: Vec<u8> = Vec::with_capacity(block_len);

        while block_len > available {
            block.extend_from_slice(&self.output[(self.output.len() - available)..]);
            block_len -= available;

            /* generate new output */
            let mut context = Md5::new();
            context.update(&self.state);
            self.output = context.finalize().into();
            available = self.output.len();

            /* increment state */
            for state in self.state.iter_mut().rev() {
                let was_zero = *state == 0;

                *state = state.wrapping_add(1);

                if !was_zero {
                    break;
                }
            }
        }

        let rest_block_start = self.output.len() - available;
        block.extend_from_slice(&self.output[rest_block_start..(rest_block_start + block_len)]);
        self.output_available = available - block_len;

        Ok(block)
    }

    pub fn random_final(&mut self) {
        self.bytes_needed = 0;
        self.state.fill(0);
        self.output_available = 0;
        self.output.fill(0);
    }
}

impl Default for RandomStruct {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_random_bytes1() {
        use std::cmp::Ordering;

        let mut random_struct = RandomStruct::new();
        let random_buf = (0..=255).collect::<Vec<u8>>();
        random_struct.random_update(&random_buf);

        // output based on reference C implementation
        let correct_output = [
            228, 175, 223, 214, 41, 129, 0, 155, 170, 166, 121, 35, 162, 43, 33, 128, 160, 243,
            114, 7, 151, 239, 226, 136, 33, 211, 27, 198, 6, 67, 81, 58, 144, 153, 107, 102, 82,
            197, 87, 249, 67, 193, 15, 136, 73, 133, 20, 150, 158, 10, 240, 157, 163, 134, 162, 41,
            220, 113, 234, 241, 137, 33, 118, 245, 226, 186, 194, 180, 96, 150, 34, 250, 211, 159,
            3, 37, 70, 244, 46, 5, 202, 36, 86, 178, 108, 126, 170, 92, 237, 197, 98, 134, 146, 1,
            157, 109, 254, 8, 162, 50, 21, 18, 83, 25, 12, 59, 212, 63, 219, 66, 228, 35, 60, 49,
            96, 176, 69, 8, 34, 1, 197, 15, 219, 104, 245, 209, 237, 212, 70, 134, 88, 173, 211,
            100, 153, 147, 14, 147, 82, 228, 109, 213, 144, 185, 242, 8, 43, 107, 43, 90, 170, 190,
            0, 74, 157, 117, 35, 51, 15, 87, 233, 47, 75, 156, 190, 113, 108, 215, 176, 11, 207,
            166, 139, 243, 226, 203, 200, 112, 99, 200, 88, 223, 114, 178, 107, 33, 29, 0, 53, 0,
            171, 160, 196, 231, 94, 231, 62, 238, 230, 104, 76, 163, 194, 162, 28, 149, 109, 60,
            178, 27, 104, 142, 246, 27, 58, 218, 142, 250, 126, 214, 248, 228, 71, 253, 159, 228,
            77, 147, 212, 168, 20, 127, 252, 238, 144, 118, 179, 169, 177, 31, 168, 50, 75, 177,
            43, 176, 172, 125, 15, 120, 153, 88, 37, 3, 141, 168,
        ];

        match random_struct.generate_bytes(256) {
            Ok(random_bytes) => {
                assert_eq!(random_bytes.cmp(&correct_output.to_vec()), Ordering::Equal);
            }
            Err(_) => {
                assert!(false, "generate_bytes returned an error");
            }
        }
    }

    #[test]
    fn test_random_bytes2() {
        use std::cmp::Ordering;

        let mut random_struct = RandomStruct::new();
        let random_buf = (0..=255).rev().collect::<Vec<u8>>();
        random_struct.random_update(&random_buf);

        // output based on reference C implementation
        let correct_output = [
            232, 185, 23, 232, 237, 125, 183, 144, 177, 65, 7, 180, 228, 117, 195, 232, 242, 214,
            237, 200, 33, 44, 215, 119, 171, 226, 106, 110, 153, 111, 167, 172, 119, 21, 207, 99,
            27, 42, 207, 77, 24, 33, 229, 238, 7, 189, 199, 180, 17, 235, 224, 158, 252, 115, 239,
            180, 105, 217, 178, 129, 83, 182, 175, 237, 62, 40, 31, 85, 36, 220, 92, 167, 69, 77,
            180, 219, 87, 70, 142, 192, 72, 46, 47, 96, 169, 218, 147, 7, 37, 20, 179, 253, 119,
            208, 134, 127, 252, 174, 137, 28, 175, 176, 183, 13, 16, 122, 115, 179, 166, 64, 131,
            154, 240, 77, 204, 209, 155, 61, 21, 174, 234, 14, 147, 116, 145, 41, 150, 214, 14,
            102, 62, 9, 233, 131, 211, 10, 135, 231, 207, 248, 159, 35, 255, 99, 80, 196, 32, 99,
            88, 191, 131, 102, 200, 67, 6, 179, 92, 200, 39, 147, 248, 62, 35, 135, 28, 242, 63,
            79, 44, 121, 27, 20, 160, 151, 238, 80, 246, 85, 131, 151, 255, 233, 193, 23, 125, 25,
            10, 184, 38, 89, 26, 204, 64, 41, 145, 0, 23, 52, 105, 155, 162, 52, 144, 92, 210, 27,
            62, 168, 109, 83, 1, 115, 94, 9, 73, 88, 20, 71, 24, 13, 220, 53, 68, 76, 232, 198,
            240, 111, 54, 225, 232, 5, 145, 200, 217, 25, 80, 250, 228, 24, 48, 131, 220, 56, 84,
            153, 156, 60, 93, 250, 70, 175, 134, 193, 82, 252,
        ];

        match random_struct.generate_bytes(256) {
            Ok(random_bytes) => {
                assert_eq!(random_bytes.cmp(&correct_output.to_vec()), Ordering::Equal);
            }
            Err(_) => {
                assert!(false, "generate_bytes returned an error");
            }
        }
    }
}
