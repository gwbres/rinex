//! Carrier channels and associated methods 
use thiserror::Error;
use std::str::FromStr;
use crate::sv;
use crate::constellation::Constellation;

/*
/// Carrier code
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum Code {
    /// GPS/GLONASS/QZSS/SBAS L1 C/A,
    C1, 
    /// GPS/GLONASS L1P
    P1,
    /// BeiDou B1i
    B1,
    /// Galileo E1
    E1,
    /// GPS / QZSS L2C
    C2, 
    /// GPS / GLONASS L2P
    P2,
    /// BeiDou B2i
    B2,
    /// Galileo E5
    E5,
}

#[derive(Debug)]
pub enum CodeError {
    /// Unknown Carrier code identifier
    UnknownCode(String),
}

impl std::str::FromStr for Code {
    type Err = CodeError;
    fn from_str (code: &str) -> Result<Code, CodeError> {
        if code.eq("C1") {
            Ok(Code::C1)
        } else if code.eq("C2") {
            Ok(Code::C2)
        } else if code.contains("P1") {
            Ok(Code::P1)
        } else if code.contains("P2") {
            Ok(Code::P2)
        } else if code.contains("B1") | code.eq("B1i") {
            Ok(Code::B1)
        } else if code.eq("B2") | code.eq("B2i") {
            Ok(Code::B2)
        } else if code.eq("E1") {
            Ok(Code::E1)
        } else if code.eq("E5") | code.eq("E5a") {
            Ok(Code::E5)
        } else {
            Err(CodeError::UnknownCode(code.to_string()))
        }
    }
}

impl std::fmt::Display for Code {
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Code::C1 => fmt.write_str("C1"),
            Code::C2 => fmt.write_str("C2"),
            Code::P1 => fmt.write_str("P1"),
            Code::P2 => fmt.write_str("P2"),
            Code::B1 => fmt.write_str("B1"),
            Code::B2 => fmt.write_str("B2"),
            Code::E1 => fmt.write_str("E1"),
            Code::E5 => fmt.write_str("E5"),
        }
    }
}

impl Default for Code {
    /// Builds `C1` as default code
    fn default() -> Code {
        Code::C1
    }
}
*/

#[derive(Debug, Clone, Copy)]
#[derive(PartialEq, PartialOrd)]
pub enum Channel {
    /// L1 (GPS, SBAS, QZSS)
    L1,
    /// L2 (GPS, QZSS)
    L2,
    /// L5 (GPS, SBAS), QZSS 
    L5,
    /// LEX (QZSS)
    LEX, 
    /// Glonass channel 1 with possible channel offset
    G1(Option<u8>),
    /// Glonass channel 2 with possible channel offset
    G2(Option<u8>),
    /// E1: GAL
    E1,
    /// E2: GAL
    E2,
    /// E5: GAL E5a + E5b
    E5, 
    /// E6: GAL military
    E6
}

impl Default for Channel {
    fn default() -> Channel {
        Channel::L1
    }
}

#[derive(Error, Debug)]
pub enum Error {
    /// Unable to parse Channel from given string content
    #[error("unable to parse channel from content \"{0}\"")]
    ParseError(String),
    #[error("unable to identify glonass channel from \"{0}\"")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("unable to identify constellation + carrier code")]
    SvError(#[from] sv::Error),
    #[error("non recognized observable \"{0}\"")]
    InvalidObservable(String),
}

impl std::str::FromStr for Channel {
    type Err = Error; 
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.contains("L1") { 
            Ok(Channel::L1)
        } else if s.contains("L2") {
            Ok(Channel::L2)
        } else if s.contains("L5") {
            Ok(Channel::L5)
        
        } else if s.contains("G1") {
            if s.eq("G1") {
                Ok(Channel::G1(None))
            } else if s.contains("G1(") {
                let items : Vec<&str> = s.split("(").collect();
                let item = items[1].replace(")","");
                Ok(Channel::G1(
                    Some(u8::from_str_radix(&item, 10)?)))
            } else {
                Err(Error::ParseError(s.to_string()))
            }
        
        } else if s.contains("G2") {
            if s.eq("G2") {
                Ok(Channel::G2(None))
            } else if s.contains("G2(") {
                let items : Vec<&str> = s.split("(").collect();
                let item = items[1].replace(")","");
                Ok(Channel::G2(
                    Some(u8::from_str_radix(&item, 10)?)))
            } else {
                Err(Error::ParseError(s.to_string()))
            }

        } else {
            Err(Error::ParseError(s.to_string())) 
        }
    }
}

impl Channel {
    /// Returns frequency associated to this channel in MHz 
    pub fn carrier_frequency_mhz (&self) -> f64 {
        match self {
            Channel::L1 | Channel::E1 => 1575.42_f64,
            Channel::L2 | Channel::E2 => 1227.60_f64,
            Channel::L5 | Channel::E5 => 1176.45_f64,
            Channel::G1(Some(c)) => 1602.0_f64 + (*c as f64 *9.0/16.0), 
            Channel::G1(_) => 1602.0_f64,
            Channel::G2(Some(c)) => 1246.06_f64 + (*c as f64 * 7.0/16.0),
            Channel::G2(_) => 1246.06_f64,
            _ => 0.0, //TODO
        }
    }
    
