use super::{mapper_msg_with_payload, Error, Result, Serialize};
use helium_proto::MapperScan;

use crate::Gps;

pub const CBRS_MCC: u16 = 315;
pub const CBRS_MNC: u16 = 10;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CellScan {
    pub scan_counter: u32,
    pub gps: Gps,
    pub results: Vec<CellScanResult>,
}

impl CellScan {
    pub fn random() -> CellScan {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut results = Vec::new();
        for _ in 0..rng.gen_range(1..40) {
            results.push(CellScanResult::random());
        }
        CellScan {
            scan_counter: 24,
            gps: Gps::rounded(),
            results,
        }
    }
}

impl From<CellScan> for helium_proto::MapperCellScanV1 {
    fn from(scan_response: CellScan) -> Self {
        Self {
            scan_counter: scan_response.scan_counter,
            gps: Some(scan_response.gps.into()),
            results: scan_response
                .results
                .into_iter()
                .map(|r| r.into())
                .collect(),
        }
    }
}

impl TryFrom<helium_proto::MapperCellScanV1> for CellScan {
    type Error = Error;

    fn try_from(proto: helium_proto::MapperCellScanV1) -> Result<Self> {
        if let Some(gps) = proto.gps {
            Ok(Self {
                scan_counter: proto.scan_counter,
                gps: gps.into(),
                results: proto.results.into_iter().map(|r| r.into()).collect(),
            })
        } else {
            Err(Error::ProtoHasNone("gps"))
        }
    }
}

impl From<CellScan> for helium_proto::mapper_payload::Message {
    fn from(scan_results: CellScan) -> Self {
        use helium_proto::{mapper_payload, mapper_scan};
        mapper_payload::Message::Scan(MapperScan {
            version: Some(mapper_scan::Version::ScanV1(scan_results.into())),
        })
    }
}

impl TryFrom<MapperScan> for CellScan {
    type Error = Error;

    fn try_from(proto: MapperScan) -> Result<Self> {
        match proto.version {
            Some(helium_proto::mapper_scan::Version::ScanV1(v1)) => v1.try_into(),
            None => Err(Error::ProtoHasNone("version")),
        }
    }
}

impl From<CellScan> for helium_proto::MapperMsg {
    fn from(scan_results: CellScan) -> Self {
        mapper_msg_with_payload(scan_results.into())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CellScanResult {
    pub mcc: u16,
    pub mnc: u16,
    pub earfcn: u32,
    pub physical_cell_id: u64,
    pub rsrp: i32,
    pub rsrq: i32,
    pub cell_id: u64,
    pub bandwidth: u32,
    pub lte: bool,
}

impl CellScanResult {
    pub fn is_our_network(&self) -> Result<bool> {
        if self.mcc == CBRS_MCC && self.mnc == CBRS_MNC {
            let top_20_bits = self.cell_id >> 8;
            // Helium cells all have a prefix in this range
            Ok((0x0099D..=0x00A00).contains(&top_20_bits))
        } else {
            Ok(false)
        }
    }

    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Self {
            mcc: rng.gen_range(0..999),
            mnc: rng.gen_range(0..999),
            cell_id: rng.gen_range(0..68719476735),
            earfcn: rng.gen_range(0..4294967295),
            rsrp: rng.gen_range(-144..-44),
            rsrq: rng.gen_range(-20..-3),
            physical_cell_id: rng.gen_range(0..503),
            bandwidth: rng.gen_range(0..4294967295),
            lte: true,
        }
    }
}

impl From<CellScanResult> for helium_proto::MapperCellScanResult {
    fn from(scan_result: CellScanResult) -> Self {
        Self {
            lte: scan_result.lte,
            cid: scan_result.cell_id,
            plmn: ((scan_result.mcc as u32) << 16) | scan_result.mnc as u32,
            fcn: scan_result.earfcn,
            pci: scan_result.physical_cell_id as u32,
            rsrp: scan_result.rsrp,
            rsrq: scan_result.rsrq,
            bandwidth: scan_result.bandwidth,
        }
    }
}

impl From<helium_proto::MapperCellScanResult> for CellScanResult {
    fn from(scan_result: helium_proto::MapperCellScanResult) -> Self {
        Self {
            lte: scan_result.lte,
            cell_id: scan_result.cid,
            mcc: (scan_result.plmn >> 16) as u16,
            mnc: scan_result.plmn as u16,
            earfcn: scan_result.fcn,
            physical_cell_id: scan_result.pci as u64,
            rsrp: scan_result.rsrp,
            rsrq: scan_result.rsrq,
            bandwidth: scan_result.bandwidth,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use helium_proto::Message;

    #[test]
    fn scan_roundtrip_proto() {
        let scan_results = CellScan::random();
        let proto: helium_proto::MapperCellScanV1 = scan_results.clone().into();

        let mut proto_bytes = Vec::new();
        proto.encode(&mut proto_bytes).unwrap();
        let scan_results_returned = helium_proto::MapperCellScanV1::decode(proto_bytes.as_slice())
            .unwrap()
            .try_into()
            .unwrap();
        assert_eq!(scan_results, scan_results_returned);
    }
}
