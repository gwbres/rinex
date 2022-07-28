//! `NavigationData` parser and related methods
use std::io::Write;
use thiserror::Error;
use std::str::FromStr;
use std::collections::{BTreeMap,HashMap};

use crate::sv;
use crate::epoch;
use crate::header;
use crate::version;
use crate::navigation::database;
use crate::constellation::Constellation;
use crate::navigation::database::NAV_MESSAGES;

/// `ComplexEnum` is record payload 
#[derive(Clone, PartialEq, Debug)]
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

#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
pub struct StoMessage {
    
}

/// Possible Navigation Frame declinations for an epoch
#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
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

#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[derive(Eq, Ord)]
#[derive(EnumString)]
/// Navigation Frame for a given epoch
pub enum Frame {
    /// Ephemeris are sorted by Space vehicule,
    /// and by description codes. To determine existing
    /// description coddes, refer to RINEX specifications,
    /// or db/NAV/navigation.json. "clockBias", "clockDrift"
    /// and "clockDriftRate" are the only common description codes.
    /// Other, like "iode", "crs", "cus".. are constellation
    /// and revision dependent.
    Eph(BTreeMap<Sv, HashMap<String, ComplexEnum>>),
    /// System Time Offset message
    Sto(StoMessage),
    /// Earth Orientation Parameters
    Eop(EopMessage),
    /// Ionospheric Model Message
    Ion(IonMessage),
}

impl Frame {
    /// Unwraps self as Ephemeris frame 
    pub fn as_eph (&self) -> Option<&BTreeMap<Sv, HashMap<String, ComplexEnum>>> {
        match self {
            Self::Eph(fr) => Some(fr),
            _ => None,
        }
    }
    /// Unwraps self as System Time Offset frame
    pub fn as_sto (&self) -> Option<&StoMessage> {
        match self {
            Self::Sto(fr) => Some(fr),
            _ => None,
        }
    }
    /// Unwraps self as Earth Orientation parameters frame
    pub fn as_eop (&self) -> Option<&EopMessage> {
        match self {
            Self::Eop(fr) => Some(fr),
            _ => None,
        }
    }
    /// Unwraps self as Ionospheric Model frame
    pub fn as_ion (&self) -> Option<&IonMessage> {
        match self {
            Self::Ion(fr) => Some(fr),
            _ => None,
        }
    }
}

/// Navigation Record.
/// Data is sorted by epoch, and by Frame class.
pub type Record = BTreeMap<epoch::Epoch, BTreeMap<FrameClass, Frame>>;

