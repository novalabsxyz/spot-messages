use super::*;
use helium_proto::{mapper_gps, MapperGps};
use rust_decimal::Decimal;

pub const ZERO_DECIMAL: Decimal = Decimal::from_parts(0, 0, 0, false, 0);

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Gps {
    /// UTC of position fix
    pub timestamp: DateTime<Utc>,
    /// Latitude in degrees
    pub lat: Decimal,
    /// Longitude in degrees
    pub lon: Decimal,
    /// Horizontal dilution of position
    pub hdop: Decimal,
    /// Height of geoid (mean sea level) above WGS84 ellipsoid
    pub altitude: Decimal,
    /// Number of satellites in use
    pub num_sats: u8,
    /// Speed over ground (SoG), km/h
    pub speed: Decimal,
}

pub use h3o::Resolution;

impl Gps {
    pub fn is_locked(&self) -> bool {
        self.num_sats >= 3 && self.hdop > ZERO_DECIMAL
    }

    pub fn to_h3_cell(&self, r: h3o::Resolution) -> Result<h3o::CellIndex> {
        use rust_decimal::prelude::ToPrimitive;
        match (self.lat.to_f64(), self.lon.to_f64()) {
            (Some(lat), Some(lon)) => {
                let coord = h3o::LatLng::new(lat, lon)?;
                Ok(coord.to_cell(r))
            }
            (None, _) => Err(Error::DecimalCouldNotMapToFloat { decimal: self.lat }),
            (_, None) => Err(Error::DecimalCouldNotMapToFloat { decimal: self.lon }),
        }
    }

    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let timestamp = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 5).unwrap();
        Gps {
            timestamp,
            lat: Decimal::new(rng.gen_range(-90_00000..90_00000), 5),
            lon: Decimal::new(rng.gen_range(-180_00000..180_00000), 5),
            hdop: Decimal::new(rng.gen_range(0..10_00), 2),
            //// WGS-84 on the surface of earth ranges from +85m (Iceland) to -106m (India)
            altitude: Decimal::new(rng.gen_range(-10_600..8_500), 2),
            num_sats: rng.gen_range(0..12),
            speed: Decimal::new(rng.gen_range(0..50_00), 2),
        }
    }

    /// provides a rounded GPS value for easy roundtrip testing to lorawan payloads
    pub fn rounded() -> Self {
        let timestamp = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 5).unwrap();
        Gps {
            timestamp,
            lat: Decimal::new(-50_12345, 5),
            lon: Decimal::new(120_12345, 5),
            hdop: Decimal::new(9_05, 2),
            altitude: Decimal::new(9_25, 2),
            num_sats: 5,
            speed: Decimal::new(50_50, 2),
        }
    }
}

impl From<Gps> for helium_proto::MapperGpsV1 {
    fn from(gps_data: Gps) -> helium_proto::MapperGpsV1 {
        helium_proto::MapperGpsV1 {
            timestamp: time::to_proto_units(gps_data.timestamp),
            lat: latlon::to_proto_units(gps_data.lat),
            lon: latlon::to_proto_units(gps_data.lon),
            hdop: hdop::to_units(gps_data.hdop),
            altitude: altitude::to_proto_units(gps_data.altitude),
            num_sats: gps_data.num_sats as u32,
            speed: speed::to_proto_units(gps_data.speed),
        }
    }
}

impl From<helium_proto::MapperGpsV1> for Gps {
    fn from(gps_proto: helium_proto::MapperGpsV1) -> Gps {
        Gps {
            timestamp: time::from_proto_units(gps_proto.timestamp),
            lat: latlon::from_proto_units(gps_proto.lat),
            lon: latlon::from_proto_units(gps_proto.lon),
            hdop: hdop::from_units(gps_proto.hdop),
            altitude: altitude::from_proto_units(gps_proto.altitude),
            num_sats: gps_proto.num_sats as u8,
            speed: speed::from_proto_units(gps_proto.speed),
        }
    }
}

