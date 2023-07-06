use chrono::{prelude::*, DateTime, NaiveDateTime};

pub use helium_proto::{self, Message as ProtoMessage};
use helium_proto::{mapper_payload, MapperMsg};

pub use helium_crypto::public_key::PublicKey;
use helium_crypto::Verify;

mod cell_attach;
pub use cell_attach::*;

mod gps;
pub use gps::*;

mod cell_scan;
pub use cell_scan::*;

mod keys;
pub use keys::*;

mod ports;
pub use ports::*;

mod beacon;
pub use beacon::*;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub enum Payload {
    CellAttach(CellAttach),
    CellScan(CellScan),
    Beacon(Beacon),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub payload: Payload,
    pub signature: Vec<u8>,
    pub pubkey: PublicKey,
    pub hotspots: Vec<PublicKey>,
}

use thiserror::Error;
#[derive(Error, Debug)]
pub enum Error {
    #[error("parse int error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    #[error("unexpected attach result str: {0}")]
    UnexpectedAttachResultStr(String),
    #[error("h3o: {0}")]
    H3o(#[from] h3o::error::InvalidLatLng),
    #[error("invalid attach result value: {value}")]
    InvalidAttachResultInt { value: i32 },
    #[error("proto should have some but has none for field \"{0}\"")]
    ProtoHasNone(&'static str),
    #[error("decimal could not map to float: {decimal}")]
    DecimalCouldNotMapToFloat { decimal: rust_decimal::Decimal },
    #[error("pubkey parsing error: {error} for the following bytes: {bytes:?}")]
    PubkeyParse {
        error: helium_crypto::Error,
        bytes: Vec<u8>,
    },
    #[error("the signature ({signature:?}) does not verify the message ({msg:?}) for the pubkey {pubkey}")]
    SignatureVerification {
        pubkey: Box<PublicKey>,
        msg: Vec<u8>,
        signature: Vec<u8>,
    },
}

impl TryFrom<mapper_payload::Message> for Payload {
    type Error = Error;

    fn try_from(value: mapper_payload::Message) -> std::result::Result<Self, Self::Error> {
        match value {
            mapper_payload::Message::Beacon(beacon) => Ok(Payload::Beacon(beacon.try_into()?)),
            mapper_payload::Message::Attach(attach) => Ok(Payload::CellAttach(attach.try_into()?)),
            mapper_payload::Message::Scan(scan) => Ok(Payload::CellScan(scan.try_into()?)),
        }
    }
}

impl TryFrom<MapperMsg> for Message {
    type Error = Error;

    fn try_from(value: MapperMsg) -> std::result::Result<Self, Self::Error> {
        match value.version {
            Some(helium_proto::mapper_msg::Version::MsgV1(msg)) => msg.try_into(),
            _ => Err(Error::ProtoHasNone("version")),
        }
    }
}

impl Message {
    pub fn try_from_with_signature_verification(value: MapperMsg) -> Result<Self> {
        match value.version {
            Some(helium_proto::mapper_msg::Version::MsgV1(msg)) => Self::inner_try_from(msg, true),
            _ => Err(Error::ProtoHasNone("version")),
        }
    }

    /// with_verification flag will verify the signature of the message
    fn inner_try_from(value: helium_proto::MapperMsgV1, with_verification: bool) -> Result<Self> {
        let payload = value.payload.ok_or(Error::ProtoHasNone("payload"))?;
        let payload = payload.message.ok_or(Error::ProtoHasNone("message"))?;
        let pubkey = PublicKey::from_bytes(&value.pubkey).map_err(|error| Error::PubkeyParse {
            error,
            bytes: value.pubkey,
        })?;

        if with_verification {
            let mut payload_bytes = Vec::new();
            payload.encode(&mut payload_bytes);
            pubkey
                .verify(&payload_bytes, &value.signature)
                .map_err(|_| Error::SignatureVerification {
                    pubkey: Box::new(pubkey.clone()),
                    msg: payload_bytes,
                    signature: value.signature.clone(),
                })?;
        }

        let payload = payload.try_into()?;

        Ok(Self {
            payload,
            signature: value.signature,
            pubkey,
            hotspots: value
                .hotspots
                .into_iter()
                .map(|v| {
                    PublicKey::from_bytes(&v)
                        .map_err(|error| Error::PubkeyParse { error, bytes: v })
                })
                .collect::<Result<_>>()?,
        })
    }
}

/// This TryFrom implementation will throw an error if:
///     * certain Vec<u8>'s are not parsable as pubkeys
///     * the protos are missing fields
impl TryFrom<helium_proto::MapperMsgV1> for Message {
    type Error = Error;

    fn try_from(value: helium_proto::MapperMsgV1) -> std::result::Result<Self, Self::Error> {
        Self::inner_try_from(value, false)
    }
}

fn mapper_msg_with_payload(payload: mapper_payload::Message) -> MapperMsg {
    use helium_proto::{mapper_msg, MapperMsgV1, MapperPayload};
    MapperMsg {
        version: Some(mapper_msg::Version::MsgV1(MapperMsgV1 {
            pubkey: vec![0; 32],
            payload: Some(MapperPayload {
                message: Some(payload),
            }),
            signature: vec![0; 64],
            hotspots: vec![],
        })),
    }
}
