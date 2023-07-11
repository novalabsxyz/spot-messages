use super::{Error, Result, PublicKey};
use rust_decimal::Decimal;
use helium_proto::DataRate;

/*
message hotspot {
  bytes pubkey = 1;
  uint64 h3_cell = 2;
  // snr in 0.1 dB
  uint32 snr = 3;
  // rssi in 0.1 dBm
  int32 rssi = 4;
  // frequency in mHz
  int32 frequency = 5;
  data_rate data_rate = 6;
}
 */

#[derive(Debug, Clone, PartialEq)]
pub struct LoraGw {
    pub pubkey: PublicKey,
    pub h3_cell: h3o::CellIndex,
    pub snr: Decimal,
    pub rssi: Decimal,
    pub frequency: Decimal,
    pub data_rate: DataRate,
}


impl TryFrom<helium_proto::LoraGw> for LoraGw {
    type Error = Error;
    fn try_from(value: helium_proto::LoraGw) -> Result<Self> {
        Ok(LoraGw {
            pubkey: PublicKey::try_from(value.pubkey.clone())
                .map_err(|error| Error::PubkeyParse { error, bytes: value.pubkey })?,
            h3_cell: h3o::CellIndex::try_from(value.h3_cell)?,
            snr: snr::from_proto_units(value.snr),
            rssi: rssi::from_proto_units(value.rssi),
            frequency: frequency::from_proto_units( value.frequency),
            data_rate: DataRate::from_i32(value.data_rate).ok_or(Error::InvalidDatarate(value.data_rate))?,
        })
    }
}

impl From<LoraGw> for helium_proto::LoraGw {
    fn from(value: LoraGw) -> Self {
        helium_proto::LoraGw {
            pubkey: value.pubkey.to_vec(),
            h3_cell: value.h3_cell.into(),
            snr: snr::to_proto_units(value.snr),
            rssi: rssi::to_proto_units(value.rssi),
            frequency: frequency::to_proto_units(value.frequency),
            data_rate: value.data_rate.into(),
        }
    }
}

pub mod snr {
    use super::*;

    const SNR_PROTO_SCALAR: Decimal = Decimal::from_parts(1, 0, 0, false, 1);

    pub fn to_proto_units(snr: Decimal) -> u32 {
        let scaled = snr.checked_div(SNR_PROTO_SCALAR).unwrap().round();
        scaled.to_string().parse::<u32>().unwrap()
    }

    pub fn from_proto_units(snr: u32) -> Decimal {
        let snr_unscaled = Decimal::new(snr.into(), 0);
        snr_unscaled.checked_mul(SNR_PROTO_SCALAR).unwrap()
    }
}

pub mod rssi {
    use super::*;

    const RSSI_PROTO_SCALAR: Decimal = Decimal::from_parts(1, 0, 0, false, 2);

    pub fn to_proto_units(rssi: Decimal) -> i32 {
        let scaled = rssi.checked_div(RSSI_PROTO_SCALAR).unwrap().round();
        scaled.to_string().parse::<i32>().unwrap()
    }

    pub fn from_proto_units(rssi: i32) -> Decimal {
        let rssi_unscaled = Decimal::new(rssi.into(), 0);
        rssi_unscaled.checked_mul(RSSI_PROTO_SCALAR).unwrap()
    }
}

pub mod frequency {
    use super::*;

    const FREQUENCY_PROTO_SCALAR: Decimal = Decimal::from_parts(1, 0, 0, false, 3);

    pub fn to_proto_units(frequency: Decimal) -> u32 {
        let scaled = frequency.checked_div(FREQUENCY_PROTO_SCALAR).unwrap().round();
        scaled.to_string().parse::<u32>().unwrap()
    }

    pub fn from_proto_units(frequency: u32) -> Decimal {
        let frequency_unscaled = Decimal::new(frequency.into(), 0);
        frequency_unscaled.checked_mul(FREQUENCY_PROTO_SCALAR).unwrap()
    }
}