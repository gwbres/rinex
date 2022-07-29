//! `ObservationData` parser and related methods
use std::io::Write;
use thiserror::Error;
use std::str::FromStr;
use chrono::Timelike;
use bitflags::bitflags;
use std::collections::{BTreeMap, HashMap};

use crate::sv;
use crate::epoch;
use crate::header;
use crate::version;
use crate::constellation;
use crate::constellation::Constellation;
use crate::constellation::augmentation::Augmentation;

#[cfg(feature = "with-serde")]
use serde::Serialize;

/// `Ssi` describes signals strength
#[repr(u8)]
#[derive(PartialOrd, Ord, PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub enum Ssi {
    /// Ssi ~= 0 dB/Hz
    DbHz0 = 0,
    /// Ssi < 12 dB/Hz
    DbHz12 = 1,
    /// 12 dB/Hz <= Ssi < 17 dB/Hz
    DbHz12_17 = 2, 
    /// 18 dB/Hz <= Ssi < 23 dB/Hz
    DbHz18_23 = 3, 
    /// 24 dB/Hz <= Ssi < 29 dB/Hz
    DbHz21_29 = 4, 
    /// 30 dB/Hz <= Ssi < 35 dB/Hz
    DbHz30_35 = 5, 
    /// 36 dB/Hz <= Ssi < 41 dB/Hz
    DbHz36_41 = 6, 
    /// 42 dB/Hz <= Ssi < 47 dB/Hz
    DbHz42_47 = 7, 
    /// 48 dB/Hz <= Ssi < 53 dB/Hz
    DbHz48_53 = 8, 
    /// Ssi >= 54 dB/Hz 
    DbHz54 = 9, 
}

impl Default for Ssi {
    fn default() -> Ssi { Ssi::DbHz54 }
}

impl std::str::FromStr for Ssi {
    type Err = std::io::Error;
    fn from_str (code: &str) -> Result<Self, Self::Err> {
        match code {
            "0" => Ok(Ssi::DbHz0),
            "1" => Ok(Ssi::DbHz12),
            "2" => Ok(Ssi::DbHz12_17),
            "3" => Ok(Ssi::DbHz18_23),
            "4" => Ok(Ssi::DbHz21_29),
            "5" => Ok(Ssi::DbHz30_35),
            "6" => Ok(Ssi::DbHz36_41),
            "7" => Ok(Ssi::DbHz42_47),
            "8" => Ok(Ssi::DbHz48_53),
            "9" => Ok(Ssi::DbHz54),
            _ =>  Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid Ssi code")),
        }
    }
}

impl Ssi {
    /// Returns true if `self` is a bad signal level, very poor quality,
    /// measurements should be discarded
    pub fn is_bad (self) -> bool {
        self <= Ssi::DbHz18_23
    }
    /// Returns true if `self` is a weak signal level, poor quality
    pub fn is_weak (self) -> bool {
        self < Ssi::DbHz30_35
    }
    /// Returns true if `self` is a strong signal level, good quality as defined by standard
    pub fn is_strong (self) -> bool {
        self >= Ssi::DbHz30_35
    }
    /// Returns true if `self` is a very strong signal level, very high quality
    pub fn is_excellent (self) -> bool {
        self > Ssi::DbHz42_47
    }
    /// Returns true if `self` matches a strong signal level (defined by standard)
    pub fn is_ok (self) -> bool { self.is_strong() }
}

