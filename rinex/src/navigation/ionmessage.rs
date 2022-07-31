//! `Navigation` new ION Ionospheric model messages
use bitflags::bitflags;
use crate::epoch;
use thiserror::Error;
use std::str::FromStr;

/// Model parsing error
#[derive(Debug, Error)]
pub enum Error {
    #[error("ng model missing 1st line")]
    NgModelMissing1stLine,
    #[error("kb model missing 1st line")]
    KbModelMissing1stLine,
    #[error("bd model missing 1st line")]
    BdModelMissing1stLine,
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
    #[error("failed to parse float data")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse date field")]
    ParseDateError(#[from] epoch::ParseDateError),
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
            _ => return Err(Error::NgModelMissing1stLine)
        };
        let (epoch, rem) = line.split_at(23);
        let (a0, rem) = rem.split_at(19);
        let (a1, a2) = rem.split_at(19);
        
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::KbModelMissing2ndLine)
        };
        let (a3, rem) = line.split_at(23);
        let (b0, rem) = rem.split_at(19);
        let (b1, b2) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::KbModelMissing3rdLine)
        };
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

        let date = epoch::str2date(epoch.trim())?;
        let alpha = (
            f64::from_str(a0.trim()).unwrap_or(0.0_f64),
            f64::from_str(a1.trim()).unwrap_or(0.0_f64),
            f64::from_str(a2.trim()).unwrap_or(0.0_f64),
            f64::from_str(a3.trim()).unwrap_or(0.0_f64),
        );
        let beta = (
            f64::from_str(b0.trim()).unwrap_or(0.0_f64),
            f64::from_str(b1.trim()).unwrap_or(0.0_f64),
            f64::from_str(b2.trim()).unwrap_or(0.0_f64),
            f64::from_str(b3.trim()).unwrap_or(0.0_f64),
        );

        Ok((epoch::Epoch {
            date,
            flag: epoch::EpochFlag::Ok,
        },
        Self {
            alpha,
            beta,
            region,
        }))
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
            _ => return Err(Error::NgModelMissing1stLine)
        };
        let (epoch, rem) = line.split_at(23);
        let (a0, rem) = rem.split_at(19);
        let (a1, rem) = rem.split_at(19);
        
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::NgModelMissing2ndLine)
        };
        
        let date = epoch::str2date(epoch.trim())?;
        let a = (
            f64::from_str(a0.trim())?,
            f64::from_str(a1.trim())?,
            f64::from_str(rem.trim())?,
        );
        let f = f64::from_str(line.trim())?;
        Ok((epoch::Epoch {
            date,
            flag: epoch::EpochFlag::Ok,
        },
        Self {
            a,
            region: NgRegionFlags::from_bits(f as u16).unwrap_or(NgRegionFlags::empty()),
        }))
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
            _ => return Err(Error::BdModelMissing1stLine)
        };
        let (epoch, rem) = line.split_at(23);
        let (a0, rem) = rem.split_at(19);
        let (a1, a2) = rem.split_at(19);
        
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::KbModelMissing2ndLine)
        };
        let (a3, rem) = line.split_at(23);
        let (a4, rem) = rem.split_at(19);
        let (a5, a6) = rem.split_at(19);
        
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::KbModelMissing3rdLine)
        };
        let (a7, a8) = rem.split_at(23);
        
        let date = epoch::str2date(epoch.trim())?;
        let alpha = (
            f64::from_str(a0.trim()).unwrap_or(0.0_f64),
            f64::from_str(a1.trim()).unwrap_or(0.0_f64),
            f64::from_str(a2.trim()).unwrap_or(0.0_f64),
            f64::from_str(a3.trim()).unwrap_or(0.0_f64),
            f64::from_str(a4.trim()).unwrap_or(0.0_f64),
            f64::from_str(a5.trim()).unwrap_or(0.0_f64),
            f64::from_str(a6.trim()).unwrap_or(0.0_f64),
            f64::from_str(a7.trim()).unwrap_or(0.0_f64),
            f64::from_str(a8.trim()).unwrap_or(0.0_f64),
        );
        Ok((epoch::Epoch {
            date,
            flag: epoch::EpochFlag::Ok,
        },
        Self {
            alpha
        }))
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
