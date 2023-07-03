use chrono::{prelude::*, DateTime, NaiveDateTime};

pub use helium_proto::{self, Message};
mod cell_attach;
pub use cell_attach::*;

mod gps;
pub use gps::*;

mod cell_scan;
pub use cell_scan::*;

mod ports;
pub use ports::*;

mod beacon;
pub use beacon::*;

pub type Result<T = ()> = std::result::Result<T, Error>;

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
}

fn mapper_msg_with_payload(
    payload: helium_proto::mapper_payload::Message,
) -> helium_proto::MapperMsg {
    use helium_proto::{mapper_msg, MapperMsg, MapperMsgV1, MapperPayload};
    MapperMsg {
        version: Some(mapper_msg::Version::MsgV1(MapperMsgV1 {
            pub_key: vec![0; 32],
            payload: Some(MapperPayload {
                message: Some(payload),
            }),
            signature: vec![0; 64],
            hotspots: vec![],
        })),
    }
}
