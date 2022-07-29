//! `NavigationData` parser and related methods
use std::io::Write;
use thiserror::Error;
use std::str::FromStr;
use strum_macros::EnumString;
use std::collections::{BTreeMap, HashMap};

use crate::epoch;
use crate::header;
use crate::sv;
use crate::sv::Sv;
use crate::epoch::{Epoch, ParseDateError};
use crate::version::Version;
use crate::navigation::database;
use crate::constellation::Constellation;
use crate::navigation::database::NAV_MESSAGES;
use crate::navigation::ionmessage;

/// `ComplexEnum` is record payload 
#[derive(Clone, Debug)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum ComplexEnum {
    U8(u8),
    Str(String), 
    F32(f32),
    F64(f64),
}

impl std::fmt::Display for ComplexEnum {
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ComplexEnum::U8(u)  => write!(fmt, "{:X}", u),
            ComplexEnum::Str(s) => write!(fmt, "{}", s),
            ComplexEnum::F32(f) => write!(fmt, "{:.10e}", f),
            ComplexEnum::F64(f) => write!(fmt, "{:.10e}", f),
        }
    }
}

/// `ComplexEnum` related errors
#[derive(Error, Debug)]
pub enum ComplexEnumError {
    #[error("failed to parse int value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("unknown type descriptor \"{0}\"")]
    UnknownTypeDescriptor(String),
}

impl ComplexEnum {
    /// Builds a `ComplexEnum` from type descriptor and string content
    pub fn new (desc: &str, content: &str) -> Result<ComplexEnum, ComplexEnumError> {
        match desc {
            "f32" => {
                Ok(ComplexEnum::F32(f32::from_str(&content.replace("D","e"))?))
            },
            "f64" => {
                Ok(ComplexEnum::F64(f64::from_str(&content.replace("D","e"))?))
            },
            "u8" => {
                Ok(ComplexEnum::U8(u8::from_str_radix(&content, 16)?))
            },
            "str" => {
                Ok(ComplexEnum::Str(String::from(content)))
            },
            _ => Err(ComplexEnumError::UnknownTypeDescriptor(desc.to_string())),
        }
    }
    pub fn as_f32 (&self) -> Option<f32> {
        match self {
            ComplexEnum::F32(f) => Some(f.clone()),
            _ => None,
        }
    }
    pub fn as_f64 (&self) -> Option<f64> {
        match self {
            ComplexEnum::F64(f) => Some(f.clone()),
            _ => None,
        }
    }
    pub fn as_str (&self) -> Option<String> {
        match self {
            ComplexEnum::Str(s) => Some(s.clone()),
            _ => None,
        }
    }
    pub fn as_u8 (&self) -> Option<u8> {
        match self {
            ComplexEnum::U8(u) => Some(u.clone()),
            _ => None,
        }
    }
}

/// Possible Navigation Frame declinations for an epoch
#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[derive(Eq, Ord)]
#[derive(EnumString)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum FrameClass {
    #[strum(serialize = "EPH", deserialize = "EPH")]
    Ephemeris,
    #[strum(serialize = "STO", deserialize = "STO")]
    SystemTimeOffset,
    #[strum(serialize = "EOP", deserialize = "EOP")]
    EarthOrientation,
    #[strum(serialize = "ION", deserialize = "ION")]
    IonosphericModel,
}

impl Default for FrameClass {
    fn default() -> Self{
        Self::Ephemeris
    }
}

impl std::fmt::Display for FrameClass {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Ephemeris => f.write_str("EPH"),
            Self::SystemTimeOffset => f.write_str("STO"),
            Self::EarthOrientation => f.write_str("EOP"),
            Self::IonosphericModel => f.write_str("ION"),
        }
    }
}


/// Navigation Message Types 
#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[derive(Eq, Ord)]
#[derive(EnumString)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum MsgType {
    /// Legacy NAV
    LNAV,
    /// FDMA
    FDMA,
    /// IFNV,
    IFNV,
    /// D1D2,
    D1D2,
    /// SBAS
    SBAS,
    /// CNVX special marker
    CNVX,
}

impl Default for MsgType {
    fn default() -> Self {
        Self::LNAV
    }
}

