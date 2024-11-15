pub mod secp256k1;
pub mod eddsa_ed25519;

use std::error::Error as StdError;
use std::ops::Deref;
use crate::signing::secp256k1::Secp256k1PrivateKey;
use crate::signing::eddsa_ed25519::EddsaEd25519PrivateKey;

#[derive(Debug)]
pub enum Error {
    /// Returned when trying to create an algorithm which does not exist.
    NoSuchAlgorithm(String),
    /// Returned when an error occurs during deserialization of a Private or
    /// Public key from various formats.
    ParseError(String),
    /// Returned when an error occurs during the signing process.
    SigningError(Box<dyn StdError>),
    /// Returned when an error occurs during key generation
    KeyGenError(String),
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::SigningError(err) => Some(&**err),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::NoSuchAlgorithm(ref s) => write!(f, "NoSuchAlgorithm: {}", s),
            Error::ParseError(ref s) => write!(f, "ParseError: {}", s),
            Error::SigningError(ref err) => write!(f, "SigningError: {}", err),
            Error::KeyGenError(ref s) => write!(f, "KeyGenError: {}", s),
        }
    }
}

/// A private key instance.
/// The underlying content is dependent on implementation.
pub trait PrivateKey {
    /// Returns the algorithm name used for this private key.
    fn get_algorithm_name(&self) -> &str;
    /// Return the private key encoded as a hex string.
    fn as_hex(&self) -> String;
    /// Return the private key bytes.
    fn as_slice(&self) -> &[u8];
    /// Return the address String.
    fn get_address(&self) -> String {
        let context = create_context(self
            .get_algorithm_name()).unwrap();
        String::from(bs58::encode(
            context
                .get_public_key(&*create_private_key(self.get_algorithm_name(), self.as_hex().as_str()).unwrap()
                ).unwrap().as_slice())
            .into_string())
    }
    /// Return the address String.
    fn get_pubkey(&self) -> String {
        let context = create_context(self
            .get_algorithm_name()).unwrap();
        context
            .get_public_key(&*create_private_key(self.get_algorithm_name(), self.as_hex().as_str()).unwrap()
            ).unwrap().as_hex()
    }
}

/// A public key instance.
/// The underlying content is dependent on implementation.
pub trait PublicKey {
    /// Returns the algorithm name used for this public key.
    fn get_algorithm_name(&self) -> &str;
    /// Return the public key encoded as a hex string.
    fn as_hex(&self) -> String;
    /// Return the public key bytes.
    fn as_slice(&self) -> &[u8];
    /// Return the address String.
    fn get_address(&self) -> String {
        String::from(bs58::encode(self.as_slice()).into_string())
    }
}

/// A context for a cryptographic signing algorithm.
pub trait Context {
    /// Returns the algorithm name.
    fn get_algorithm_name(&self) -> &str;
    /// Sign a message
    /// Given a private key for this algorithm, sign the given message bytes
    /// and return a hex-encoded string of the resulting signature.
    /// # Arguments
    ///
    /// * `message`- the message bytes
    /// * `private_key` the private key
    ///
    /// # Returns
    ///
    /// * `signature` - The signature in a hex-encoded string
    fn sign(&self, message: &[u8], key: &dyn PrivateKey) -> Result<String, Error>;

    /// Verifies that the signature of a message was produced with the
    /// associated public key.
    /// # Arguments
    ///
    /// * `signature` - the hex-encoded signature
    /// * `message` - the message bytes
    /// * `public_key` - the public key to use for verification
    ///
    /// # Returns
    ///
    /// * `boolean` - True if the public key is associated with the signature for that method,
    ///            False otherwise
    fn verify(&self, signature: &[u8], message: &[u8], key: &dyn PublicKey) -> Result<bool, Error>;

    /// Produce the public key for the given private key.
    /// # Arguments
    ///
    /// `private_key` - a private key
    ///
    /// # Returns
    /// * `public_key` - the public key for the given private key
    fn get_public_key(&self, private_key: &dyn PrivateKey) -> Result<Box<dyn PublicKey>, Error>;

    ///Generates a new random PrivateKey using this context.
    /// # Returns
    ///
    /// * `private_key` - a random private key
    fn new_random_private_key(&self) -> Result<Box<dyn PrivateKey>, Error>;
}

pub fn create_secret_key(algorithm_name: &str) -> Result<Box<dyn PrivateKey>, Error> {
    match algorithm_name {
        "secp256k1" => Ok(Box::new(Secp256k1PrivateKey::new())),
        "eddsa_ed25519" => Ok(Box::new(EddsaEd25519PrivateKey::new())),
        _ => Err(Error::NoSuchAlgorithm(format!(
            "no such algorithm: {}",
            algorithm_name
        )))
    }
}

