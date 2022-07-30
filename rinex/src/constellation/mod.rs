//! `GNSS` constellations & associated methods
use thiserror::Error;
pub mod augmentation;
use augmentation::Augmentation;

#[cfg(feature = "with-serde")]
use serde::{Serialize, Deserialize};

#[derive(Error, Debug)]
/// Constellation parsing & identification related errors
pub enum Error {
    #[error("code length mismatch, expecting {0} got {1}")]
    CodeLengthMismatch(usize,usize),
    #[error("unknown constellation code \"{0}\"")]
    UnknownCode(String),
}

/// Describes all known `GNSS` constellations
/// when manipulating `RINEX`
#[derive(Clone, Copy, Debug)]
#[derive(PartialEq, Eq)]
#[derive(PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum Constellation {
    /// `GPS` american constellation,
    GPS,
    /// `Glonass` russian constellation
    Glonass,
    /// `BeiDou` chinese constellation
    BeiDou,
    /// `QZSS` japanese constellation
    QZSS,
    /// `Galileo` european constellation
    Galileo,
    /// `SBAS` 
    SBAS(Augmentation),
    /// `IRNSS` constellation
    IRNSS,
    /// `Mixed` for Mixed constellations 
    /// RINEX files description
    Mixed,
}

impl Default for Constellation {
    /// Builds a default `GNSS::GPS` constellation
    fn default() -> Constellation {
        Constellation::GPS
    }
}

impl Constellation {
    /// Identifies `gnss` constellation from given 1 letter code.    
    /// Given code should match official RINEX codes.    
    /// This method is case insensitive though
    pub fn from_1_letter_code (code: &str) -> Result<Constellation, Error> {
        if code.len() != 1 {
            return Err(Error::CodeLengthMismatch(1, code.len()))
        }
        if code.to_lowercase().eq("g") {
            Ok(Constellation::GPS)
        } else if code.to_lowercase().eq("r") {
            Ok(Constellation::Glonass)
        } else if code.to_lowercase().eq("c") {
            Ok(Constellation::BeiDou)
        } else if code.to_lowercase().eq("e") {
            Ok(Constellation::Galileo)
        } else if code.to_lowercase().eq("j") {
            Ok(Constellation::QZSS)
        } else if code.to_lowercase().eq("s") {
            Ok(Constellation::SBAS(Augmentation::default()))
        } else if code.to_lowercase().eq("i") {
            Ok(Constellation::IRNSS)
        } else if code.to_lowercase().eq("m") {
            Ok(Constellation::Mixed)
        } else {
            Err(Error::UnknownCode(code.to_string()))
        }
    }
    /// Converts self to 1 letter code (RINEX standard code)
    pub fn to_1_letter_code (&self) -> &str {
        match self {
            Constellation::GPS => "G",
            Constellation::Glonass => "R",
            Constellation::Galileo => "E",
            Constellation::BeiDou => "C",
            Constellation::SBAS(_) => "S",
            Constellation::QZSS => "J",
            Constellation::IRNSS => "I",
            Constellation::Mixed => "M",
        } 
    }
    /// Identifies `gnss` constellation from given 3 letter code.    
    /// Given code should match official RINEX codes.    
    /// This method is case insensitive though
    pub fn from_3_letter_code (code: &str) -> Result<Constellation, Error> {
        if code.len() != 3 {
            return Err(Error::CodeLengthMismatch(3, code.len()))
        }
        if code.to_lowercase().eq("gps") {
            Ok(Constellation::GPS)
        } else if code.to_lowercase().eq("glo") {
            Ok(Constellation::Glonass)
        } else if code.to_lowercase().eq("bds") {
            Ok(Constellation::BeiDou)
        } else if code.to_lowercase().eq("gal") {
            Ok(Constellation::Galileo)
        } else if code.to_lowercase().eq("qzs") {
            Ok(Constellation::QZSS)
        } else if code.to_lowercase().eq("sbs") {
            Ok(Constellation::SBAS(Augmentation::default()))
        } else if code.to_lowercase().eq("irn") {
            Ok(Constellation::IRNSS)
        } else {
            Err(Error::UnknownCode(code.to_string()))
        }
    }
    /// Converts self to 3 letter code (RINEX standard code)
    pub fn to_3_letter_code (&self) -> &str {
        match self {
            Constellation::GPS => "GPS",
            Constellation::Glonass => "GLO",
            Constellation::Galileo => "GAL",
            Constellation::BeiDou => "BDS",
            Constellation::SBAS(_) => "SBS",
            Constellation::QZSS => "QZS",
            Constellation::IRNSS => "IRN",
            Constellation::Mixed => "MIX",
        } 
    }
    /// Identifies `gnss` constellation from given standard plain name,
    /// like "GPS", or "Galileo". This method is not case sensitive.
    pub fn from_plain_name (code: &str) -> Result<Constellation, Error> {
        if code.to_lowercase().contains("gps") {
            Ok(Constellation::GPS)
        } else if code.to_lowercase().contains("glonass") {
            Ok(Constellation::Glonass)
        } else if code.to_lowercase().contains("galileo") {
            Ok(Constellation::Galileo)
        } else if code.to_lowercase().contains("qzss") {
            Ok(Constellation::QZSS)
        } else if code.to_lowercase().contains("beidou") {
            Ok(Constellation::BeiDou)
        } else if code.to_lowercase().contains("sbas") {
            Ok(Constellation::SBAS(Augmentation::default()))
        } else if code.to_lowercase().contains("irnss") {
            Ok(Constellation::IRNSS)
        } else if code.to_lowercase().contains("mixed") {
            Ok(Constellation::Mixed)
        } else {
            Err(Error::UnknownCode(code.to_string()))
        }
    }
}