bitflags! {
    #[cfg_attr(feature = "with-serde", derive(Serialize))]
    pub struct LliFlags: u8 {
        /// Current epoch is marked Ok or Unknown status 
        const OK_OR_UNKNOWN = 0x00;
        /// Lock lost between previous observation and current observation,
        /// cycle slip is possible
        const LOCK_LOSS = 0x01;
        /// Opposite wavelenght factor to the one defined
        /// for the satellite by a previous WAVELENGTH FACT comment,
        /// or opposite to default value, is not previous WAVELENFTH FACT comment
        const HALF_CYCLE_SLIP = 0x02;
        /// Observing under anti spoofing,
        /// might suffer from decreased SNR - decreased signal quality
        const UNDER_ANTI_SPOOFING = 0x04;
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct ObservationData {
	/// physical measurement
	pub obs: f64,
	/// Lock loss indicator 
	pub lli: Option<LliFlags>,
	/// Signal strength indicator
	pub ssi: Option<Ssi>,
}

impl ObservationData {
	/// Builds new ObservationData structure from given predicates
    pub fn new (obs: f64, lli: Option<LliFlags>, ssi: Option<Ssi>) -> ObservationData {
		ObservationData {
			obs,
			lli,
			ssi,
		}
	}
    /// Returns `true` if self is determined as `ok`.    
    /// self is declared `ok` if LLI and SSI flags are not provided,
    /// because they are considered as unknown/ok if missing by default.   
    /// If LLI exists:    
    ///    + LLI must match the LliFlags::OkOrUnknown flag (strictly)    
    /// if SSI exists:    
    ///    + SSI must match the .is_ok() criteria, refer to API 
    pub fn is_ok (self) -> bool {
        let lli_ok = self.lli.unwrap_or(LliFlags::OK_OR_UNKNOWN) == LliFlags::OK_OR_UNKNOWN;
        let ssi_ok = self.ssi.unwrap_or(Ssi::default()).is_ok();
        lli_ok && ssi_ok
    }

    /// Returns Real Distance, by converting observed pseudo range,
    /// and compensating for distant and local clock offsets.
    /// See [p17-p18 of the RINEX specifications]. It makes only
    /// sense to apply this method on Pseudo Range observations.
    /// - rcvr_offset: receiver clock offset for this epoch, given in file
    /// - sv_offset: sv clock offset
    /// - bias: other (optionnal..) additive biases
    pub fn pr_real_distance (&self, rcvr_offset: f64, sv_offset: f64, biases: f64) -> f64 {
        self.obs + 299_792_458.0_f64 * (rcvr_offset - sv_offset) + biases
    }
}

/// `Record` content for OBS data files.   
/// Measurements are sorted by `epoch` (timestamps + flags).    
/// Measurements are of two kinds:
///  + Option<f64>: receiver clock offsets for OBS data files where   
///    receiver clock offsets are 'applied'    
///  + map of ObservationData (physical measurements) sorted by `Sv` and by observation codes 
pub type Record = BTreeMap<epoch::Epoch, 
    (Option<f64>, 
    BTreeMap<sv::Sv, HashMap<String, ObservationData>>)>;

#[derive(Error, Debug)]
/// OBS Data `Record` parsing specific errors
pub enum Error {
    #[error("failed to parse date")]
    ParseDateError(#[from] epoch::ParseDateError),
    #[error("failed to parse epoch flag")]
    ParseEpochFlagError(#[from] std::io::Error),
    #[error("failed to parse sv")]
    SvError(#[from] sv::Error),
    #[error("failed to integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse vehicules properly (n_sat mismatch)")]
    EpochParsingError,
}

/// Returns true if given content matches a new OBSERVATION data epoch
pub fn is_new_epoch (line: &str, v: version::Version) -> bool {
    let parsed: Vec<&str> = line
        .split_ascii_whitespace()
        .collect();
	if v.major < 3 {
        // old RINEX
        // epoch block is type dependent
        if parsed.len() > 6 {
            //  * contains at least 6 items
            let mut datestr = parsed[0].to_owned(); // Y
            datestr.push_str(" ");
            datestr.push_str(parsed[1]); // m
            datestr.push_str(" ");
            datestr.push_str(parsed[2]); // d
            datestr.push_str(" ");
            datestr.push_str(parsed[3]); // h
            datestr.push_str(" ");
            datestr.push_str(parsed[4]); // m
            datestr.push_str(" ");
            datestr.push_str(parsed[5]); // s
            epoch::str2date(&datestr).is_ok()
        } else {
            false // does not match
                // an epoch descriptor
        }
    } else {
        // Modern RINEX
        // OBS::V3 behaves like all::V4
        match line.chars().nth(0) {
            Some(c) => {
                c == '>' // epochs always delimited
                    // by this new identifier
            },
            _ => false,
        }
    }
}

/// Builds `Record` entry for `ObservationData`
/// from given epoch content
pub fn build_record_entry (header: &header::Header, content: &str)
        -> Result<(epoch::Epoch, Option<f64>, BTreeMap<sv::Sv, HashMap<String, ObservationData>>), Error> 
{
    let mut lines = content.lines();
    let mut line = lines.next()
        .unwrap();

    // epoch::
    let mut offset : usize = 
        2+1 // Y
        +2+1 // d
        +2+1 // m
        +2+1 // h
        +2+1 // m
        +11; // secs
    
    // V > 2 epoch::year is a 4 digit number
    if header.version.major > 2 {
        offset += 2
    }

    // V > 2 might start with a ">" marker
    if line.starts_with(">") {
        line = line.split_at(1).1.clone();
    }

    let (date, rem) = line.split_at(offset);
    let (flag, rem) = rem.split_at(3);
    let (n_sat, mut rem) = rem.split_at(3);
    let n_sat = u16::from_str_radix(n_sat.trim(), 10)?;
    let n_sv_line : usize = num_integer::div_ceil(n_sat, 12).into();

    let flag = epoch::EpochFlag::from_str(flag.trim())?;
    let date = epoch::str2date(date)?; 
    let epoch = epoch::Epoch::new(date, flag);

    let mut sv_list : Vec<sv::Sv> = Vec::with_capacity(24);
	let mut map : BTreeMap<sv::Sv, HashMap<String, ObservationData>> = BTreeMap::new();
	
    // all encountered obs codes
    let obs = header.obs
        .as_ref()
        .unwrap();
    let obs_codes = &obs.codes;
    
    // grabbing possible clock_offsets content
    let offs : Option<&str> = match header.version.major < 2 {
        true => {
            // old fashion RINEX:
            // clock offsets are last 12 characters
            if line.len() > 60-12 {
                Some(line.split_at(60-12).1.trim())
            } else {
                None
            }
        },
        false => {
            // modern RINEX:
            let min_len : usize = 
                 4+1 // y
                +2+1 // m
                +2+1 // d
                +2+1 // h
                +2+1 // m
                +11+1// s
                +3   // flag
                +3;   // n_sat
            if line.len() > min_len {
                Some(line.split_at(min_len).1.trim()) // increased precision
            } else {
                None
            }
        },
    };
    let clock_offset : Option<f64> = match offs.is_some() {
        true => {
            if let Ok(f) = f64::from_str(offs.unwrap()) {
                Some(f)
            } else {
                None // parsing failed for some reason
            }
        },
        false => None, // empty field
    };

    if header.version.major < 3 {
        // old fashion:
        //   Sv list is passed on 1st and possible several lines
        let mut offset : usize = 0;
        for _ in 0..n_sv_line {
            loop {
                let sv_str = &rem[offset..offset+3];
                let identifier = sv_str.chars().nth(0)
                    .unwrap(); 
                let prn = u8::from_str(&sv_str[1..].trim())?;
                // build `sv` 
                let sv : sv::Sv = match identifier.is_ascii_whitespace() {
                    true => sv::Sv::new(header.constellation.unwrap(), prn),
                    false => {
                        let constell : Constellation = match identifier {
                            'G' => Constellation::GPS,
                            'R' => Constellation::Glonass,
                            'J' => Constellation::QZSS,
                            'E' => Constellation::Galileo,
                            'C' => Constellation::BeiDou,
                            'S' => Constellation::SBAS(Augmentation::default()),
                            _ => return Err(
                                Error::SvError(
                                    sv::Error::ConstellationError(
                                        constellation::Error::UnknownCode(identifier.to_string())))),
                        };
                        sv::Sv::new(constell, prn)
                    },
                };
                
                sv_list.push(sv);
                offset += 3;
                if offset == rem.len() {
                    line = lines.next()
                        .unwrap();
                    rem = line.trim();
                    offset = 0;
                    break
                }
            } // sv systems content 
        } // sv system ID
    
        // verify identified list sanity
        if sv_list.len() != n_sat.into() {
            return Err(Error::EpochParsingError) // mismatch
        }

		for i in 0..sv_list.len() { // per vehicule
			let mut offset : usize = 0;
			let mut obs_map : HashMap<String, ObservationData> = HashMap::new();

			// old RINEX revision : using previously identified Sv 
			let sv : sv::Sv = sv_list[i]; 
			let codes =  obs_codes
                .get(&sv.constellation)
                .unwrap();
			let mut code_index : usize = 0;
			loop { // per obs code
				let code = &codes[code_index];
				let obs : Option<f64> = match line.len() < offset+14 { 
					true => {
						// cant' grab a new measurement
						//  * line is empty: contains only empty measurements
						//  * end of line is reached
						None
					},
					false => {
						let obs = &line[offset..offset+14];
						if let Ok(f) = f64::from_str(&obs.trim()) {
							Some(f)
						} else {
							None // empty field
						}
					},
				};

				let lli : Option<LliFlags> = match line.len() < offset+14+1 {
					true => {
						// can't parse lli here
						// 	* line is over and this measurement
						//    does not have lli nor ssi 
						None
					},
					false => {
						let lli = &line[offset+14..offset+14+1];
						if let Ok(lli) = u8::from_str_radix(&lli, 10) {
                            LliFlags::from_bits(lli)
                        } else {
                            None
                        }
					},
				};

				let ssi : Option<Ssi> = match line.len() < offset+14+2 {
					true => {
						// can't parse ssi here
						// 	* line is over and this measurement
						//    does not have ssi 
						None
					},
					false => {
						let ssi = &line[offset+14+1..offset+14+2];
						let ssi = match Ssi::from_str(ssi) {
							Ok(ssi) => Some(ssi),
							Err(_) => None, // ssi field is empty
						};
						ssi
					},
				};
				
				if let Some(obs) = obs { // parsed something
					let obs = ObservationData::new(obs, lli, ssi);
					obs_map.insert(code.to_string(), obs); 
				}
				
				code_index += 1;
				if code_index == obs_codes.len() {
					break // last code that system sv
				}
				
				offset += 14 // F14.3
					+1  // +lli
					+1; // +ssi

				if offset >= line.len() {
					// we just parsed the last
					// code for this line
					offset = 0;
					if let Some(l) = lines.next() {
						line = l;
					}
				}
			} // for all obs code
            map.insert(sv, obs_map);
			if let Some(l) = lines.next() {
				line = l;
			} else {
				break
			}
		} // for all systems
    } // V < 3 old fashion
	else { // V > 2 modern RINEX
		for _ in 0..n_sat {
			if let Some(l) = lines.next() {
				line = l;
			} else {
				break
			}
			
			// parse Sv and identify
			let (sv, rem) = line.split_at(3);
			let identifier = sv.chars().nth(0)
				.unwrap();
			let prn = u8::from_str_radix(&sv[1..].trim(),10)?;
			let constell : Constellation = match identifier {
				'G' => Constellation::GPS,
				'R' => Constellation::Glonass,
				'J' => Constellation::QZSS,
				'E' => Constellation::Galileo,
				'C' => Constellation::BeiDou,
				//'H' => Constellation::SBAS(Augmentation::default()),
                'S' => Constellation::SBAS(Augmentation::default()),
				_ => return Err(
                        Error::SvError(
                            sv::Error::ConstellationError(
                                constellation::Error::UnknownCode(identifier.to_string())))),
			};
			let sv = sv::Sv::new(constell, prn);
			// retrieve obs code for that system
			let codes =  &obs_codes[&constell];
			let mut offset : usize = 0;
			let mut code_index : usize = 0;
			let mut obs_map : HashMap<String, ObservationData> = HashMap::new();
			loop { // per obs code
				let code = &codes[code_index];
				let obs = &rem[offset..offset+14];
				let obs : Option<f64> = match f64::from_str(&obs.trim()) {
					Ok(f) => Some(f),
					Err(_) => None, // empty field
				};
				let lli : Option<LliFlags> = match rem.len() < offset+14+1 {
					true => {
						// can't parse lli here,
						// line is terminated by an OBS without lli nor ssi
						None
					},
					false => {
						let lli = &rem[offset+14..offset+14+1];
						if let Ok(lli) = u8::from_str_radix(&lli, 10) {
                            LliFlags::from_bits(lli)
                        } else {
                            None
                        }
					},
				};
				let ssi : Option<Ssi> = match rem.len() < offset+14+2 {
					true => {
						// can't parse ssi here,
						// line is terminated by an OBS without ssi
						None
					},
					false => {
						let ssi = &rem[offset+14+1..offset+14+2];
						let ssi = match Ssi::from_str(ssi) {
							Ok(ssi) => Some(ssi),
							Err(_) => None, // ssi field is empty
						};
						ssi
					},
				};

				if let Some(obs) = obs { // parsed something
					let obs = ObservationData::new(obs, lli, ssi);
					obs_map.insert(code.to_string(), obs);
					code_index += 1;
				}
				
				offset += 14 // F14.3
					+1  // +lli
					+1; // +ssi
				
				if offset >= rem.len() { // done parsing this line
					map.insert(sv, obs_map);
					break
				}
			} // per obs code
		} // per sat
	} // V>2
    Ok((epoch, clock_offset, map))
}

/// Pushes observation record into given file writer
pub fn to_file (header: &header::Header, record: &Record, mut writer: std::fs::File) -> std::io::Result<()> {
    for (epoch, (clock_offset, sv)) in record.iter() {
        let date = epoch.date;
        let flag = epoch.flag;
        let vehicules = sv.keys();
        let nb_sv = vehicules.len(); 
        let obscodes = &header.obs.as_ref().unwrap().codes;
        // first line(s)
        //   Epoch + flag + svnn + possible clock offset
        match header.version.major {
            1|2 => {
                write!(writer, " {} ",  date.format("%y %m %d %H %M").to_string())?;
                write!(writer, " {}         ", date.time().second())?;
                write!(writer, " {}", flag)?; 
                write!(writer, " {}", nb_sv)?; 
                let nb_extra = nb_sv / 12;
                let mut index = 0;
                for vehicule in vehicules.into_iter() {
                    write!(writer, "{}", vehicule)?; 
                    if (index+1) % 12 == 0 {
                        if let Some(clock_offset) = clock_offset {
                            write!(writer, "{:3.9}", clock_offset)?
                        }
                        write!(writer, "\n                                ")?
                    }
                    index += 1
                }
                if nb_extra == 0 {
                    if let Some(clock_offset) = clock_offset {
                        let _ = write!(writer, "{:3.9}\n", clock_offset);
                    } else {
                        let _ = write!(writer, "\n");
                    }
                }
            },
            _ => { // Modern revisions 
                write!(writer, "> {} ",  date.format("%Y %m %d %H %M").to_string())?;
                write!(writer, " {}         ", date.time().second())?;
                write!(writer, " {} ", flag)?; 
                write!(writer, " {}", nb_sv)?; 
                if let Some(clock_offset) = clock_offset {
                    write!(writer, "{:.12}", clock_offset)?
                }
                write!(writer, "\n")?
            }
        }
        // epoch body
        let mut index = 0;
        for (sv, obs) in sv.iter() {
            let mut modulo = 5;
            if header.version.major > 2 {
                // modern RINEX
                modulo = 100000; // 'infinite': no wrapping
                    // we behave like CRX2RNX which does not respect the standards,
                    // we should wrap @ 80 once again
                let _ = write!(writer, "{} ", sv);
            } else {
                let _ = write!(writer, " ");
            }
            // observables for this constellation
            // --> respect header order and data might be missing
            let codes = &obscodes[&sv.constellation];
            for code in codes.iter() {
                if let Some(data) = obs.get(code) {
                    let _ = write!(writer, "{:13.3}", data.obs);
                    if let Some(lli) = data.lli {
                        let _ = write!(writer, "{}", lli.bits());
                    } else {
                        let _ = write!(writer, " ");
                    }
                    if let Some(ssi) = data.ssi {
                        let _ = write!(writer, "{}", ssi as u8);
                    } else {
                        let _ = write!(writer, " ");
                    }
                    if (index+1) % modulo == 0 {
                        let _ = write!(writer, "\n");
                    }
                    let _ = write!(writer, " ");
                } else {
                    // obs is missing, simply fill with whitespace
                    let _ = write!(writer, "                ");
                }
                index += 1
            }
            write!(writer, "\n")?
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn ssi() {
        let ssi = Ssi::from_str("0").unwrap(); 
        assert_eq!(ssi, Ssi::DbHz0);
        assert_eq!(ssi.is_bad(), true);
        let ssi = Ssi::from_str("9").unwrap(); 
        assert_eq!(ssi.is_excellent(), true);
        let ssi = Ssi::from_str("10"); 
        assert_eq!(ssi.is_err(), true);
    }
    #[test]
    fn new_epoch() {
        assert_eq!(        
            is_new_epoch("95 01 01 00 00 00.0000000  0  7 06 17 21 22 23 28 31",
                version::Version {
                    major: 2,
                    minor: 0,
                }
            ),
            true
        );
        assert_eq!(        
            is_new_epoch("21700656.31447  16909599.97044          .00041  24479973.67844  24479975.23247",
                version::Version {
                    major: 2,
                    minor: 0,
                }
            ),
            false
        );
        assert_eq!(        
            is_new_epoch("95 01 01 11 00 00.0000000  0  8 04 16 18 19 22 24 27 29",
                version::Version {
                    major: 2,
                    minor: 0,
                }
            ),
            true
        );
        assert_eq!(        
            is_new_epoch("95 01 01 11 00 00.0000000  0  8 04 16 18 19 22 24 27 29",
                version::Version {
                    major: 3,
                    minor: 0,
                }
            ),
            false 
        );
        assert_eq!(        
            is_new_epoch("> 2022 01 09 00 00 30.0000000  0 40",
                version::Version {
                    major: 3,
                    minor: 0,
                }
            ),
            true 
        );
        assert_eq!(        
            is_new_epoch("> 2022 01 09 00 00 30.0000000  0 40",
                version::Version {
                    major: 2,
                    minor: 0,
                }
            ),
            false
        );
        assert_eq!(        
            is_new_epoch("G01  22331467.880   117352685.28208        48.950    22331469.28",
                version::Version {
                    major: 3,
                    minor: 0,
                }
            ),
            false
        );
    }
}
