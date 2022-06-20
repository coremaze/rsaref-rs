mod r_random;
pub use r_random::RandomStruct;

mod nn;
pub use nn::{NNDigit, NNHalfDigit};

#[derive(Debug)]
pub enum RSAError {
    ContentEncoding,
    Data,
    DigestAlgorithm,
    Encoding,
    Key,
    KeyEncoding,
    Len,
    ModulusLen,
    NeedRandom,
    PrivateKey,
    PublicKey,
    Signature,
    SignatureEncoding,
    EncryptionAlgorithm,
}

// #[derive(Debug)]
// pub struct RSAPublicKey {
//     bits: usize,
//     modulus: BigUint,
//     exponent: BigUint,
// }

// #[derive(Debug)]
// pub struct RSAPrivateKey {
//     bits: usize,
//     modulus: BigUint,
//     public_exponent: BigUint,
//     exponent: BigUint,
//     prime: [BigUint; 2],
//     prime_exponent: [BigUint; 2],
//     coefficient: BigUint,
// }

// pub struct RSAProtoKey {
//     pub bits: usize,
//     pub use_fermat4: bool,
// }