/// Navigation Frame for a given epoch
#[derive(Debug, Clone)]
#[derive(EnumString)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum Frame {
    /// Ephemeris for a given Vehicule `Sv`,
    /// with vehicule internal clock bias, clock drift and clock drift rate.
    /// Rest of data is constellation dependent, see
    /// RINEX specifications or db/NAV/navigation.json.
    Eph(MsgType, Sv, f64, f64, f64, HashMap<String, ComplexEnum>),
    /*
    /// System Time Offset message
    Sto(StoMessage),
    /// Earth Orientation Parameters
    Eop(EopMessage), */
    /// Ionospheric Model Message
    Ion(ionmessage::Message),
}

impl Frame {
    /// Unwraps self as Ephemeris frame
    pub fn as_eph (&self) -> Option<(&MsgType, &Sv, &f64, &f64, &f64, &HashMap<String, ComplexEnum>)> {
        match self {
            Self::Eph(msg, sv, clk, clk_dr, clk_drr, map) => Some((msg, sv, clk, clk_dr, clk_drr, map)),
            _ => None,
        }
    }
    /// Unwraps self as Ephemeris frame
    pub fn as_mut_eph (&mut self) -> Option<(&mut MsgType, &mut Sv, &mut f64, &mut f64, &mut f64, &mut HashMap<String, ComplexEnum>)> {
        match self {
            Self::Eph(msg, sv, clk, clk_dr, clk_drr, map) => Some((msg, sv, clk, clk_dr, clk_drr, map)),
            _ => None,
        }
    }
    /// Unwraps self as Ionospheric Model frame
    pub fn as_ion (&self) -> Option<&ionmessage::Message> {
        match self {
            Self::Ion(fr) => Some(fr),
            _ => None,
        }
    }
    /// Unwraps self as Ionospheric Model frame
    pub fn as_mut_ion (&mut self) -> Option<&mut ionmessage::Message> {
        match self {
            Self::Ion(fr) => Some(fr),
            _ => None,
        }
    }
}

/// Navigation Record.
/// Data is sorted by epoch, and by Frame class.
pub type Record = BTreeMap<Epoch, BTreeMap<FrameClass, Frame>>;

/// Returns true if given content matches the beginning of a 
/// Navigation record epoch
pub fn is_new_epoch (line: &str, v: Version) -> bool {
    if v.major < 3 { // old RINEX
        if line.len() < 23 {
            return false // not enough bytes 
                // to describe a PRN and an Epoch
        }
        let (prn, _) = line.split_at(2);
        // 1st entry is a valid integer number
        if u8::from_str_radix(prn.trim(), 10).is_err() {
            return false
        }
        // rest matches a valid epoch descriptor
        let datestr = &line[3..22];
        epoch::str2date(&datestr).is_ok()

    } else if v.major == 3 { // RINEX V3
        if line.len() < 24 {
            return false // not enough bytes
                // to describe an SVN and an Epoch
        }
        // 1st entry matches a valid SV description
        let (sv, _) = line.split_at(4);
        if Sv::from_str(sv).is_err() {
            return false
        }
        // rest matches a valid epoch descriptor
        let datestr = &line[4..23];
        epoch::str2date(&datestr).is_ok()

    } else { // Modern --> easy 
        if let Some(c) = line.chars().nth(0) {
            c == '>' // new epoch marker 
        } else {
            false
        }
    }
}