impl TryFrom<MapperGps> for Gps {
    type Error = Error;

    fn try_from(proto: MapperGps) -> Result<Self> {
        if let Some(mapper_gps::Version::GpsV1(proto)) = proto.version {
            Ok(proto.into())
        } else {
            Err(Error::ProtoHasNone("version"))
        }
    }
}

impl From<Gps> for mapper_payload::Message {
    fn from(gps: Gps) -> Self {
        mapper_payload::Message::Gps(MapperGps {
            version: Some(mapper_gps::Version::GpsV1(gps.into())),
        })
    }
}

pub mod hdop {
    use super::*;

    pub fn to_units(hdop: Decimal) -> u32 {
        let multiplier = Decimal::new(100, 0);
        let scaled = hdop.checked_mul(multiplier).unwrap().round();
        scaled.to_string().parse::<u32>().unwrap()
    }

    pub(crate) fn from_units(hdop: u32) -> Decimal {
        Decimal::new(hdop.into(), 2)
    }
}

pub mod time {
    use super::*;
    // time for 2023-01-01 00:00:00 UTC
    const REFERENCE: i64 = 1672531200;

    pub(crate) fn to_lora_units(datetime: DateTime<Utc>) -> u32 {
        (datetime.timestamp() - REFERENCE) as u32
    }

    pub(crate) fn from_lora_units(timestamp: u32) -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp_opt(timestamp as i64 + REFERENCE, 0).unwrap(),
            Utc,
        )
    }

    pub fn to_proto_units(datetime: DateTime<Utc>) -> u64 {
        datetime.timestamp() as u64
    }

    pub fn from_proto_units(timestamp: u64) -> DateTime<Utc> {
        DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp_opt(timestamp as i64, 0).unwrap(),
            Utc,
        )
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use chrono::TimeZone;

        #[test]
        fn time_to_lora_units() {
            let datetime = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 5).unwrap();
            assert_eq!(to_lora_units(datetime), 5);
        }

        #[test]
        fn time_from_lora_units() {
            let datetime = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 5).unwrap();
            assert_eq!(from_lora_units(5), datetime);
        }

        #[test]
        fn time_to_proto_units() {
            let datetime = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 5).unwrap();
            assert_eq!(to_proto_units(datetime), 1672531205);
        }

        #[test]
        fn time_from_proto_units() {
            let datetime = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 5).unwrap();
            assert_eq!(from_proto_units(1672531205), datetime);
        }
    }
}

pub mod latlon {
    use super::*;

    const LAT_OFFSET: Decimal = Decimal::from_parts(9000000, 0, 0, false, 5);
    const LON_OFFSET: Decimal = Decimal::from_parts(18000000, 0, 0, false, 5);

    pub(crate) enum Degrees {
        Lat(Decimal),
        Lon(Decimal),
    }

    pub(crate) enum Unit {
        Lat(u32),
        Lon(u32),
    }

    pub(crate) fn to_lora_units(coordinate: Degrees) -> u32 {
        let offset_degrees = match coordinate {
            Degrees::Lat(lat) => lat + LAT_OFFSET,
            Degrees::Lon(lon) => lon + LON_OFFSET,
        };
        let multiplier = Decimal::new(100000, 0);
        let scaled = offset_degrees.checked_mul(multiplier).unwrap().round();
        scaled.to_string().parse::<u32>().unwrap()
    }

    pub(crate) fn from_lora_units(unit: Unit) -> Decimal {
        match unit {
            // (-90, 90)
            Unit::Lat(lat) => Decimal::new(lat.into(), 5) - LAT_OFFSET,
            // (-180, 180)
            Unit::Lon(lon) => Decimal::new(lon.into(), 5) - LON_OFFSET,
        }
    }