    /// Returns channel bandwidth in MHz
    pub fn bandwidth_mhz (&self) -> f64 {
        match self {
            Channel::L1 | Channel::G1(_) | Channel::E1 => 15.345_f64,
            Channel::L2 | Channel::G2(_) | Channel::E2 => 11.0_f64,
            Channel::L5 | Channel::E5 => 12.5_f64,
            Channel::E6 => 0.0, //TODO
            Channel::LEX => 0.0, //TODO
        }
    }

    /// Identifies Frequency channel, from given observable, related
    /// to given Constellation
    pub fn from_observable (constellation: Constellation, observable: &str) -> Result<Self, Error> {
        match constellation {
            Constellation::GPS => {
                if observable.contains("1") {
                    Ok(Self::L1)
                } else if observable.contains("2") {
                    Ok(Self::L2)
                } else if observable.contains("5") {
                    Ok(Self::L5)
                } else {
                    Err(Error::InvalidObservable(observable.to_string()))
                }
            },
            Constellation::Glonass => {
                if observable.contains("1") {
                    Ok(Self::G1(None))
                } else if observable.contains("2") {
                    Ok(Self::G2(None))
                } else {
                    Err(Error::InvalidObservable(observable.to_string()))
                }
            },
            Constellation::Galileo => {
                if observable.contains("1") {
                    Ok(Self::E1)
                } else if observable.contains("2") {
                    Ok(Self::E2)
                } else if observable.contains("5") {
                    Ok(Self::E5)
                } else if observable.contains("6") {
                    Ok(Self::E6)
                } else {
                    Err(Error::InvalidObservable(observable.to_string()))
                }
            },
            Constellation::SBAS(_) => {
                if observable.contains("1") {
                    Ok(Self::L1)
                } else if observable.contains("5") {
                    Ok(Self::L5)
                } else {
                    Err(Error::InvalidObservable(observable.to_string()))
                }
            },
            Constellation::QZSS => {
                if observable.contains("1") {
                    Ok(Self::L1)
                } else if observable.contains("2") {
                    Ok(Self::L2)
                } else if observable.contains("5") {
                    Ok(Self::L5)
                } else if observable.contains("7") {
                    Ok(Self::LEX) // TODO confirm !
                } else {
                    Err(Error::InvalidObservable(observable.to_string()))
                }
            },
            Constellation::IRNSS => {
                if observable.contains("1") {
                    Ok(Self::L1)
                } else if observable.contains("5") {
                    Ok(Self::L5)
                } else {
                    Err(Error::InvalidObservable(observable.to_string()))
                }
            },
            _ => todo!("not implemented for constellation \"{}\" yet..", constellation.to_3_letter_code()),
        }
    }
    
    /// Builds a Channel Frequency from an `Sv` 3 letter code descriptor,
    /// mainly used in `ATX` RINEX for so called `frequency` field
    pub fn from_sv_code (code: &str) -> Result<Self, Error> {
        let sv = sv::Sv::from_str(code)?;
        match sv.constellation {
            Constellation::GPS => {
                match sv.prn {
                    1 => Ok(Self::L1),
                    2 => Ok(Self::L2),
                    5 => Ok(Self::L5),
                    _ => Ok(Self::L1),
                }
            },
            Constellation::Glonass => {
                match sv.prn {
                    1 => Ok(Self::G1(None)),
                    2 => Ok(Self::G2(None)),
                    _ => Ok(Self::G1(None)),
                }
            },
            Constellation::Galileo => { 
                match sv.prn {
                    1 => Ok(Self::E1),
                    2 => Ok(Self::E2),
                    5 => Ok(Self::E5),
                    _ => Ok(Self::E1),
                }
            },
            Constellation::SBAS(_) => {
                match sv.prn {
                    1 => Ok(Self::L1),
                    5 => Ok(Self::L5),
                    _ => Ok(Self::L1),
                }
            },
            Constellation::BeiDou => {
                match sv.prn {
                    1 => Ok(Self::E1),
                    2 => Ok(Self::E2),
                    5 => Ok(Self::E5),
                    6 => Ok(Self::E6),
                    _ => Ok(Self::E1),
                }
            },
            Constellation::QZSS => {
                match sv.prn {
                    1 => Ok(Self::L1),
                    2 => Ok(Self::L2),
                    5 => Ok(Self::L5),
                    6 => Ok(Self::LEX),
                    _ => Ok(Self::L1),
                }
            },
            Constellation::IRNSS => {
                match sv.prn { // TODO: confirm!
                    1 => Ok(Self::L1),
                    5 => Ok(Self::L5),
                    _ => Ok(Self::L1),
                }
            },
            _ => panic!("non supported conversion from {}", sv.constellation.to_3_letter_code())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    /*#[test]
    fn test_code() {
        assert_eq!(Code::from_str("C1").is_ok(), true);
        assert_eq!(Code::from_str("L1").is_err(), true);
        assert_eq!(Code::from_str("P1").is_ok(),  true);
    }*/
    #[test]
    fn test_channel() {
        assert_eq!(Channel::from_str("L1").is_ok(), true);
        assert_eq!(Channel::from_str("C1").is_err(), true);
        assert_eq!(Channel::from_str("L5").is_ok(), true);
    }
}
