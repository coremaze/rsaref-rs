use std::{
    io::{BufReader, Read},
    ops::{Add, Mul, Sub},
};

use crate::r_random::RandomStruct;
use num_integer::Integer;
use rsa::BigUint;

trait RSASerialize {
    fn to_be(&self, bytes: usize) -> Vec<u8>;
}

impl RSASerialize for BigUint {
    fn to_be(&self, bytes: usize) -> Vec<u8> {
        let le = self.to_bytes_le();
        let result = if le.len() > bytes {
            let mut result = le[0..bytes].to_vec();
            result.reverse();
            result
        } else {
            let mut result = Vec::<u8>::with_capacity(bytes);
            result.extend(le);
            let bytes_needed = bytes - result.len();
            if bytes_needed > 0 {
                result.extend(vec![0u8; bytes_needed]);
            }
            result.reverse();
            result
        };
        assert_eq!(result.len(), bytes);
        result
    }
}

use crate::RSAError;

const MAX_RSA_MODULUS_BITS: usize = 1024;
const MAX_RSA_MODULUS_LEN: usize = (MAX_RSA_MODULUS_BITS + 7) / 8;

#[derive(Debug)]
pub struct RSAPublicKey {
    bits: u32,
    modulus: BigUint,
    exponent: BigUint,
}

#[derive(Debug)]
pub struct RSAPrivateKey {
    bits: u32,
    modulus: BigUint,
    public_exponent: BigUint,
    exponent: BigUint,
    prime: [BigUint; 2],
    prime_exponent: [BigUint; 2],
    coefficient: BigUint,
}

pub struct RSAProtoKey {
    pub bits: u32,
    pub use_fermat4: bool,
}

impl RSAPublicKey {
    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::<u8>::with_capacity(260);

        result.extend(self.bits.to_le_bytes());
        result.extend(self.modulus.to_be(1024 / 8));
        result.extend(self.exponent.to_be(1024 / 8));

        assert_eq!(result.len(), 260);

        result
    }

    pub fn decode(data: &[u8]) -> Result<Self, String> {
        if data.len() < 260 {
            return Err("Input data is not large enough".to_string());
        }

        let mut reader = BufReader::new(data);

        let mut bits_buf = [0u8; 4];
        reader.read_exact(&mut bits_buf).unwrap();
        let bits = u32::from_le_bytes(bits_buf);

        let mut modulus_buf = [0u8; 1024 / 8];
        reader.read_exact(&mut modulus_buf).unwrap();
        let modulus = BigUint::from_bytes_be(&modulus_buf);

        let mut exponent_buf = [0u8; 1024 / 8];
        reader.read_exact(&mut exponent_buf).unwrap();
        let exponent = BigUint::from_bytes_be(&exponent_buf);

        Ok(Self {
            bits,
            modulus,
            exponent,
        })
    }

    fn rsa_public_block(&self, input: &[u8]) -> Result<Vec<u8>, RSAError> {
        let m = BigUint::from_bytes_be(input);
        let n = &self.modulus;
        let e = &self.exponent;

        if m.cmp(n).is_ge() {
            return Err(RSAError::Data);
        }

        // Perform operation
        let c = m.modpow(e, n);

        let output_len = ((self.bits + 7) / 8) as usize;
        let output = c.to_be(output_len);

        Ok(output)
    }

    fn rsa_public_encrypt(
        &self,
        input: &[u8],
        random_struct: &mut RandomStruct,
    ) -> Result<Vec<u8>, RSAError> {
        let modulus_len = ((self.bits + 7) / 8) as usize;
        if input.len() + 11 > modulus_len {
            return Err(RSAError::Len);
        }

        let mut pkcs_block = [0u8; MAX_RSA_MODULUS_LEN];
        /* block type 2 */
        pkcs_block[1] = 2;

        for e in pkcs_block[2..(modulus_len - input.len() - 1)].iter_mut() {
            loop {
                let random_byte = random_struct.generate_bytes(1)?[0];
                if random_byte != 0 {
                    *e = random_byte;
                    break;
                }
            }
        }

        let mut i = modulus_len - input.len() - 1;

        /* separator */
        pkcs_block[i] = 0;
        i += 1;

        for (target, src) in pkcs_block[i..].iter_mut().zip(input) {
            *target = *src;
        }

        self.rsa_public_block(&pkcs_block[..modulus_len])
    }

    pub fn encrypt(
        &self,
        input: &[u8],
        random_struct: &mut RandomStruct,
    ) -> Result<Vec<u8>, RSAError> {
        let mut result = Vec::<u8>::with_capacity(input.len());
        for chunk in input.chunks(48) {
            let encrypted_chunk = self.rsa_public_encrypt(chunk, random_struct)?;
            result.extend(&encrypted_chunk);
        }
        Ok(result)
    }

    fn rsa_public_decrypt(&self, input: &[u8]) -> Result<Vec<u8>, RSAError> {
        let modulus_len = ((self.bits + 7) / 8) as usize;
        if input.len() > modulus_len {
            return Err(RSAError::Len);
        }

        let pkcs_block = self.rsa_public_block(input)?;

        if pkcs_block.len() != modulus_len {
            return Err(RSAError::Len);
        }

        /* Require block type 1. */
        if pkcs_block[0] != 0 || pkcs_block[1] != 1 {
            return Err(RSAError::Data);
        }

        let mut separator_start: usize = 0;
        for (i, e) in pkcs_block[2..pkcs_block.len() - 1].iter().enumerate() {
            separator_start = i + 2;
            if *e != 0xFF {
                break;
            }
        }

        /* separator */
        if pkcs_block[separator_start] != 0 {
            return Err(RSAError::Data);
        }

        let i = separator_start + 1;

        let output_len = modulus_len - i;

        if output_len + 11 > modulus_len {
            return Err(RSAError::Data);
        }

        let output = pkcs_block[i..].to_vec();

        Ok(output)
    }

    pub fn decrypt(&self, input: &[u8]) -> Result<Vec<u8>, RSAError> {
        let mut result = Vec::<u8>::with_capacity(input.len());
        for chunk in input.chunks(64) {
            let decrypted_chunk = self.rsa_public_decrypt(chunk)?;
            result.extend(&decrypted_chunk);
        }
        Ok(result)
    }
}