    pub fn to_proto_units(coordinate: Decimal) -> i32 {
        let multiplier = Decimal::new(100000, 0);
        let scaled = coordinate.checked_mul(multiplier).unwrap().round();
        scaled.to_string().parse::<i32>().unwrap()
    }

    pub fn from_proto_units(unit: i32) -> Decimal {
        Decimal::new(unit.into(), 5)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use rand::Rng;

        #[test]
        fn lat_constant() {
            assert_eq!(LAT_OFFSET.to_string(), "90.00000");
        }

        #[test]
        fn lon_constant() {
            assert_eq!(LON_OFFSET.to_string(), "180.00000");
        }

        #[test]
        fn roundtrip_lat_lora() {
            let mut rng = rand::thread_rng();
            let random_lat = rng.gen_range(-90_00000..90_00000);
            let lat = Decimal::new(random_lat, 5);
            let units = to_lora_units(Degrees::Lat(lat));
            let degrees = from_lora_units(Unit::Lat(units));
            assert_eq!(lat, degrees);
        }

        #[test]
        fn roundtrip_lon_lora() {
            let mut rng = rand::thread_rng();
            let random_lon = rng.gen_range(-180_00000..180_00000);
            let lon = Decimal::new(random_lon, 5);
            let units = to_lora_units(Degrees::Lon(lon));
            let degrees = from_lora_units(Unit::Lon(units));
            assert_eq!(lon, degrees);
        }

        #[test]
        fn roundtrip_latlon_proto() {
            let mut rng = rand::thread_rng();
            let random_lat = rng.gen_range(-90_00000..90_00000);
            let lat = Decimal::new(random_lat, 5);
            let units = to_proto_units(lat);
            let degrees = from_proto_units(units);
            assert_eq!(lat, degrees);
        }
    }
}

pub mod altitude {
    use super::*;
    #[allow(clippy::inconsistent_digit_grouping)]
    const ALTITUDE_OFFSET: Decimal = Decimal::from_parts(110_00, 0, 0, false, 2);
    #[allow(clippy::zero_prefixed_literal, clippy::inconsistent_digit_grouping)]
    const ALTITUDE_LORA_SCALAR: Decimal = Decimal::from_parts(0_25, 0, 0, false, 2);
    #[allow(clippy::zero_prefixed_literal, clippy::inconsistent_digit_grouping)]
    const ALTITUDE_PROTO_SCALAR: Decimal = Decimal::from_parts(1, 0, 0, false, 2);

    pub(crate) fn to_lora_units(altitude: Decimal) -> u32 {
        let offset_altitude = altitude + ALTITUDE_OFFSET;
        let scaled = offset_altitude
            .checked_div(ALTITUDE_LORA_SCALAR)
            .unwrap()
            .round();
        scaled.to_string().parse::<u32>().unwrap()
    }

    pub(crate) fn from_lora_units(altitude: u32) -> Decimal {
        let altitude_unscaled = Decimal::new(altitude.into(), 0);
        let altitude_unoffset = altitude_unscaled.checked_mul(ALTITUDE_LORA_SCALAR).unwrap();
        altitude_unoffset - ALTITUDE_OFFSET
    }

    pub fn to_proto_units(altitude: Decimal) -> i32 {
        let scaled = altitude.checked_div(ALTITUDE_PROTO_SCALAR).unwrap().round();
        scaled.to_string().parse::<i32>().unwrap()
    }

    pub fn from_proto_units(altitude: i32) -> Decimal {
        let altitude_unscaled = Decimal::new(altitude.into(), 0);
        altitude_unscaled
            .checked_mul(ALTITUDE_PROTO_SCALAR)
            .unwrap()
    }

    #[cfg(test)]
    mod test {
        use super::*;
        #[test]
        fn altitude_offset() {
            assert_eq!(ALTITUDE_OFFSET.to_string(), "110.00");
        }

