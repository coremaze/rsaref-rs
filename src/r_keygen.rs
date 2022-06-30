use std::str::FromStr;

use crate::rsa::{
    RSAPrivateKey, RSAProtoKey, RSAPublicKey, MAX_RSA_MODULUS_BITS, MIN_RSA_MODULUS_BITS,
};
use crate::RSAError;
use num_integer::Integer;
use rand::thread_rng;
use rsa::{BigUint, RsaPrivateKey};
use std::ops::{Add, Mul, Sub};

fn generate_primes(proto_key: &RSAProtoKey) -> Result<[BigUint; 2], RSAError> {
    // Use other rsa library to generate primes for us (lol)
    let key = RsaPrivateKey::new(&mut thread_rng(), proto_key.bits as usize)
        .map_err(|_| RSAError::Key)?;
    let mut primes: [BigUint; 2] = Default::default();
    primes[0] = key.primes()[0].clone();
    primes[1] = key.primes()[1].clone();
    Ok(primes)
}

fn mod_inv(b: &BigUint, c: &BigUint) -> BigUint {
    /* Apply extended Euclidean algorithm, modified to avoid negative numbers. */
    let mut u1 = BigUint::from_str("1").unwrap();
    let mut v1 = BigUint::from_str("0").unwrap();
    let mut u3 = b.clone();
    let mut v3 = c.clone();

    let mut u1_sign = 1;
    let zero = BigUint::from_str("0").unwrap();

    while v3 != zero {
        let (q, t3) = u3.div_rem(&v3);
        let w = q.mul(&v1);
        let t1 = u1.add(&w);
        u1 = v1.clone();
        v1 = t1.clone();
        u3 = v3.clone();
        v3 = t3.clone();
        u1_sign = -u1_sign;
    }

    /* Negate result if sign is negative. */
    if u1_sign < 0 {
        c.sub(&u1)
    } else {
        u1
    }
}

pub fn generate_pem_keys(
    proto_key: &RSAProtoKey,
) -> Result<(RSAPublicKey, RSAPrivateKey), RSAError> {
    let bits = proto_key.bits as usize;
    if !(MIN_RSA_MODULUS_BITS..=MAX_RSA_MODULUS_BITS).contains(&bits) {
        return Err(RSAError::ModulusLen);
    }

    let e = if proto_key.use_fermat4 {
        BigUint::from_str("65537").unwrap()
    } else {
        BigUint::from_str("3").unwrap()
    };

    let primes = generate_primes(proto_key)?;

    /* Sort so that p > q. (p = q case is extremely unlikely.) */
    let (p, q) = if primes[0] > primes[1] {
        (&primes[0], &primes[1])
    } else {
        (&primes[1], &primes[0])
    };

    /* Compute n = pq, qInv = q^{-1} mod p, d = e^{-1} mod (p-1)(q-1),
    dP = d mod p-1, dQ = d mod q-1. */

    let n = p.clone().mul(q);
    let q_inv = mod_inv(q, p);

    let t = BigUint::from_str("1").unwrap();
    let p_minus_1 = p.clone().sub(&t);
    let q_minus_1 = q.clone().sub(&t);
    let phi_n = p_minus_1.clone().mul(&q_minus_1);

    let d = mod_inv(&e, &phi_n);
    let (_, dp) = d.div_rem(&p_minus_1);
    let (_, dq) = d.div_rem(&q_minus_1);

    let private_key = RSAPrivateKey::from_components(
        proto_key.bits,
        n,
        e,
        d,
        [p.clone(), q.clone()],
        [dp, dq],
        q_inv,
    );

    Ok((private_key.public_key(), private_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_prime_length() {
        match generate_primes(&RSAProtoKey {
            bits: 512,
            use_fermat4: true,
        }) {
            Ok(primes) => {
                assert!(primes[0].to_bytes_be().len() == 32);
                assert!(primes[1].to_bytes_be().len() == 32);
                assert!(primes[0] != primes[1]);
            }
            Err(_) => assert!(false, "generate_primes returned an error."),
        }
    }

    #[test]
    pub fn test_prime_crypt() {
        match generate_pem_keys(&RSAProtoKey {
            bits: 512,
            use_fermat4: true,
        }) {
            Ok((public_key, private_key)) => {
                let data = (0u8..=255).collect::<Vec<u8>>();

                let encrypted_data = private_key.encrypt(&data).unwrap();
                let decrypted_data = public_key.decrypt(&encrypted_data).unwrap();

                assert!(data == decrypted_data);
            }
            Err(_) => assert!(false, "generate_primes returned an error."),
        }
    }
}
