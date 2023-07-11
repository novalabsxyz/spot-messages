use std::result::Result;

pub mod file;

pub trait KeyTrait {
    type Error: core::fmt::Debug + core::fmt::Display;
    fn pubkey(&self) -> Result<helium_crypto::public_key::PublicKey, Self::Error>;
    fn sign(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error>;
}
