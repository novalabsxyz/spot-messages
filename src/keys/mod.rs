use super::PublicKey;

use std::result::Result;

pub mod file;

pub trait KeyTrait {
    type Error: core::fmt::Debug;
    fn pubkey(&self) -> Result<PublicKey, Self::Error>;
    fn sign(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error>;
}