/// Returns true if given content matches the beginning of a 
/// Navigation record epoch
pub fn is_new_epoch (line: &str, v: version::Version) -> bool {
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
        if sv::Sv::from_str(sv).is_err() {
            return false
        }
        // rest matches a valid epoch descriptor
        let datestr = &line[4..23];
        epoch::str2date(&datestr).is_ok()

    } else { // Modern RINEX --> easy
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
    #[error("failed to parse msg type")]
    SvError(#[from] sv::Error),
    #[error("failed to parse cplx data")]
    ParseComplexError(#[from] ComplexEnumError),
    #[error("failed to parse sv::prn")]
    ParseIntError(#[from] std::num::ParseIntError), 
    #[error("failed to parse date")]
    ParseDateError(#[from] epoch::ParseDateError), 
}

/// Builds `Record` entry for `NavigationData`
pub fn build_record_entry (header: &header::Header, content: &str)
        -> Result<(epoch::Epoch, sv::Sv, HashMap<String, ComplexEnum>), Error>
{
    //  <o 
    //     SV + Epoch + SvClock infos + RecType + MsgType are always there
    //     Other items are constellation dependent => key map
    //     easier to deal with than OBS: 
    //           (*) listing is fixed
    //           (*) nb of items fixed
    let mut lines = content.lines();
    let mut map: HashMap<String, ComplexEnum> = HashMap::new(); 
    let version_major = header.version.major;

    let mut line = lines.next()
        .unwrap();
    
    // might parse a 1st line (V ≥ 4)
    //    [+] RecType + SV + MsgType 
    //                       ^---> newly introduced
    //                   ^-------> deduce constellation identification keys
    //                             as we didn't get this information from file header
    //         ^-----------------> newly introduced
    let (rectype, sv, msgtype): (ComplexEnum, Option<&str>, ComplexEnum)
            = match version_major >= 4 
    {
        true => {
            let items: Vec<&str> = line.split_ascii_whitespace()
                .collect();
            line = lines.next()
                .unwrap();
            (ComplexEnum::new("str", &items[0].trim())?,
            Some(items[1].trim()),
            ComplexEnum::new("str", &items[2].trim())?)
        },
        false => {
            (ComplexEnum::new("str", "EPH")?, // ephemeride, default
            None,
            ComplexEnum::new("str", "LNAV")?) // legacy, default
        },
    };

    map.insert("msg".to_string(), msgtype);
    map.insert("type".to_string(), rectype);

    // parse a 2nd line
    // V < 4
    //    [+] SV + Epoch + SvClock infos
    //         ^-> deduce constellation identification keys
    //             as we didn't get this information from file header
    // V ≥ 4
    //    [+]  Epoch ; SvClock infos
    let (sv, rem) : (&str, &str) = match sv.is_none() {
        true => line.split_at(3), // V < 4
        false => (sv.unwrap(), line), // V ≥ 4
    };

    let (date, rem) = rem.split_at(20);
    let (svbias, rem) = rem.split_at(19);
    let (svdrift, svdriftr) = rem.split_at(19);

    let sv: sv::Sv = match header.constellation {
        // SV identification problem
        //  (+) GLONASS NAV special case
        //      SV'X' is omitted 
        //  (+) faulty RINEX producer with unique constellation
        //      SV'X' is omitted
        Some(Constellation::Mixed) => sv::Sv::from_str(sv.trim())?,
        Some(c) => {
            let prn = u8::from_str_radix(sv.trim(), 10)?;  
            sv::Sv::new(c, prn)
        },
		_ => unreachable!(), // RINEX::NAV body while Type!=NAV
    };

    map.insert("clockBias".to_string(), ComplexEnum::new("f64", svbias.trim())?); 
    map.insert("clockDrift".to_string(), ComplexEnum::new("f64", svdrift.trim())?); 
    map.insert("clockDriftRate".to_string(), ComplexEnum::new("f64", svdriftr.trim())?); 

    line = lines.next()
        .unwrap();

    let mut total: usize = 0; 
    let mut new_line = true;
    
    // NAV database for all following items
    // [1] identify revision for given satellite vehicule constellation
    //     (not using header.revision because it could be Constellation::Mixed
    let db_revision =  database::closest_revision(
        sv.constellation,
        header.version
    );
    if let Some(revision) = db_revision { // contained in db
        // [2] retrieve db items to parse
        let items : Vec<_> = NAV_MESSAGES
            .iter()
            .filter(|r| r.constellation == sv.constellation.to_3_letter_code())
            .map(|r| {
                r.revisions
                    .iter()
                    .filter(|r| // identified db revision
                        u8::from_str_radix(r.major,10).unwrap() == revision.major
                        && u8::from_str_radix(r.minor,10).unwrap() == revision.minor
                    )
                    .map(|r| &r.items)
                    .flatten()
            })
            .flatten()
            .collect();
        // parse items
        for item in items.iter() {
            let (k, v) = item;
            let offset: usize = match new_line {
                false => 19,
                true => {
                    new_line = false;
                    if header.version.major >= 3 {
                        22+1
                    } else {
                        22
                    }
                }
            };
            total += offset;
            let (content, rem) = line.split_at(offset);
            line = rem;
            if !k.eq(&"spare") {
                let rec_item = ComplexEnum::new(v, content.trim())?;
                map.insert(String::from(*k), rec_item);
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
    }
    Ok((
        epoch::Epoch::new(
            epoch::str2date(date)?,
            epoch::EpochFlag::default(), // always, for NAV
        ),
        sv,
        map,
    ))
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
        for (_sv, data) in sv.iter() {
            for (_obs, data) in data.iter() {
                let _ = write!(writer, "{}", data);
            }
            if (index+1)%4 == 0 {
                let _ = write!(writer, "\n    ");
            }
            index += 1
        }
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
}
