//! Describes a `RINEX` header, includes
//! rinex header parser and associated methods
use crate::leap;
use crate::antex;
use crate::clocks;
use crate::version;
//use crate::gnss_time;
use crate::hardware;
use crate::reader::BufferedReader;
use crate::types::{Type, TypeError};
use crate::merge::MergeError;
use crate::meteo;
use crate::observation;
use crate::ionosphere;
use crate::constellation;
use crate::constellation::{Constellation, augmentation::Augmentation};

use thiserror::Error;
use std::str::FromStr;
use strum_macros::EnumString;
use std::collections::HashMap;
use std::io::{prelude::*};

#[cfg(feature = "with-serde")]
use serde::{Serialize, Deserialize};

#[cfg(feature = "with-serde")]
use crate::formatter::point3d;

#[derive(Clone, Debug)]
#[derive(EnumString)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum MarkerType {
    /// Earth fixed & high precision
    #[strum(serialize = "GEODETIC", serialize = "Geodetic")]
    Geodetic,
    /// Earth fixed & low precision
    #[strum(serialize = "NON GEODETIC", serialize = "NonGeodetic")]
    NonGeodetic,
    /// Generated from network
    #[strum(serialize = "NON PHYSICAL", serialize = "NonPhysical")]
    NonPhysical,
    /// Orbiting space vehicule
    #[strum(serialize = "SPACE BORNE", serialize = "Spaceborne")]
    Spaceborne,
    /// Aircraft, balloon..
    #[strum(serialize = "AIR BORNE", serialize = "Airborne")]
    Airborne,
    /// Mobile water craft
    #[strum(serialize = "WATER CRAFT", serialize = "Watercraft")]
    Watercraft,
    /// Mobile terrestrial vehicule
    #[strum(serialize = "GROUND CRAFT", serialize = "Groundcraft")]
    Groundcraft,
    /// Fixed on water surface
    #[strum(serialize = "FXED BUOY", serialize = "FixedBuoy")]
    FixedBuoy,
    /// Floating on water surface
    #[strum(serialize = "FLOATING BUOY", serialize = "FloatingBuoy")]
    FloatingBuoy,
    /// Floating on ice
    #[strum(serialize = "FLOATING ICE", serialize = "FloatingIce")]
    FloatingIce, 
    /// Fixed on glacier
    #[strum(serialize = "GLACIER", serialize = "Glacier")]
    Glacier,
    /// Rockets, shells, etc..
    #[strum(serialize = "BALLISTIC", serialize = "Ballistic")]
    Ballistic,
    /// Animal carrying a receiver
    #[strum(serialize = "ANIMAL", serialize = "Animal")]
    Animal,
    /// Human being carrying a receiver
    #[strum(serialize = "HUMAN", serialize = "Human")]
    Human,
}

impl Default for MarkerType {
    fn default() -> Self { 
        Self::Geodetic 
    }
}

