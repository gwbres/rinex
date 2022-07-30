//! `Navigation` new ION Ionospheric model messages
use bitflags::bitflags;
use crate::epoch;
use thiserror::Error;
use std::str::FromStr;

/// Model parsing error
#[derive(Debug, Error)]
pub enum Error {
    #[error("ng model missing 2nd line")]
    NgModelMissing2ndLine,
    #[error("kb model missing 2nd line")]
    KbModelMissing2ndLine,
    #[error("bd model missing 2nd line")]
    BdModelMissing2ndLine,
    #[error("kb model missing 3rd line")]
    KbModelMissing3rdLine,
    #[error("bd model missing 3rd line")]
    BdModelMissing3rdLine,
    #[error("missing data fields")]
    MissingData,
}

/// Klobuchar Parameters region
#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub enum KbRegionCode {
    /// Coefficients apply to wide area
    WideArea = 0,
    /// Japan Area coefficients
    JapanArea = 1,
}

impl Default for KbRegionCode {
    fn default() -> Self {
        Self::WideArea
    }
}

/// Klobuchar model payload,
/// we don't know how to parse the possible extra Region Code yet
#[derive(Default)]
#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct KbModel {
    /// Alpha coefficients 
    /// ([sec], [sec.semi-circle⁻¹], [sec.semi-circle⁻²], [sec.semi-circle⁻³])
    pub alpha: (f64,f64,f64,f64),
    /// Beta coefficients
    /// ([sec], [sec.semi-circle⁻¹], [sec.semi-circle⁻²], [sec.semi-circle⁻³])
    pub beta: (f64,f64,f64,f64),
    /// Region flag
    pub region: KbRegionCode,
}

