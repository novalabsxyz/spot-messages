use super::KeyTrait;
use helium_crypto::{KeyTag, KeyType, Network};

use rand::rngs::OsRng;
use std::{
    convert::TryFrom,
    fs,
    path::{self, Path},
    sync::Arc,
};

#[derive(Clone)]
pub struct File {
    pub keypair: Arc<helium_crypto::Keypair>,
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("helium crypto error: {0}")]
    HeliumCrypto(#[from] helium_crypto::Error),
    #[error("io error loading keypair: {0}")]
    IoKeypairRead(std::io::Error),
    #[error("io error writing keypair: {0}")]
    IoKeypairWrite(std::io::Error),
}

impl File {
    pub fn load(path: &Path) -> Result<File, Error> {
        let data = fs::read(path).map_err(Error::IoKeypairRead)?;
        if data.is_empty() {
            Ok(Self::create_and_save_key(path)?)
        } else {
            Ok(helium_crypto::Keypair::try_from(&data[..])?.into())
        }
    }

    fn save(keypair: &helium_crypto::Keypair, path: &Path) -> Result<(), Error> {
        if let Some(parent) = path::PathBuf::from(path).parent() {
            fs::create_dir_all(parent).map_err(Error::IoKeypairWrite)?;
        };
        fs::write(path, keypair.to_vec()).map_err(Error::IoKeypairWrite)?;
        Ok(())
    }

    pub fn create_key() -> Result<File, Error> {
        Ok(helium_crypto::Keypair::generate(
            KeyTag {
                network: Network::MainNet,
                key_type: KeyType::Secp256k1,
            },
            &mut OsRng,
        )
        .into())
    }

    pub fn create_and_save_key(path: &path::Path) -> Result<File, Error> {
        let file = Self::create_key()?;
        Self::save(&file.keypair, path)?;
        Ok(file)
    }

    pub fn sign(&self, msg: &[u8]) -> Result<Vec<u8>, Error> {
        use helium_crypto::Sign;
        Ok(self.keypair.sign(msg)?)
    }
}

impl From<helium_crypto::Keypair> for File {
    fn from(keypair: helium_crypto::Keypair) -> Self {
        Self {
            keypair: Arc::new(keypair),
        }
    }
}

impl KeyTrait for File {
    type Error = Error;

    fn pubkey(&self) -> Result<helium_crypto::PublicKey, Self::Error> {
        Ok(self.keypair.public_key().clone())
    }

    fn sign(&self, msg: &[u8]) -> Result<Vec<u8>, Self::Error> {
        use helium_crypto::Sign;
        Ok(self.keypair.sign(msg)?)
    }
}