/// Describes `RINEX` file header
#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct Header {
    /// revision for this `RINEX`
    pub version: version::Version, 
    /// type of `RINEX` file
    pub rinex_type: Type, 
    /// specific `GNSS` constellation system,
	/// may not exist for RINEX files 
    pub constellation: Option<Constellation>, 
    /// comments extracted from `header` section
    pub comments : Vec<String>,
    /// program name
    pub program: String, 
    /// program `run by`
    pub run_by: String,
    /// program's `date`
    pub date: String, 
    /// station label
    pub station: String, 
    /// station identifier
    pub station_id: String, 
    /// optionnal station URL 
    pub station_url: String, 
    /// name of observer
    pub observer: String, 
    /// name of production agency
    pub agency: String, 
    /// optionnal receiver placement infos
    pub marker_type: Option<MarkerType>,
    /// optionnal leap seconds infos
    pub leap: Option<leap::Leap>, 
    /// station approxiamte coordinates
    #[cfg_attr(feature = "with-serde", serde(with = "point3d"))]
    pub coords: Option<rust_3d::Point3D>, 
    /// optionnal observation wavelengths
    pub wavelengths: Option<(u32,u32)>, 
    /// optionnal sampling interval (s)
    pub sampling_interval: Option<f32>, 
    /// optionnal file license
    pub license: String,
    /// optionnal Object Identifier (IoT)
    pub doi: String,
    /// optionnal GPS/UTC time difference
    pub gps_utc_delta: Option<u32>,
    /// processing:   
    /// optionnal data scaling
    pub data_scaling: Option<f64>,
    // optionnal ionospheric compensation param(s)
    //ionospheric_corr: Option<Vec<IonoCorr>>,
    // possible time system correction(s)
    //gnsstime_corr: Option<Vec<gnss_time::GnssTimeCorr>>,
    ////////////////////////////////////////
    // Hardware
    ////////////////////////////////////////
    /// optionnal receiver infos
    pub rcvr: Option<hardware::Rcvr>, 
    /// optionnal antenna infos
    pub ant: Option<hardware::Antenna>, 
    //////////////////////////////////
    // OBSERVATION
    //////////////////////////////////
    /// Observation record specific fields
    pub obs: Option<observation::HeaderFields>,
    //////////////////////////////////
    // Meteo 
    //////////////////////////////////
    /// Meteo record specific fields
    pub meteo: Option<meteo::HeaderFields>,
    //////////////////////////////////
    // Clocks fields 
    //////////////////////////////////
    /// Clocks record specific fields
    pub clocks: Option<clocks::HeaderFields>,
    //////////////////////////////////
    // Antex
    //////////////////////////////////
    /// ANTEX record specific fields
    pub antex: Option<antex::HeaderFields>,
    /////////////////////////////////
    // IONEX
    /////////////////////////////////
    /// IONEX record specific fields
    pub ionex: Option<ionosphere::HeaderFields>,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("CRINEX related content mismatch")]
    CrinexFormatError,
    #[error("RINEX version is not supported '{0}'")]
    VersionNotSupported(String),
    #[error("Line \"{0}\" should begin with Rinex version \"x.yy\"")]
    VersionFormatError(String),
    #[error("rinex type error")]
    TypeError(#[from] TypeError),
    #[error("constellation error")]
    ConstellationError(#[from] constellation::Error),
    #[error("failed to parse leap from \"{0}\"")]
    LeapParsingError(#[from] leap::Error),
    #[error("failed to parse antenna / receiver infos")]
    AntennaRcvrError(#[from] std::io::Error),
    #[error("failed to parse integer value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse date")]
    DateParsingError(#[from] chrono::ParseError),
    #[error("failed to parse ANTEX fields")]
    AntexParsingError(#[from] antex::record::Error),
    #[error("failed to parse PCV field")]
    ParsePcvError(#[from] antex::pcv::Error),
    #[error("faulty ionex format")]
    FaultyIonexDescription,
}

impl Default for Header {
    fn default() -> Header {
        Header {
            version: version::Version::default(), 
            rinex_type: Type::default(),
            constellation: Some(Constellation::default()),
            comments: Vec::new(),
            program: String::new(),
            run_by: String::new(),
            date: String::new(),
            station: String::new(),
            station_id: String::new(),
            observer: String::new(),
            agency: String::new(),
            marker_type: None,
            station_url: String::new(),
            doi: String::new(),
            license: String::new(),
            leap: None,
            gps_utc_delta: None,
            // hardware
            rcvr: None,
            ant: None,
            coords: None, 
            wavelengths: None,
            // processing
            data_scaling: None,
            //ionospheric_corr: None,
            //gnsstime_corr: None,
            sampling_interval: None,
            /////////////////////////
            // OBSERVATION
            /////////////////////////
            obs: None,
            /////////////////////////
            // OBSERVATION / METEO
            /////////////////////////
			meteo: None,
            /////////////////////////
            // Clocks
            /////////////////////////
            clocks: None,
            /////////////////////////
            // Antex
            /////////////////////////
            antex: None,
            /////////////////////////
            // IONEX 
            /////////////////////////
            ionex: None,
        }
    }
}

impl Header {
    /// Builds a `Header` from local file and previously grabbed 1st line
    pub fn new (reader: &mut BufferedReader) -> Result<Header, Error> { 
        let mut crinex : Option<observation::Crinex> = None;
        let mut crnx_version = version::Version::default(); 
        let mut rinex_type = Type::default();
        let mut constellation : Option<Constellation> = None;
        let mut version = version::Version::default();
        let mut comments   : Vec<String> = Vec::new();
        let mut program    = String::new();
        let mut run_by     = String::new();
        let mut date       = String::new();
        let mut station    = String::new();
        let mut station_id = String::new();
        let mut observer   = String::new();
        let mut agency     = String::new();
        let mut license    = String::new();
        let mut doi        = String::new();
        let mut station_url= String::new();
        let mut marker_type : Option<MarkerType> = None; 
        // Hardware 
        let mut ant_model = String::new();
        let mut ant_sn = String::new();
        let mut ant_coords : Option<rust_3d::Point3D> = None;
        let mut ant_hen    : Option<(f32,f32,f32)> = None;
        let mut rcvr       : Option<hardware::Rcvr> = None;
        // other
        let mut leap       : Option<leap::Leap> = None;
        let mut sampling_interval: Option<f32> = None;
        let mut coords     : Option<rust_3d::Point3D> = None;
        // (OBS)
        let mut obs_clock_offset_applied = false;
        let mut obs_code_lines : u8 = 0; 
        let mut current_code_syst = Constellation::default(); // to keep track in multi line scenario + Mixed constell 
        let mut obs_codes  : HashMap<Constellation, Vec<String>> = HashMap::with_capacity(10);
        // (OBS/METEO)
		let mut met_codes  : Vec<meteo::observable::Observable> = Vec::new();
		let mut met_sensors: Vec<meteo::sensor::Sensor> = Vec::with_capacity(3);
        // CLOCKS
        let mut clk_ref = String::new();
        let mut clk_codes: Vec<clocks::record::DataType> = Vec::new();
        let mut clk_agency_code = String::new();
        let mut clk_agency_name = String::new();
        let mut clk_station_name = String::new();
        let mut clk_station_id = String::new();
        // ANTEX
        let mut pcv : Option<antex::pcv::Pcv> = None;
        let mut ant_relative_values = String::from("AOAD/M_T");
        let mut ref_ant_sn : Option<String> = None;
        // IONEX
        let mut ionex = ionosphere::HeaderFields::default();
        // iterate on a line basis
        let lines = reader.lines();
        for l in lines { 
            let line = l.unwrap();
            if line.len() < 60 {
                continue // --> invalid header content
            }
            let (content, marker) = line.split_at(60);
            ///////////////////////////////
            // [0] END OF HEADER  
            //     --> done parsing
            ///////////////////////////////
            if marker.trim().eq("END OF HEADER") {
                break
            ///////////////////////////////
            // [0*] COMMENTS
            ///////////////////////////////
            } if marker.trim().eq("COMMENT") {
                // --> storing might be useful
                comments.push(content.trim().to_string());
                continue
            
            //////////////////////////////////////
            // [1] CRINEX Special fields
            /////////////////////////////////////
            } else if marker.contains("CRINEX VERS") {
                let version = content.split_at(20).0;
                crnx_version = version::Version::from_str(version.trim())?
            } else if marker.contains("CRINEX PROG / DATE") {
                let (pgm, remainder) = content.split_at(20);
                let (_, remainder) = remainder.split_at(20);
                let date = remainder.split_at(20).0.trim();
                crinex = Some(
                    observation::Crinex {
                        version: crnx_version, 
                        prog: pgm.trim().to_string(),
                        date: chrono::NaiveDateTime::parse_from_str(date, "%d-%b-%y %H:%M")?
                    })
            
            ////////////////////////////////////////
            // [2] ANTEX special header
            ////////////////////////////////////////
            } else if marker.contains("ANTEX VERSION / SYST") {
                let (vers, system) = content.split_at(8);
                version = version::Version::from_str(vers.trim())?;
                if let Ok(constell) = Constellation::from_str(system.trim()) {
                    constellation = Some(constell)
                }
                rinex_type = Type::AntennaData;
            } else if marker.contains("PCV TYPE / REFANT") {
                let (pcv_str, rem) = content.split_at(20);
                let (ref_type, rem) = rem.split_at(20);
                let (ref_sn, _) = rem.split_at(20);
                if let Ok(p) = antex::pcv::Pcv::from_str(pcv_str.trim()) {
                    pcv = Some(p)
                }
                if ref_type.trim().len() > 0 {
                    ant_relative_values = ref_type.trim().to_string();
                }
                if ref_sn.trim().len() > 0 {
                    ref_ant_sn = Some(ref_sn.trim().to_string())
                }
            
            //////////////////////////////////////
            // [2] IONEX special header 
            //////////////////////////////////////
            } else if marker.contains("IONEX VERSION / TYPE") {
                let (vers, rem) = line.split_at(20);
                let (type_str, rem) = rem.split_at(20); 
                let (system_str, _) = rem.split_at(20);
                version = version::Version::from_str(vers.trim())?;
                rinex_type = Type::from_str(type_str.trim())?;
                if rinex_type != Type::IonosphereMaps {
                    return Err(Error::FaultyIonexDescription)
                }
                ionex = ionex.with_system(system_str.trim())

            ///////////////////////////////////////
            // ==> from now on
            // RINEX standard / shared attributes
            ///////////////////////////////////////
            } else if marker.contains("RINEX VERSION / TYPE") {
                let (vers, rem) = line.split_at(20);
                let (type_str, rem) = rem.split_at(20); 
                let (constell_str, _) = rem.split_at(20);
                rinex_type = Type::from_str(type_str.trim())?;
                if type_str.contains("GLONASS") {
                    // special case, sometimes GLONASS NAV
                    // drops the constellation field cause it's implied
                    constellation = Some(Constellation::Glonass)
                } else if type_str.contains("METEOROLOGICAL DATA") {
                    // these files are not tied to a constellation system,
                    // therefore, do not have this field
                    constellation = None
                } else { // regular files
                    if let Ok(constell) = Constellation::from_str(constell_str.trim()) {
                        
                        constellation = Some(constell)
                    }
                }
                version = version::Version::from_str(vers.trim())?;
                if !version.is_supported() {
                    return Err(Error::VersionNotSupported(vers.to_string()))
                }
            
            } else if marker.contains("PGM / RUN BY / DATE") {
                let (pgm, rem) = line.split_at(20);
                program = pgm.trim().to_string();
                let (rb, rem) = rem.split_at(20);
                run_by = match rb.trim().eq("") {
                    true => String::from("Unknown"),
                    false => rb.trim().to_string(), 
                };
                let (date_str, _) = rem.split_at(20);
                date = date_str.trim().to_string()
            
            } else if marker.contains("MARKER NAME") {
                station = content.split_at(20).0.trim().to_string()
            
            } else if marker.contains("MARKER NUMBER") {
                station_id = content.split_at(20).0.trim().to_string()
            
            } else if marker.contains("MARKER TYPE") {
                let code = content.split_at(20).0.trim();
                if let Ok(marker) = MarkerType::from_str(code) {
                    marker_type = Some(marker)
                }
            
            } else if marker.contains("OBSERVER / AGENCY") {
                let (obs, ag) = content.split_at(20);
                observer = obs.trim().to_string();
                agency = ag.trim().to_string()

            } else if marker.contains("REC # / TYPE / VERS") {
                if let Ok(receiver) = hardware::Rcvr::from_str(content) {
                    rcvr = Some(receiver)
                }

			} else if marker.contains("SENSOR MOD/TYPE/ACC") {
                if let Ok(sensor) = meteo::sensor::Sensor::from_str(content) {
                    met_sensors.push(sensor)
                }
            } else if marker.contains("SENSOR POS XYZ/H") {
                let (x_str, rem) = content.split_at(14);
                let (y_str, rem) = rem.split_at(14);
                let (z_str, rem) = rem.split_at(14);
                let (h_str, phys_str) = rem.split_at(14);
                if let Ok(observable) = meteo::observable::Observable::from_str(phys_str.trim()) {
                    for sensor in met_sensors.iter_mut() {
                        if sensor.observable == observable {
                            if let Ok(x) = f64::from_str(x_str.trim()) {
                                if let Ok(y) = f64::from_str(y_str.trim()) {
                                    if let Ok(z) = f64::from_str(z_str.trim()) {
                                        if let Ok(h) = f64::from_str(h_str.trim()) {
                                            *sensor = sensor.
                                                with_position((x,y,z,h))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            
            } else if marker.contains("ANT # / TYPE") {
                let (model, rem) = content.split_at(20);
                let (sn, _) = rem.split_at(20);
                ant_model = model.trim().to_string();
                ant_sn = sn.trim().to_string();
            
            } else if marker.contains("LEAP SECOND") {
                let leap_str = content.split_at(40).0.trim();
                if let Ok(lleap) = leap::Leap::from_str(leap_str) {
                    leap = Some(lleap)
                }

            } else if marker.contains("DOI") {
                let (content, _) = content.split_at(40); //  TODO: confirm please
                doi = content.trim().to_string()

            } else if marker.contains("MERGED FILE") {
                //TODO V > 3 
                // nb# of merged files

            } else if marker.contains("STATION INFORMATION") {
                let (url, _) = content.split_at(40); //TODO confirm please 
                station_url = url.trim().to_string()

            } else if marker.contains("LICENSE OF USE") {
                let (lic, _) = content.split_at(40); //TODO confirm please 
                license = lic.trim().to_string()
            
            } else if marker.contains("WAVELENGTH FACT L1/2") {
                //TODO

            } else if marker.contains("APPROX POSITION XYZ") {
                let items: Vec<&str> = content.split_ascii_whitespace()
                    .collect();
                if let Ok(x) = f64::from_str(items[0].trim()) {
                    if let Ok(y) = f64::from_str(items[1].trim()) {
                        if let Ok(z) = f64::from_str(items[2].trim()) {
                            coords = Some(rust_3d::Point3D::new(x,y,z))
                        }
                    }
                }

            } else if marker.contains("ANTENNA: DELTA H/E/N") {
                let (h, rem) = content.split_at(15);
                let (e, rem) = rem.split_at(15);
                let (n, _) = rem.split_at(15);
                if let Ok(h) = f32::from_str(h.trim()) {
                    if let Ok(e) = f32::from_str(e.trim()) {
                        if let Ok(n) = f32::from_str(n.trim()) {
                            ant_hen = Some((h, e, n))
                        }
                    }
                }

            } else if marker.contains("ANTENNA: DELTA X/Y/Z") {
                let items: Vec<&str> = content.split_ascii_whitespace()
                    .collect();
                if let Ok(x) = f64::from_str(items[0].trim()) {
                    if let Ok(y) = f64::from_str(items[1].trim()) {
                        if let Ok(z) = f64::from_str(items[2].trim()) {
                            ant_coords = Some(rust_3d::Point3D::new(x,y,z))
                        }
                    }
                }

            } else if marker.contains("ANTENNA: B.SIGHT XYZ") {
                //TODO
            } else if marker.contains("ANTENNA: ZERODIR XYZ") {
                //TODO
            } else if marker.contains("CENTER OF MASS: XYZ") {
                //TODO
            } else if marker.contains("ANTENNA: PHASECENTER") {
                //TODO
            
            } else if marker.contains("RCV CLOCK OFFS APPL") {
                let value = content.split_at(20).0.trim();
                if let Ok(n) = i32::from_str_radix(value, 10) {
                    obs_clock_offset_applied = n > 0
                }

            } else if marker.contains("# OF SATELLITES") {
                // ---> we don't need this info,
                //     user can determine it by analyzing the record

            } else if marker.contains("PRN / # OF OBS") {
                // ---> we don't need this info,
                //     user can determine it by analyzing the record
                 
            } else if marker.contains("SYS / PHASE SHIFT") {
                //TODO

            } else if marker.contains("SYS / PVCS APPLIED") {
                // RINEX::ClockData specific 
                // + satellite system (G/R/E/C/I/J/S)
                // + programe name to apply Phase Center Variation
                // + source of corrections (url)
                // <o repeated for each satellite system
                // <o blank field when no corrections applied
            
            } else if marker.contains("TYPES OF OBS") { 
                // --> parsing Observables (V<3 old fashion)
                // ⚠ ⚠ could either be observation or meteo data
                if obs_code_lines == 0 { // first line ever
                    let (n, rem) = content.split_at(6);
                    if let Ok(n) = u8::from_str_radix(n.trim(), 10) {
                        obs_code_lines = num_integer::div_ceil(n, 9); // max. items per line

                    } else {
                        continue // failed to identify # of observables
                            // --> we'll continue grabing some header infos
                            //     record builder will not produce much
                    }
                    
                    let codes : Vec<String> = rem
                        .split_ascii_whitespace()
                        .map(|r| r.trim().to_string())
                        .collect();
                    if rinex_type == Type::ObservationData {
                        match constellation {
                            Some(Constellation::Mixed) => {
                                // Old RINEX + Mixed Constellation:
                                // description is not accurate enough to determine which
                                // code will be measured for which constellation
                                // ---> copy them for all major constellations...
                                //      record builder will use the ones it needs
                                let constells : Vec<Constellation> = vec![
                                    Constellation::GPS,
                                    Constellation::Glonass,
                                    Constellation::Galileo,
                                    Constellation::Beidou,
                                    Constellation::SBAS(Augmentation::default()),
                                    Constellation::QZSS,
                                ];
                                for i in 0..constells.len() {
                                    obs_codes.insert(constells[i], codes.clone());
                                } 
                            },
                            Some(constellation) => {
                                obs_codes.insert(constellation, codes.clone());
                            },
                            None => unreachable!("OBS rinex with no constellation specified"),
                        }
                    } else if rinex_type == Type::MeteoData {
                        for c in codes {
                            if let Ok(o) = meteo::observable::Observable::from_str(&c) {
                                met_codes.push(o);
                            }
                        }
                    }
                    obs_code_lines -= 1
                } else {
                    // Observables, 2nd, 3rd.. lines 
                    let codes : Vec<String> = content 
                        .split_ascii_whitespace()
                        .map(|r| r.trim().to_string())
                        .collect(); 
                    if rinex_type == Type::ObservationData {
                        // retrieve correspond system and append codes with new values 
                        let to_retrieve : Vec<Constellation> = match constellation {
                            Some(Constellation::Mixed) => {
                                vec![ // Old OBS Data + Mixed constellation ==> no means to differentiate
                                    Constellation::GPS,
                                    Constellation::Glonass,
                                    Constellation::Galileo,
                                    Constellation::Beidou,
                                    Constellation::SBAS(Augmentation::default()),
                                    Constellation::QZSS,
                                ]
                            },
                            Some(c) => vec![c],
                            None => unreachable!("OBS rinex with no constellation specified"),
                        };
                        for r in to_retrieve {
                            // retrieve map being built
                            if let Some(mut prev) = obs_codes.remove(&r) {
                                // increment obs code map
                                for code in &codes {
                                    prev.push(code.to_string());
                                }
                                obs_codes.insert(r, prev); // (re)insert
                            } 
                        }
                    } else if rinex_type == Type::MeteoData {
                        // simple append, list is simpler
                        for c in codes {
                            if let Ok(o) = meteo::observable::Observable::from_str(&c) {
                                met_codes.push(o);
                            }
                        }
                    }
                    obs_code_lines -= 1
                }

            } else if marker.contains("SYS / # / OBS TYPES") {
                // --> observable (V>2 modern fashion)
                if obs_code_lines == 0 {
                    // First line describing observables 
                    let (identifier, rem) = content.split_at(1);
                    let (n, rem) = rem.split_at(5);
                    if let Ok(n) = u8::from_str_radix(n.trim(), 10) {
                        obs_code_lines = num_integer::div_ceil(n, 13); // max. items per line
                    } else {
                        continue // failed to identify # of observables,
                            // we'll continue parsing header section,
                            // record builder won't produce much
                    }
                    let codes : Vec<String> = rem
                        .split_ascii_whitespace()
                        .map(|r| r.trim().to_string())
                        .collect();
                    if let Ok(constell) = Constellation::from_1_letter_code(identifier) {
                        current_code_syst = constell.clone(); // to keep track,
                            // on 2nd and 3rd line, system will not be reminded
                        obs_codes.insert(constell, codes);
                    }
                } else {
                    // 2nd, 3rd.. line of observables 
                    let codes : Vec<String> = content
                        .split_ascii_whitespace()
                        .map(|r| r.trim().to_string())
                        .collect();
                    // increment list with new codes
                    if let Some(list) = obs_codes.get_mut(&current_code_syst) {
                        // increment obs code map
                        for code in codes {
                            list.push(code);
                        }
                        //obs_codes.insert(current_code_syst, prev); // (re)insert)
                    }
                } 
                obs_code_lines -= 1

            } else if marker.contains("ANALYSIS CENTER") {
                let (code, agency) = content.split_at(3);
                clk_agency_code = code.trim().to_string();
                clk_agency_name = agency.trim().to_string();

            } else if marker.contains("# / TYPES OF DATA") {
                let (n, r) = content.split_at(6);
                let n = u8::from_str_radix(n.trim(),10)?;
                let mut rem = r.clone();
                for _ in 0..n {
                    let (code, r) = rem.split_at(6);
                    if let Ok(c) = clocks::record::DataType::from_str(code.trim()) {
                        clk_codes.push(c)
                    }
                    rem = r.clone()
                }

            } else if marker.contains("STATION NAME / NUM") {
                let (name, num) = content.split_at(4);
                clk_station_name = name.trim().to_string();
                clk_station_id = num.trim().to_string();

            } else if marker.contains("STATION CLK REF") {
                clk_ref = content.trim().to_string()
         
            } else if marker.contains("SIGNAL STRENGHT UNIT") {
                //TODO
            } else if marker.contains("INTERVAL") {
                let intv = content.split_at(20).0.trim();
                sampling_interval = Some(f32::from_str(intv)?)

            } else if marker.contains("GLONASS SLOT / FRQ #") {
                //TODO
            } else if marker.contains("GLONASS COD/PHS/BIS") {
                //TODO

            } else if marker.contains("ION ALPHA") { 
                //TODO
                //0.7451D-08 -0.1490D-07 -0.5960D-07  0.1192D-06          ION ALPHA           

            } else if marker.contains("ION BETA") {
                //TODO
                //0.9011D+05 -0.6554D+05 -0.1311D+06  0.4588D+06          ION BETA            
            } else if marker.contains("IONOSPHERIC CORR") {
                // TODO
                // GPSA 0.1025E-07 0.7451E-08 -0.5960E-07 -0.5960E-07
                // GPSB 0.1025E-07 0.7451E-08 -0.5960E-07 -0.5960E-07

            } else if marker.contains("TIME SYSTEM CORR") {
                // TODO
                // GPUT 0.2793967723E-08 0.000000000E+00 147456 1395
            
            } else if marker.contains("DELTA-UTC") {
                //TODO
                //0.931322574615D-09 0.355271367880D-14   233472     1930 DELTA-UTC: A0,A1,T,W
            
            } else if marker.contains("DESCRIPTION") { // IONEX description
                ionex = ionex
                    .with_description(content.trim())
            } else if marker.contains("OBSERABLES USED") { // IONEX observables
                ionex = ionex
                    .with_observables(content.trim())
            } else if marker.contains("# OF STATIONS") { // IONEX
                if let Ok(u) = u32::from_str_radix(content.trim(), 10) {
                    ionex = ionex
                        .with_stations(u)
                }
            } else if marker.contains("# OF SATELLITES") { // IONEX
                if let Ok(u) = u32::from_str_radix(content.trim(), 10) {
                    ionex = ionex
                        .with_satellites(u)
                }
            }
        }

        Ok(Header{
            version: version,
            rinex_type,
            constellation,
            comments,
            program,
            run_by,
            date,
            station,
            station_id,
            agency,
            observer,
            license,
            doi,
            station_url,
            marker_type,
            rcvr, 
            leap,
            coords: coords,
            wavelengths: None,
            gps_utc_delta: None,
            sampling_interval: sampling_interval,
            data_scaling: None,
            //ionospheric_corr: None,
            //gnsstime_corr: None,
            ///////////////////////
            // Hardware
            ///////////////////////
            ant: {
                if ant_model.len() > 0 {
                    Some(hardware::Antenna {
                        model: ant_model.clone(),
                        sn: ant_sn.clone(),
                        coords: ant_coords.clone(),
                        height: {
                            if let Some((h,_,_)) = ant_hen {
                                Some(h)
                            } else {
                                None
                            }
                        },
                        eastern_ecc: None, 
                        northern_ecc: None, //TODO ant_northern_ecc.clone(),
                    })
                } else {
                    None
                }
            },
            ///////////////////////
            // OBSERVATION
            ///////////////////////
            obs: {
                if obs_codes.len() > 0 {
                    Some(observation::HeaderFields {
                        crinex: crinex.clone(),
                        codes: obs_codes.clone(),
                        clock_offset_applied: obs_clock_offset_applied,
                    })
                } else {
                    None
                }
            },
            ////////////////////////
            // OBSERVATION / METEO
            ////////////////////////
            meteo: {
                if met_codes.len() > 0 {
                    Some(meteo::HeaderFields {
                        codes: met_codes.clone(),
                        sensors: met_sensors.clone(),
                    })
                } else {
                    None
                }
            },
            ///////////////////////
            // CLOCKS
            ///////////////////////
            clocks: {
                if clk_codes.len() > 0 {
                    Some(clocks::HeaderFields {
                        codes: clk_codes.clone(),
                        agency: { 
                            if clk_agency_code.len() > 0 {
                                Some(clocks::Agency {
                                    code: clk_agency_code.clone(),
                                    name: clk_agency_name.clone(),
                                })
                            } else {
                                None
                            }
                        },
                        station: { 
                            if clk_station_name.len() > 0 {
                                Some(clocks::Station {
                                    name: clk_station_name.clone(),
                                    id: clk_station_id.clone(),
                                })
                            } else {
                                None
                            }
                        },
                        clock_ref: {
                            if clk_ref.len() > 0 {
                                Some(clk_ref.clone())
                            } else {
                                None
                            }
                        },
                    })
                } else {
                    None
                }
            },
            ///////////////////////
            // IONEX
            ///////////////////////
            ionex: {
                if rinex_type == Type::IonosphereMaps {
                    Some(ionex)
                } else {
                    None
                }
            },
            ///////////////////////
            // ANTEX
            ///////////////////////
            antex: {
                if let Some(pcv) = pcv {
                    Some(antex::HeaderFields {
                        pcv,
                        relative_values: ant_relative_values,
                        reference_sn: ref_ant_sn,
                    })
                } else {
                    None
                }
            },
        })
    }
    /// `Merges` self and given header
    /// we call this maethod when merging two rinex record
    /// to create the optimum combined/total RINEX file.
    /// This is not a feature of teqc.
    /// When merging:  
    ///  + retains oldest revision number  
    ///  + constellation remains identical if self & `b` share the same constellation,
    ///   otherwise, self::constellation is upgraded to `mixed`.  
    ///  + `b` comments are retained, header section comments are not analyzed   
    ///  + prefers self::attriutes over `b` attributes  
    ///  + appends (creates) `b` attributes that do not exist in self
    ///TODO: sampling interval special case
    ///TODO: rcvr_clock_offset_applied special case :
    /// apply/modify accordingly
    ///TODO: data scaling special case: apply/modify accordingly
    pub fn merge_mut (&mut self, header: &Self) -> Result<(), MergeError> {
        if self.rinex_type != header.rinex_type {
            return Err(MergeError::FileTypeMismatch)
        }

        let (a_rev, b_rev) = (self.version, header.version);
        let (a_cst, b_cst) = (self.constellation, header.constellation);
        // constellation upgrade ?
        if a_cst != b_cst {
            self.constellation = Some(Constellation::Mixed)
        }
        // retain oldest revision
        self.version = std::cmp::min(a_rev, b_rev);
        for c in &header.comments {
            self.comments.push(c.to_string()) 
        } 
        // leap second new info ?
        if let Some(leap) = header.leap {
            if self.leap.is_none() {
                self.leap = Some(leap)
            }
        }
        if let Some(delta) = header.gps_utc_delta {
            if self.gps_utc_delta.is_none() {
                self.gps_utc_delta = Some(delta)
            }
        }
        if let Some(rcvr) = &header.rcvr {
            if self.rcvr.is_none() {
                self.rcvr = Some(
                    hardware::Rcvr {
                        model: rcvr.model.clone(),
                        sn: rcvr.sn.clone(),
                        firmware: rcvr.firmware.clone(),
                    }
                )
            }
        }
        if let Some(ant) = &header.ant {
            if self.ant.is_none() {
                self.ant = Some(
                    hardware::Antenna {
                        model: ant.model.clone(),
                        sn: ant.sn.clone(),
                        coords: ant.coords.clone(),
                        height: ant.height,
                        eastern_ecc: ant.eastern_ecc,
                        northern_ecc: ant.northern_ecc,
                    }
                )
            }
        }
        //TODO append new array
        /*if let Some(a) = &header.sensors {
            if let Some(b) = &self.sensors {
                for sens in a {
                    if !b.contains(sens) {
                        b.push(*sens)
                    }
                }
            } else {
                self.sensors = Some(a.to_vec())
            }
        }*/
        if let Some(coords) = &header.coords {
            if self.coords.is_none() {
                self.coords = Some(rust_3d::Point3D {
                    x: coords.x,
                    y: coords.y,
                    z: coords.z,
                })
            }
        }
        if let Some(wavelengths) = header.wavelengths {
            if self.wavelengths.is_none() {
                self.wavelengths = Some(wavelengths)
            }
        }
        //TODO as mut ref
        /*if let Some(a) = &header.obs_codes {
            if let Some(&mut b) = self.obs_codes.as_ref() {
                for (k, v) in a {
                    b.insert(*k, v.to_vec());
                }
            } else {
                self.obs_codes = Some(a.clone())
            }
        }*/
        
        /*if let Some(a) = header.data_scaling {
            if let Some(b) = self.data_scaling {

            } else {

            }
        } else {
            if let Some(b) = self.data_scaling {

            }
        }*/

        Ok(())
    }
    
    /// Returns true if self is a `Compressed RINEX`
    pub fn is_crinex (&self) -> bool { 
        if let Some(obs) = &self.obs {
            obs.crinex.is_some()
        } else {
            false
        }
    }

    /// Creates a Basic Header structure
    /// for NAV RINEX
    pub fn basic_nav() -> Self {
        Self::default()
            .with_type(Type::NavigationData)
    }
    
    /// Creates a Basic Header structure
    /// for OBS RINEX
    pub fn basic_obs() -> Self {
        Self::default()
            .with_type(Type::ObservationData)
    }

    /// Returns Header structure with desired RINEX type
    pub fn with_type(&self, t: Type) -> Self {
        let mut s = self.clone();
        s.rinex_type = t;
        s
    } 

    /// Adds general information to Self
    pub fn with_general_infos (&self, program: &str, run_by: &str, agency: &str) -> Self {
        let mut s = self.clone();
        s.program = program.to_string();
        s.run_by = run_by.to_string();
        s.agency = agency.to_string();
        s
    }

    /// Adds crinex generation attributes to self,
    /// has no effect if this is not an Observation Data header.
    pub fn with_crinex (&self, c: observation::Crinex) -> Self {
        let mut s = self.clone();
        if let Some(ref mut obs) = s.obs {
            obs.crinex = Some(c)
        }
        s
    }

    /// Adds receiver information to self
    pub fn with_rcvr (&self, r: hardware::Rcvr) -> Self {
        let mut s = self.clone();
        s.rcvr = Some(r);
        s
    }
    
    /// Adds antenna information to self
    pub fn with_antenna (&self, a: hardware::Antenna) -> Self {
        let mut s = self.clone();
        s.ant = Some(a);
        s
    }

    /// Adds desired constellation to Self
    pub fn with_constellation (&self, c: Constellation) -> Self {
        let mut s = self.clone();
        s.constellation = Some(c);
        s
    }

    /// adds comments to Self
    pub fn with_comments (&self, c: Vec<String>) -> Self {
        let mut s = self.clone();
        s.comments = c.clone();
        s
    }
}

impl std::fmt::Display for Header {
    /// `header` formatter, mainly for 
    /// `RINEX` file production purposes
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_crinex() {
            // two special header lines
        }
        // RINEX VERSION / TYPE 
        write!(f, "{:6}.{:02}           ", self.version.major, self.version.minor)?;
        match self.rinex_type {
            Type::NavigationData => {
                match self.constellation {
                    Some(Constellation::Glonass) => {
                        // Glonass Special case
                        write!(f,"{:<20}", "G: GLONASS NAV DATA")?;
                        write!(f,"{:<20}", "")?;
                        write!(f,"{}", "RINEX VERSION / TYPE\n")?
                    },
                    Some(c) => {
                        write!(f,"{:<20}", "NAVIGATION DATA")?;
                        write!(f,"{:<20}", c.to_1_letter_code())?;
                        write!(f,"{:<20}", "RINEX VERSION / TYPE\n")?
                    },
                    _ => panic!("constellation must be specified when formatting a NavigationData") 
                }
            },
            Type::ObservationData => {
                match self.constellation {
                    Some(c) => {
                        write!(f,"{:<20}", "OBSERVATION DATA")?;
                        write!(f,"{:<20}", c.to_1_letter_code())?;
                        write!(f,"{:<20}", "RINEX VERSION / TYPE\n")?
                    },
                    _ => panic!("constellation must be specified when formatting ObservationData")
                }
            },
            Type::MeteoData => {
                write!(f,"{:<20}", "METEOROLOGICAL DATA")?;
                write!(f,"{:<20}", "")?;
                write!(f,"{:<20}", "RINEX VERSION / TYPE\n")?
            },
            Type::ClockData => todo!(),
            Type::AntennaData => todo!(),
            Type::IonosphereMaps => todo!(),
        }
        // COMMENTS 
        for comment in self.comments.iter() {
            write!(f, "{:<60}", comment)?;
            write!(f, "COMMENT\n")?
        }
        // PGM / RUN BY / DATE
        write!(f, "{:<20}", self.program)?;
        write!(f, "{:<20}", self.run_by)?;
        write!(f, "{:<20}", self.date)?; //TODO
        write!(f, "{}", "PGM / RUN BY / DATE\n")?; 
        // OBSERVER / AGENCY
        write!(f, "{:<20}", self.observer)?;
        write!(f, "{:<40}", self.agency)?;
        write!(f, "OBSERVER / AGENCY\n")?; 
        // MARKER NAME
        write!(f, "{:<20}", self.station)?;
        write!(f, "{:<40}", " ")?;
        write!(f, "{}", "MARKER NAME\n")?;
        // MARKER NUMBER
        write!(f, "{:<20}", self.station_id)?;
        write!(f, "{:<40}", " ")?;
        write!(f, "{}", "MARKER NUMBER\n")?;
        // ANT
        if let Some(ant) = &self.ant {
            write!(f, "{:<20}", ant.sn)?;
            write!(f, "{:<40}", ant.model)?;
            write!(f, "{}", "ANT # / TYPE\n")?;
            if let Some(coords) = &ant.coords {
                write!(f, "{:<20}", coords.x)?;
                write!(f, "{:<20}", coords.y)?;
                write!(f, "{:<20}", coords.z)?;
                write!(f, "{}", "APPROX POSITION XYZ\n")?
            }
            if let Some(h) = &ant.height {
                write!(f, "{:<20}", h)?;
                write!(f, "{:<20}", ant.eastern_ecc.unwrap_or(0.0_f32))?;
                write!(f, "{:<20}", ant.northern_ecc.unwrap_or(0.0_f32))?;
                write!(f, "{}", "ANTENNA: DELTA H/E/N\n")?
            }
        }
        // RCVR
        if let Some(rcvr) = &self.rcvr {
            write!(f, "{:<20}", rcvr.sn)?; 
            write!(f, "{:<20}", rcvr.model)?; 
            write!(f, "{:<20}", rcvr.firmware)?;
            write!(f, "REC # / TYPE / VERS\n")?
        }
        // INTERVAL
        if let Some(interval) = &self.sampling_interval {
            write!(f, "{:10.3}", interval)?;
            write!(f, "{:<50}", "")?;
            write!(f, "INTERVAL\n")?
        }
        // OBS codes
        match self.rinex_type {
            Type::ObservationData => {
                if let Some(obs) = &self.obs {
                    match self.version.major {
                        1|2 => { // old revisions
                            for (_constell, codes) in obs.codes.iter() {
                                let mut line = format!("{:6}", codes.len()); 
                                for i in 0..codes.len() {
                                    if (i+1)%10 == 0 {
                                        line.push_str("# / TYPES OF OBS\n");
                                        write!(f, "{}", line)?;
                                        line.clear();
                                        line.push_str(&format!("{:<6}", ""));
                                    }
                                    line.push_str(&format!(" {:>5}", codes[i]));
                                }
                                line.push_str(&format!("{:<width$}", "", width=60-line.len()));
                                line.push_str("# / TYPES OF OBS\n"); 
                                write!(f, "{}", line)?;
                                break // only once
                            }
                        },
                        _ => { // modern revisions
                            for (constell, codes) in obs.codes.iter() {
                                let mut line = format!("{:<4}", constell.to_1_letter_code());
                                line.push_str(&format!("{:2}", codes.len())); 
                                for i in 0..codes.len() {
                                    if (i+1)%14 == 0 {
                                        line.push_str(&format!("{:<width$}", "", width=60-line.len()));
                                        line.push_str("SYS / # / OBS TYPES\n"); 
                                        write!(f, "{}", line)?;
                                        line.clear();
                                        line.push_str(&format!("{:<6}", ""));
                                    }
                                    line.push_str(&format!(" {}", codes[i]))
                                }
                                line.push_str(&format!("{:<width$}", "", width=60-line.len()));
                                line.push_str("SYS / # / OBS TYPES\n"); 
                                write!(f, "{}", line)?
                            }
                        },
                    }
                } else {
                    panic!("Observation RINEX with no `obs codes` specified")
                }
            },
            Type::MeteoData => {
                if let Some(obs) = &self.meteo {
                    let codes = &obs.codes;
                    let mut line = format!("{:6}", codes.len()); 
                    for i in 0..codes.len() {
                        if (i+1)%9 == 0 {
                            line.push_str("# / TYPES OF OBS\n");
                            write!(f, "{}", line)?;
                            line.clear();
                            line.push_str(&format!("{:<6}", ""));
                        }
                        line.push_str(&format!(" {:>5}", codes[i]));
                    }
                    line.push_str(&format!("{:<width$}", "", width=60-line.len()));
                    line.push_str("# / TYPES OF OBS\n"); 
                    write!(f, "{}", line)?;
                } else {
                    panic!("Meteo RINEX with no `obs codes` specified")
                }
            },
            _ => {},
        }
        // LEAP
        if let Some(leap) = &self.leap {
            write!(f, "{:6}", leap.leap)?;
            if let Some(delta) = &leap.delta_tls {
                write!(f, "{:6}", delta)?;
                write!(f, "{:6}", leap.week.unwrap_or(0))?;
                write!(f, "{:6}", leap.day.unwrap_or(0))?;
                if let Some(system) = &leap.system {
                    write!(f, "{:<10}", system.to_3_letter_code())?
                } else {
                    write!(f, "{:<10}", " ")?
                }
            } else {
                write!(f, "{:<40}", " ")?
            }
            write!(f, "LEAP SECONDS\n")?
        }
        // SENSOR(s)
        if let Some(meteo) = &self.meteo {
            let sensors = &meteo.sensors;
            for sensor in sensors {
                write!(f, "{}", sensor)?
            }
        }
        // END OF HEADER
        write!(f, "{:>74}", "END OF HEADER\n")
    }
}