pub fn create_private_key(algorithm_name: &str, key: &str) -> Result<Box<dyn PrivateKey>, Error> {
    match algorithm_name {
        "secp256k1" => Ok(Box::new(Secp256k1PrivateKey::from_hex(key).unwrap())),
        "eddsa_ed25519" => Ok(Box::new(EddsaEd25519PrivateKey::from_hex(key).unwrap())),
        _ => Err(Error::NoSuchAlgorithm(format!(
            "no such algorithm: {}",
            algorithm_name
        )))
    }
}

pub fn create_context(algorithm_name: &str) -> Result<Box<dyn Context>, Error> {
    match algorithm_name {
        "secp256k1" => Ok(Box::new(secp256k1::Secp256k1Context::new())),
        "eddsa_ed25519" => Ok(Box::new(eddsa_ed25519::EddsaEd25519Context::new())),
        _ => Err(Error::NoSuchAlgorithm(format!(
            "no such algorithm: {}",
            algorithm_name
        ))),
    }
}

/// Factory for generating signers.
pub struct CryptoFactory<'a> {
    context: &'a dyn Context,
}

impl<'a> CryptoFactory<'a> {
    /// Constructs a CryptoFactory.
    /// # Arguments
    ///
    /// * `context` - a cryptographic context
    pub fn new(context: &'a dyn Context) -> Self {
        CryptoFactory { context }
    }

    /// Returns the context associated with this factory
    ///
    /// # Returns
    ///
    /// * `context` - a cryptographic context
    pub fn get_context(&self) -> &dyn Context {
        self.context
    }

    /// Create a new signer for the given private key.
    ///
    /// # Arguments
    ///
    /// `private_key` - a private key
    ///
    /// # Returns
    ///
    /// * `signer` - a signer instance
    pub fn new_signer(&self, key: &'a dyn PrivateKey) -> Signer {
        Signer::new(self.context, key)
    }
}

enum ContextAndKey<'a> {
    ByRef(&'a dyn Context, &'a dyn PrivateKey),
    ByBox(Box<dyn Context>, Box<dyn PrivateKey>),
}

/// A convenient wrapper of Context and PrivateKey
pub struct Signer<'a> {
    context_and_key: ContextAndKey<'a>,
}

impl<'a> Signer<'a> {
    /// Constructs a new Signer
    ///
    /// # Arguments
    ///
    /// * `context` - a cryptographic context
    /// * `private_key` - private key
    pub fn new(context: &'a dyn Context, key: &'a dyn PrivateKey) -> Self {
        Signer {
            context_and_key: ContextAndKey::ByRef(context, key),
        }
    }

    /// Constructs a new Signer with boxed arguments
    ///
    /// # Arguments
    ///
    /// * `context` - a cryptographic context
    /// * `key` - private key
    pub fn new_boxed(context: Box<dyn Context>, key: Box<dyn PrivateKey>) -> Self {
        Signer {
            context_and_key: ContextAndKey::ByBox(context, key),
        }
    }

    /// Signs the given message.
    ///
    /// # Arguments
    ///
    /// * `message` - the message bytes
    ///
    /// # Returns
    ///
    /// * `signature` - the signature in a hex-encoded string
    pub fn sign(&self, message: &[u8]) -> Result<String, Error> {
        match &self.context_and_key {
            ContextAndKey::ByRef(context, key) => context.sign(message, *key),
            ContextAndKey::ByBox(context, key) => context.sign(message, key.as_ref()),
        }
    }

    /// Return the public key for this Signer instance.
    ///
    /// # Returns
    ///
    /// * `public_key` - the public key instance
    pub fn get_public_key(&self) -> Result<Box<dyn PublicKey>, Error> {
        match &self.context_and_key {
            ContextAndKey::ByRef(context, key) => context.get_public_key(*key),
            ContextAndKey::ByBox(context, key) => context.get_public_key(key.as_ref()),
        }
    }
}

pub fn hex_str_to_bytes(s: &str) -> Result<Vec<u8>, Error> {
    for (i, ch) in s.chars().enumerate() {
        if !ch.is_digit(16) {
            return Err(Error::ParseError(format!(
                "invalid character position {}",
                i
            )));
        }
    }

    let input: Vec<_> = s.chars().collect();

    let decoded: Vec<u8> = input
        .chunks(2)
        .map(|chunk| {
            ((chunk[0].to_digit(16).unwrap() << 4) | (chunk[1].to_digit(16).unwrap())) as u8
        })
        .collect();

    Ok(decoded)
}

pub fn bytes_to_hex_str(b: &[u8]) -> String {
    b.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
mod signing_test {
    use super::create_context;

    #[test]
    fn no_such_algorithm() {
        let result = create_context("invalid");
        assert!(result.is_err())
    }
}