impl std::str::FromStr for Constellation {
    type Err = Error;
    /// Identifies `gnss` constellation from given code.   
    /// Code should be standard constellation name,
    /// or official 1/3 letter RINEX code.    
    /// This method is case insensitive
    fn from_str (code: &str) -> Result<Self, Self::Err> {
        if code.len() == 3 {
            Ok(Constellation::from_3_letter_code(code)?)
        } else if code.len() == 1 {
            Ok(Constellation::from_1_letter_code(code)?)
        } else {
            Ok(Constellation::from_plain_name(code)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_from_1_letter_code() {
        let c = Constellation::from_1_letter_code("G");
        assert_eq!(c.is_ok(), true);
        assert_eq!(c.unwrap(), Constellation::GPS);
        
        let c = Constellation::from_1_letter_code("R");
        assert_eq!(c.is_ok(), true);
        assert_eq!(c.unwrap(), Constellation::Glonass);
        
        let c = Constellation::from_1_letter_code("M");
        assert_eq!(c.is_ok(), true);
        assert_eq!(c.unwrap(), Constellation::Mixed);
        
        let c = Constellation::from_1_letter_code("J");
        assert_eq!(c.is_ok(), true);
        assert_eq!(c.unwrap(), Constellation::QZSS);
        
        let c = Constellation::from_1_letter_code("X");
        assert_eq!(c.is_err(), true);
    }
    #[test]
    fn test_from_3_letter_code() {
        let c = Constellation::from_3_letter_code("GPS");
        assert_eq!(c.is_ok(), true);
        assert_eq!(c.unwrap(), Constellation::GPS);
        let c = Constellation::from_3_letter_code("GLO");
        assert_eq!(c.is_ok(), true);
        assert_eq!(c.unwrap(), Constellation::Glonass);
        let c = Constellation::from_3_letter_code("GPX");
        assert_eq!(c.is_err(), true);
        let c = Constellation::from_3_letter_code("X");
        assert_eq!(c.is_err(), true);
    }
    #[test]
    fn test_augmentation() {
        let c = Augmentation::from_str("WAAS");
        assert_eq!(c.is_ok(), true);
        assert_eq!(c.unwrap(), Augmentation::WAAS);
        let c = Augmentation::from_str("WASS");
        assert_eq!(c.is_err(), true);
    }
}