/// Navigation Record Parsing Error
#[derive(Error, Debug)]
pub enum Error {
    #[error("epoch is missing data")]
    MissingData,
    #[error("failed to locate revision in db")]
    DataBaseRevisionError,
    #[error("failed to parse msg type")]
    SvError(#[from] sv::Error),
    #[error("failed to parse cplx data")]
    ParseComplexError(#[from] ComplexEnumError),
    #[error("failed to parse sv::prn")]
    ParseIntError(#[from] std::num::ParseIntError), 
    #[error("failed to parse sv clock fields")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse epoch date")]
    ParseDateError(#[from] ParseDateError),
}

/// Builds `Record` entry for `NavigationData`
pub fn build_record_entry (version: Version, constell: Constellation, content: &str) ->
        Result<(Epoch, FrameClass, Frame), Error>
{
    if content.starts_with(">") {
        build_modern_record_entry(version, constell, content)
    } else {
        build_v2_v3_record_entry(version, constell, content)
    }
}

/// Builds `Record` entry for Modern NAV frames
fn build_modern_record_entry (version: Version, constell: Constellation, content: &str) ->
        Result<(Epoch, FrameClass, Frame), Error>
{
    panic!("NAV4 : not yet")
}

/// Builds `Record` entry for Old NAV frames
fn build_v2_v3_record_entry (version: Version, constell: Constellation, content: &str) ->
        Result<(Epoch, FrameClass, Frame), Error>
{
    let mut lines = content.lines();
    let line = match lines.next() {
        Some(l) => l,
        _ => return Err(Error::MissingData), 
    };
    
    let version_major = version.major;
    let svnn_offset :usize = match version.major {
        1|2 => 3,
        3 => 4,
        _ => unreachable!(),
    };

    let (svnn, rem) = line.split_at(svnn_offset);
    let (date, rem) = rem.split_at(20);
    let (clk_bias, rem) = rem.split_at(19);
    let (clk_dr, clk_drr) = rem.split_at(19);

    println!("V2 V3 | 1ST LINE | SVNN \"{}\" DATE \"{}\" BIAS \"{}\" DRIFT \"{}\" DR \"{}\"",
        svnn, date, clk_bias, clk_dr, clk_drr);

    let sv : Sv = match version.major {
        1|2 => {
            Sv {
                constellation: constell,
                prn: u8::from_str_radix(svnn, 10)?,
            }
        },
        3 => Sv::from_str(svnn)?,
        _ => unreachable!(),
    };

    let clk = f64::from_str(clk_bias.trim())?;
    let clk_dr = f64::from_str(clk_dr.trim())?;
    let clk_drr = f64::from_str(clk_drr.trim())?;
    let map = parse_complex_map(version, constell, lines)?;
    let fr = Frame::Eph(MsgType::LNAV, sv, clk, clk_dr, clk_drr, map); // indicate legacy frame
    Ok((
        epoch::Epoch::new(
            epoch::str2date(date)?,
            epoch::EpochFlag::default(), // flag never given in NAV 
        ),
        FrameClass::Ephemeris, // legacy: Only Ephemeris exist
        fr, // ephemeris frame
    ))
}

/// Parses constellation + revision dependent complex map 
fn parse_complex_map (version: Version, constell: Constellation, mut lines: std::str::Lines<'_>) 
        -> Result<HashMap<String, ComplexEnum>, Error>
{
    // locate closest revision in db
    let db_revision = match database::closest_revision(constell, version) {
        Some(v) => v,
        _ => return Err(Error::DataBaseRevisionError),
    };

    // retrieve db items / fields to parse
    let items :Vec<_> = NAV_MESSAGES
        .iter()
        .filter(|r| r.constellation == constell.to_3_letter_code())
        .map(|r| {
            r.revisions
                .iter()
                .filter(|r| // identified db revision
                    u8::from_str_radix(r.major, 10).unwrap() == db_revision.major
                    && u8::from_str_radix(r.minor, 10).unwrap() == db_revision.minor
                )
                .map(|r| &r.items)
                .flatten()
        })
        .flatten()
        .collect();

    // parse items
    let mut line = match lines.next() {
        Some(l) => l,
        _ => return Err(Error::MissingData),
    };
    let mut new_line = true;
    let mut total :usize = 0;
    let mut map :HashMap<String, ComplexEnum> = HashMap::new();
    for item in items.iter() {
        let (k, v) = item;
        let offset :usize = match new_line {
            false => 19,
            true => {
                new_line = false;
                if version.major == 3 {
                    22+1
                } else {
                    22
                }
            },
        };
        total += offset;
        let (content, rem) = line.split_at(offset);
        line = rem.clone();

        if k.eq(&"spare") { // --> got something to parse in db
            let cplx = ComplexEnum::new(v, content.trim())?;
            map.insert(k.to_string(), cplx);
        }

        if total >= 76 {
            new_line = true;
            total = 0;
            if let Some(l) = lines.next() {
                line = l;
            } else {
                break
            }
        }
    }
    Ok(map)
}


/// Pushes observation record into given file writer
pub fn to_file (header: &header::Header, record: &Record, mut writer: std::fs::File) -> std::io::Result<()> {
    for (epoch, sv) in record.iter() {
        let nb_sv = sv.keys().len();
        match header.version.major {
            1|2 => {
                let _ = write!(writer, " {} {} ", nb_sv, epoch.date.format("%y %m %d %H %M %.6f").to_string());
            },
            _ => {
                let _ = write!(writer, "> {} {} ", nb_sv, epoch.date.format("%Y %m %d %H %M %.6f").to_string());
            }
        }
        let mut index = 1;
        /*for (_sv, data) in sv.iter() {
            for (_obs, data) in data.iter() {
                let _ = write!(writer, "{}", data);
            }
            if (index+1)%4 == 0 {
                let _ = write!(writer, "\n    ");
            }
            index += 1
        }*/
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_complex_enum() {
        let e = ComplexEnum::U8(10);
        assert_eq!(e.as_u8().is_some(), true);
        assert_eq!(e.as_f32().is_some(), false);
        assert_eq!(e.as_str().is_some(), false);
        let u = e.as_u8().unwrap();
        assert_eq!(u, 10);
        
        let e = ComplexEnum::Str(String::from("Hello World"));
        assert_eq!(e.as_u8().is_some(), false);
        assert_eq!(e.as_f32().is_some(), false);
        assert_eq!(e.as_str().is_some(), true);
        let u = e.as_str().unwrap();
        assert_eq!(u, "Hello World");
        
        let e = ComplexEnum::F32(10.0);
        assert_eq!(e.as_u8().is_some(), false);
        assert_eq!(e.as_f32().is_some(), true);
        assert_eq!(e.as_str().is_some(), false);
        let u = e.as_f32().unwrap();
        assert_eq!(u, 10.0_f32);
        
        let e = ComplexEnum::F64(10.0);
        assert_eq!(e.as_u8().is_some(), false);
        assert_eq!(e.as_f32().is_some(), false);
        assert_eq!(e.as_f64().is_some(), true);
        assert_eq!(e.as_str().is_some(), false);
        let u = e.as_f64().unwrap();
        assert_eq!(u, 10.0_f64);
    }
    #[test]
    fn test_is_new_epoch() {
        // NAV V<3
        let line = " 1 20 12 31 23 45  0.0 7.282570004460D-05 0.000000000000D+00 7.380000000000D+04";
        assert_eq!(is_new_epoch(line, version::Version::new(1, 0)), true);
        assert_eq!(is_new_epoch(line, version::Version::new(2, 0)), true);
        assert_eq!(is_new_epoch(line, version::Version::new(3, 0)), false);
        assert_eq!(is_new_epoch(line, version::Version::new(4, 0)), false);
        // NAV V<3
        let line = " 2 21  1  1 11 45  0.0 4.610531032090D-04 1.818989403550D-12 4.245000000000D+04";
        assert_eq!(is_new_epoch(line, version::Version::new(1, 0)), true);
        assert_eq!(is_new_epoch(line, version::Version::new(2, 0)), true);
        assert_eq!(is_new_epoch(line, version::Version::new(3, 0)), false);
        assert_eq!(is_new_epoch(line, version::Version::new(4, 0)), false);
        // GPS NAV V<3
        let line = " 3 17  1 13 23 59 44.0-1.057861372828D-04-9.094947017729D-13 0.000000000000D+00";
        assert_eq!(is_new_epoch(line, version::Version::new(1, 0)), true);
        assert_eq!(is_new_epoch(line, version::Version::new(2, 0)), true);
        assert_eq!(is_new_epoch(line, version::Version::new(3, 0)), false);
        assert_eq!(is_new_epoch(line, version::Version::new(4, 0)), false);
        // NAV V3
        let line = "C05 2021 01 01 00 00 00-4.263372393325e-04-7.525180478751e-11 0.000000000000e+00";
        assert_eq!(is_new_epoch(line, version::Version::new(1, 0)), false);
        assert_eq!(is_new_epoch(line, version::Version::new(2, 0)), false);
        assert_eq!(is_new_epoch(line, version::Version::new(3, 0)), true);
        assert_eq!(is_new_epoch(line, version::Version::new(4, 0)), false);
        // NAV V3
        let line = "R21 2022 01 01 09 15 00-2.666609361768E-04-2.728484105319E-12 5.508000000000E+05";
        assert_eq!(is_new_epoch(line, version::Version::new(1, 0)), false);
        assert_eq!(is_new_epoch(line, version::Version::new(2, 0)), false);
        assert_eq!(is_new_epoch(line, version::Version::new(3, 0)), true);
        assert_eq!(is_new_epoch(line, version::Version::new(4, 0)), false);
        // NAV V4
        let line = "> EPH G02 LNAV";
        assert_eq!(is_new_epoch(line, version::Version::new(2, 0)), false);
        assert_eq!(is_new_epoch(line, version::Version::new(3, 0)), false);
        assert_eq!(is_new_epoch(line, version::Version::new(4, 0)), true);
    }
    #[test]
    fn test_old_record_entry() {
        let entry = build_record_entry(content);
    }
}