impl RSAPrivateKey {
    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::<u8>::with_capacity(708);

        result.extend(self.bits.to_le_bytes());
        result.extend(self.modulus.to_be(1024 / 8));
        result.extend(self.public_exponent.to_be(1024 / 8));
        result.extend(self.exponent.to_be(1024 / 8));
        result.extend(self.prime[0].to_be(512 / 8));
        result.extend(self.prime[1].to_be(512 / 8));
        result.extend(self.prime_exponent[0].to_be(512 / 8));
        result.extend(self.prime_exponent[1].to_be(512 / 8));
        result.extend(self.coefficient.to_be(512 / 8));

        assert_eq!(result.len(), 708);

        result
    }

    pub fn decode(data: &[u8]) -> Result<Self, String> {
        if data.len() < 708 {
            return Err("Input data is not large enough".to_string());
        }

        let mut reader = BufReader::new(data);

        let mut bits_buf = [0u8; 4];
        reader.read_exact(&mut bits_buf).unwrap();
        let bits = u32::from_le_bytes(bits_buf);

        let mut modulus_buf = [0u8; 1024 / 8];
        reader.read_exact(&mut modulus_buf).unwrap();
        let modulus = BigUint::from_bytes_be(&modulus_buf);

        let mut public_exponent_buf = [0u8; 1024 / 8];
        reader.read_exact(&mut public_exponent_buf).unwrap();
        let public_exponent = BigUint::from_bytes_be(&public_exponent_buf);

        let mut exponent_buf = [0u8; 1024 / 8];
        reader.read_exact(&mut exponent_buf).unwrap();
        let exponent = BigUint::from_bytes_be(&exponent_buf);

        let mut prime0_buf = [0u8; 512 / 8];
        reader.read_exact(&mut prime0_buf).unwrap();
        let prime0 = BigUint::from_bytes_be(&prime0_buf);

        let mut prime1_buf = [0u8; 512 / 8];
        reader.read_exact(&mut prime1_buf).unwrap();
        let prime1 = BigUint::from_bytes_be(&prime1_buf);

        let prime = [prime0, prime1];

        let mut prime_exponent0_buf = [0u8; 512 / 8];
        reader.read_exact(&mut prime_exponent0_buf).unwrap();
        let prime_exponent0 = BigUint::from_bytes_be(&prime_exponent0_buf);

        let mut prime_exponent1_buf = [0u8; 512 / 8];
        reader.read_exact(&mut prime_exponent1_buf).unwrap();
        let prime_exponent1 = BigUint::from_bytes_be(&prime_exponent1_buf);

        let prime_exponent = [prime_exponent0, prime_exponent1];

        let mut coefficient_buf = [0u8; 512 / 8];
        reader.read_exact(&mut coefficient_buf).unwrap();
        let coefficient = BigUint::from_bytes_be(&coefficient_buf);

        Ok(Self {
            bits,
            modulus,
            public_exponent,
            exponent,
            prime,
            prime_exponent,
            coefficient,
        })
    }

    pub fn public_key(&self) -> RSAPublicKey {
        RSAPublicKey {
            bits: self.bits,
            modulus: self.modulus.clone(),
            exponent: self.public_exponent.clone(),
        }
    }

    pub fn rsa_private_encrypt(&self, input: &[u8]) -> Result<Vec<u8>, RSAError> {
        let modulus_len = ((self.bits + 7) / 8) as usize;
        if input.len() + 11 > modulus_len {
            return Err(RSAError::Len);
        }

        let mut pkcs_block = [0u8; MAX_RSA_MODULUS_LEN];
        /* block type 1 */
        pkcs_block[1] = 1;

        for e in pkcs_block
            .iter_mut()
            .take(modulus_len - input.len() - 1)
            .skip(2)
        {
            *e = 0xFF;
        }

        let mut i = modulus_len - input.len() - 1;

        /* separator */
        pkcs_block[i] = 0;
        i += 1;

        for (target, src) in pkcs_block[i..].iter_mut().zip(input) {
            *target = *src;
        }

        self.rsa_private_block(&pkcs_block[0..modulus_len])
    }

    pub fn encrypt(&self, input: &[u8]) -> Result<Vec<u8>, RSAError> {
        let mut result = Vec::<u8>::with_capacity(input.len());
        for chunk in input.chunks(48) {
            let encrypted_chunk = self.rsa_private_encrypt(chunk)?;
            result.extend(&encrypted_chunk);
        }
        Ok(result)
    }

    pub fn rsa_private_decrypt(&self, input: &[u8]) -> Result<Vec<u8>, RSAError> {
        let modulus_len = ((self.bits + 7) / 8) as usize;
        if input.len() > modulus_len {
            return Err(RSAError::Len);
        }

        let pkcs_block = self.rsa_private_block(input)?;

        if pkcs_block.len() != modulus_len {
            return Err(RSAError::Len);
        }

        /* Require block type 2. */
        if pkcs_block[0] != 0 || pkcs_block[1] != 2 {
            return Err(RSAError::Data);
        }

        let mut separator_start: usize = 0;
        for (i, e) in pkcs_block[2..pkcs_block.len() - 1].iter().enumerate() {
            /* separator */
            separator_start = i + 2;
            if *e == 0 {
                break;
            }
        }

        let i = separator_start + 1;
        if i > modulus_len {
            return Err(RSAError::Data);
        }

        let output_len = modulus_len - i;

        if output_len + 11 > modulus_len {
            return Err(RSAError::Data);
        }

        let output = pkcs_block[i..].to_vec();

        Ok(output)
    }

    pub fn decrypt(&self, input: &[u8]) -> Result<Vec<u8>, RSAError> {
        let mut result = Vec::<u8>::with_capacity(input.len());
        for chunk in input.chunks(64) {
            let decrypted_chunk = self.rsa_private_decrypt(chunk)?;
            result.extend(&decrypted_chunk);
        }
        Ok(result)
    }

    pub fn rsa_private_block(&self, input: &[u8]) -> Result<Vec<u8>, RSAError> {
        let c = BigUint::from_bytes_be(input);
        let n = &self.modulus;
        let p = &self.prime[0];
        let q = &self.prime[1];
        let dp = &self.prime_exponent[0];
        let dq = &self.prime_exponent[1];
        let qinv = &self.coefficient;

        if c.cmp(n).is_ge() {
            return Err(RSAError::Data);
        }

        /* Compute mP = cP^dP mod p  and  mQ = cQ^dQ mod q. */

        let cp = c.mod_floor(p);
        let cq = c.mod_floor(q);
        let mp = cp.modpow(dp, p);
        let mq = cq.modpow(dq, q);

        /* Chinese Remainder Theorem:
        m = ((((mP - mQ) mod p) * qInv) mod p) * q + mQ.
        */
        let mut t;
        if mp.cmp(&mq).is_ge() {
            t = mp.sub(&mq);
        } else {
            t = mq.clone().sub(&mp);
            t = p.sub(t);
        }
        t = t.mul(qinv).mod_floor(p);
        t = t.mul(q);
        t = t.add(mq);

        let output_len = ((self.bits + 7) / 8) as usize;
        let output = t.to_be(output_len);

        Ok(output)
    }
}