        #[test]
        fn altitude_lower_limit_roundtrip_lora() {
            let altitude = Decimal::new(-110_00, 2);
            assert_eq!(altitude.to_string(), "-110.00");
            let units = to_lora_units(altitude);
            assert_eq!(0, units);
            let altitude = from_lora_units(units);
            assert_eq!(altitude.to_string(), "-110.00");
        }

        #[test]
        fn altitude_zero_roundtrip_lora() {
            let altitude = Decimal::new(0, 2);
            assert_eq!(altitude.to_string(), "0.00");
            let units = to_lora_units(altitude);
            assert_eq!(110_00 / 25, units);
            let altitude = from_lora_units(units);
            assert_eq!(altitude.to_string(), "0.00");
        }

        #[test]
        fn altitude_round_down_lora() {
            let altitude = Decimal::new(10_12, 2);
            assert_eq!(altitude.to_string(), "10.12");
            let altitude = from_lora_units(to_lora_units(altitude));
            assert_eq!(altitude.to_string(), "10.00");
        }

        #[test]
        fn altitude_round_up_lora() {
            let altitude = Decimal::new(10_21, 2);
            assert_eq!(altitude.to_string(), "10.21");
            let altitude = from_lora_units(to_lora_units(altitude));
            assert_eq!(altitude.to_string(), "10.25");
        }
    }
}

pub mod speed {
    use super::*;
    #[allow(clippy::zero_prefixed_literal, clippy::inconsistent_digit_grouping)]
    const SPEED_LORA_SCALAR: Decimal = Decimal::from_parts(0_25, 0, 0, false, 2);
    const SPEED_PROTO_SCALAR: Decimal = Decimal::from_parts(1, 0, 0, false, 2);

    pub(crate) fn to_lora_units(speed: Decimal) -> u32 {
        let scaled = speed.checked_div(SPEED_LORA_SCALAR).unwrap().round();
        scaled.to_string().parse::<u32>().unwrap()
    }

    pub(crate) fn from_lora_units(speed: u32) -> Decimal {
        let speed_unscaled = Decimal::new(speed.into(), 0);
        speed_unscaled.checked_mul(SPEED_LORA_SCALAR).unwrap()
    }

    pub fn to_proto_units(speed: Decimal) -> u32 {
        let scaled = speed.checked_div(SPEED_PROTO_SCALAR).unwrap().round();
        scaled.to_string().parse::<u32>().unwrap()
    }

    pub fn from_proto_units(speed: u32) -> Decimal {
        let speed_unscaled = Decimal::new(speed.into(), 0);
        speed_unscaled.checked_mul(SPEED_PROTO_SCALAR).unwrap()
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn speed_upper_limit_roundtrip_lora() {
            let speed = Decimal::new(80_00, 2);
            assert_eq!(speed.to_string(), "80.00");
            let units = to_lora_units(speed);
            assert_eq!(80_00 / 25, units);
            let speed = from_lora_units(units);
            assert_eq!(speed.to_string(), "80.00");
        }

        #[test]
        fn speed_round_down_lora() {
            let altitude = Decimal::new(20_12, 2);
            assert_eq!(altitude.to_string(), "20.12");
            let altitude = from_lora_units(to_lora_units(altitude));
            assert_eq!(altitude.to_string(), "20.00");
        }

        #[test]
        fn speed_round_up_lora() {
            let altitude = Decimal::new(20_13, 2);
            assert_eq!(altitude.to_string(), "20.13");
            let altitude = from_lora_units(to_lora_units(altitude));
            assert_eq!(altitude.to_string(), "20.25");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use helium_proto::Message;

    #[test]
    fn gps_roundtrip_proto() {
        let gps = Gps::rounded();
        let proto: helium_proto::MapperGpsV1 = gps.clone().into();
        let mut proto_bytes = Vec::new();
        proto.encode(&mut proto_bytes).unwrap();
        let gps_returned = helium_proto::MapperGpsV1::decode(proto_bytes.as_slice())
            .unwrap()
            .into();
        assert_eq!(gps, gps_returned);
    }
}
