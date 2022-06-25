mod r_random;
pub use r_random::RandomStruct;

mod rsa;
pub use crate::rsa::{RSAPrivateKey, RSAProtoKey, RSAPublicKey};

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