impl KbModel {
    pub fn parse (mut lines: std::str::Lines<'_>) -> Result<(epoch::Epoch, Self), Error> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::NgModelMissing2ndLine)
        };
        if line.len() < 23 {
            return Err(Error::MissingData)
        }
        let (epoch, rem) = line.split_at(23);
        if rem.len() < 19 {
            return Err(Error::MissingData)
        }
        let (a0, rem) = rem.split_at(19);
        if rem.len() < 19 {
            return Err(Error::MissingData)
        }
        let (a1, a2) = rem.split_at(19);
        
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::KbModelMissing2ndLine)
        };
        if line.len() < 23 {
            return Err(Error::MissingData)
        }
        let (a3, rem) = line.split_at(23);
        if rem.len() < 19 {
            return Err(Error::MissingData)
        }
        let (b0, rem) = rem.split_at(19);
        if rem.len() < 19 {
            return Err(Error::MissingData)
        }
        let (b1, b2) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::KbModelMissing3rdLine)
        };
        if line.len() < 23 {
            return Err(Error::MissingData)
        }
        let (b3, region) = line.split_at(23);

        let region: KbRegionCode = match region.trim().len() {
            0 => KbRegionCode::WideArea,
            _ => {
                if let Ok(f) = f64::from_str(region.trim()) {
                    let code = f as u8;
                    if code == 1 {
                        KbRegionCode::JapanArea
                    } else {
                        KbRegionCode::WideArea
                    }
                } else {
                    KbRegionCode::WideArea
                }
            },
        };

        if let Ok(date) = epoch::str2date(epoch.trim()) {
            if let Ok(a0) = f64::from_str(a0.trim()) {
                if let Ok(a1) = f64::from_str(a1.trim()) {
                    if let Ok(a2) = f64::from_str(a2.trim()) {
                        if let Ok(a3) = f64::from_str(a3.trim()) {
                            if let Ok(b0) = f64::from_str(b0.trim()) {
                                if let Ok(b1) = f64::from_str(b1.trim()) {
                                    if let Ok(b2) = f64::from_str(b2.trim()) {
                                        if let Ok(b3) = f64::from_str(b3.trim()) {
                                            return Ok((
                                                epoch::Epoch {
                                                    date,
                                                    flag: epoch::EpochFlag::Ok,
                                                },
                                                Self {
                                                    alpha: (a0,a1,a2,a3),
                                                    beta: (b0,b1,b2,b3),
                                                    region,
                                                }))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(Error::MissingData)
    }
}

bitflags! {
    #[derive(Default)]
    #[cfg_attr(feature = "with-serde", derive(Serialize))]
    pub struct NgRegionFlags: u16 {
        const REGION5 = 0x01;
        const REGION4 = 0x02;
        const REGION3 = 0x04;
        const REGION2 = 0x08;
        const REGION1 = 0x10;
    }
}

/// Nequick-G Model payload
#[derive(Debug, Clone)]
#[derive(Default)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct NgModel {
    /// a_i coefficients
    /// ([sfu], [sfu.semi-circle⁻¹], [sfu.semi-circle⁻²])
    pub a: (f64,f64,f64), 
    /// Region flags
    pub region: NgRegionFlags,
}

impl NgModel {
    pub fn parse(mut lines: std::str::Lines<'_>) -> Result<(epoch::Epoch, Self), Error> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::NgModelMissing2ndLine)
        };
        if line.len() < 23 {
            return Err(Error::MissingData)
        }
        let (epoch, rem) = line.split_at(23);
        if rem.len() < 19 {
            return Err(Error::MissingData)
        }
        let (a0, rem) = rem.split_at(19);
        if rem.len() < 19 {
            return Err(Error::MissingData)
        }
        let (a1, rem) = rem.split_at(19);
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::NgModelMissing2ndLine)
        };
        if line.len() < 23 {
            return Err(Error::MissingData)
        }
        if let Ok(date) = epoch::str2date(epoch.trim()) {
            if let Ok(a0) = f64::from_str(a0.trim()) {
                if let Ok(a1) = f64::from_str(a1.trim()) {
                    if let Ok(a2) = f64::from_str(rem.trim()) {
                        if let Ok(f) = f64::from_str(line.trim()) {
                            return Ok((
                                epoch::Epoch {
                                    date,
                                    flag: epoch::EpochFlag::Ok,
                                },
                                Self {
                                    a: (a0,a1,a2),
                                    region: NgRegionFlags::from_bits(f as u16).unwrap_or(NgRegionFlags::empty()),
                                }
                            ))
                        }
                    }
                }
            }
        }
        Err(Error::MissingData)
    }
}

/// BDGIM Model payload
#[derive(Debug, Clone)]
#[derive(Default)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct BdModel {
    /// Alpha coefficients [TECu]
    pub alpha: (f64,f64,f64,f64,f64,f64,f64,f64,f64),
}

impl BdModel {
    pub fn parse (mut lines: std::str::Lines<'_>) -> Result<(epoch::Epoch, Self), Error> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::BdModelMissing2ndLine)
        };
        let (epoch, rem) = line.split_at(23);
        if rem.len() < 19 {
            return Err(Error::MissingData)
        }
        let (a0, rem) = rem.split_at(19);
        if rem.len() < 19 {
            return Err(Error::MissingData)
        }
        let (a1, a2) = rem.split_at(19);
        
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::KbModelMissing2ndLine)
        };
        let (a3, rem) = line.split_at(23);
        if rem.len() < 19 {
            return Err(Error::MissingData)
        }
        let (a4, rem) = rem.split_at(19);
        if rem.len() < 19 {
            return Err(Error::MissingData)
        }
        let (a5, a6) = rem.split_at(19);
        
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::KbModelMissing2ndLine)
        };
        let (a7, rem) = rem.split_at(23);
        if rem.len() < 19 {
            return Err(Error::MissingData)
        }
        if let Ok(a0) = f64::from_str(a0.trim()) {
            if let Ok(a1) = f64::from_str(a1.trim()) {
                if let Ok(a2) = f64::from_str(a2.trim()) {
                    if let Ok(a3) = f64::from_str(a3.trim()) {
                        if let Ok(a4) = f64::from_str(a4.trim()) {
                            if let Ok(a5) = f64::from_str(a5.trim()) {
                                if let Ok(a6) = f64::from_str(a6.trim()) {
                                    if let Ok(a7) = f64::from_str(a7.trim()) {
                                        if let Ok(a8) = f64::from_str(rem.trim()) {
                                            if let Ok(date) = epoch::str2date(epoch.trim()) {
                                                return Ok((epoch::Epoch {
                                                    date,
                                                    flag: epoch::EpochFlag::Ok,
                                                },
                                                Self {
                                                    alpha: (a0,a1,a2,a3,a4,a5,a6,a7,a8), 
                                                }))
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(Error::MissingData)
    }
}

/// Existing ION Message declinations
#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub enum Message {
    /// Klobuchar Model
    KlobucharModel(KbModel),
    /// Nequick-G Model
    NequickGModel(NgModel),
    /// BDGIM Model
    BdgimModel(BdModel),
}

impl Default for Message {
    fn default() -> Self {
        Self::KlobucharModel(KbModel::default())
    }
}

impl Message {
    /// Unwraps self as Klobuchar Model
    pub fn as_klobuchar (&self) -> Option<&KbModel> {
        match self {
            Self::KlobucharModel(model) => Some(model),
            _ => None,
        }
    }
    /// Unwraps self as Nequick Model
    pub fn as_nequick (&self) -> Option<&NgModel> {
        match self {
            Self::NequickGModel(model) => Some(model),
            _ => None,
        }
    }
    /// Unwraps self as BDGIM Model
    pub fn as_bdgim (&self) -> Option<&BdModel> {
        match self {
            Self::BdgimModel(model) => Some(model),
            _ => None,
        }
    }
}
