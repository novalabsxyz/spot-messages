use super::{keys::KeyTrait, Error, PublicKey, Result, Verify};

pub trait IntoFromLoraPayload<const N: usize> {
    fn into_lora_bytes_with_signature<K: KeyTrait>(self, key: &K) -> Result<Vec<u8>>
    where
        Self: Sized,
    {
        let bytes = self.into_lora_bytes();
        let signature = key.sign(&bytes).map_err(|e| Error::Key(e.to_string()))?;
        // remove the first two bytes because we can infer them later
        let mut bytes = bytes.to_vec();
        bytes.append(&mut signature[2..].to_vec());
        Ok(bytes)
    }

    fn from_lora_vec_with_verified_signature(pubkey: &PublicKey, vec: Vec<u8>) -> Result<Self>
    where
        Self: Sized,
    {
        let size = vec.len();
        if size < N {
            return Err(Error::InvalidVecForParsingLoraPayload {
                payload: Self::label(),
                size,
            });
        }
        let bytes: [u8; N] =
            vec[..N]
                .try_into()
                .map_err(|_| Error::InvalidVecForParsingLoraPayload {
                    payload: Self::label(),
                    size,
                })?;

        let signature_bytes = &vec[N..];
        // add back in the first two bytes of the signature
        let mut signature = vec![0x30, signature_bytes.len() as u8];
        signature.append(&mut signature_bytes.to_vec());
        pubkey
            .verify(&bytes, &signature)
            .map_err(|_| Error::SignatureVerification {
                pubkey: Box::new(pubkey.clone()),
                msg: bytes.to_vec(),
                signature: signature.to_vec(),
            })?;

        Ok(Self::from_lora_bytes(bytes))
    }
    fn into_lora_bytes(self) -> [u8; N];
    fn from_lora_bytes(bytes: [u8; N]) -> Self;
    fn label() -> &'static str;
}
