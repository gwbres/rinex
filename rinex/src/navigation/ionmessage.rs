//! `Navigation` new ION Ionospheric model messages
use bitflags::bitflags;

/// Klobuchar model payload,
/// we don't know how to parse the possible extra Region Code yet
#[derive(Default)]
#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct KbModel {
    /// Alpha coefficients 
    /// ([sec], [sec.semi-circle⁻¹], [sec.semi-circle⁻²])
    pub alpha: (f64,f64,f64),
    /// Beta coefficients
    /// ([sec], [sec.semi-circle⁻¹], [sec.semi-circle⁻²])
    pub beta: (f64,f64,f64),
}

bitflags! {
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
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct NgModel {
    /// a_i coefficients
    /// ([sfu], [sfu.semi-circle⁻¹], [sfu.semi-circle⁻²])
    pub a: (f64,f64,f64), 
    /// Region flags
    pub region: NgRegionFlags,
}

/// BDGIM Model payload
#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct BdModel {
    /// Alpha coefficients [TECu]
    pub alpha: (f64,f64,f64,f64,f64,f64,f64),
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
