//! `Navigation` new EOP Earth Orientation messages
use crate::epoch;
use thiserror::Error;
use std::str::FromStr;

/// Message Parsing error 
#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse date field")]
    ParseDateError(#[from] epoch::ParseDateError),
    #[error("eop message missing 1st line")]
    EopMissing1stLine,
    #[error("eop message missing 2nd line")]
    EopMissing2ndLine,
    #[error("eop message missing 3rd line")]
    EopMissing3rdLine,
}

/// Earth Orientation Message 
#[derive(Debug, Clone)]
#[derive(Default)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct Message {
    /// ([arc-sec], [arc-sec.day⁻¹], [arc-sec.day⁻²])
    x: (f64,f64,f64),
    /// ([arc-sec], [arc-sec.day⁻¹], [arc-sec.day⁻²])
    y: (f64,f64,f64),
    /// Message transmmission time [s] of GNSS week
    t_tm: u32,
    /// Delta UT1 ([sec], [sec.day⁻¹], [-sec.day⁻²])
    dut1: (f64,f64,f64),
}

impl Message {
    pub fn parse (mut lines: std::str::Lines<'_>) -> Result<(epoch::Epoch, Self), Error> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::EopMissing1stLine)
        };
        let (epoch, rem) = line.split_at(23);
        let (xp, rem) = rem.split_at(19);
        let (dxp, ddxp) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::EopMissing2ndLine)
        };
        let (_, rem) = line.split_at(23);
        let (yp, rem) = rem.split_at(19);
        let (dyp, ddyp) = rem.split_at(19);

        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::EopMissing3rdLine)
        };
        let (t_tm, rem) = line.split_at(23);
        let (dut, rem) = rem.split_at(19);
        let (ddut, dddut) = rem.split_at(19);

        let date = epoch::str2date(epoch.trim())?;
        let x = (
            f64::from_str(xp.trim()).unwrap_or(0.0_f64),
            f64::from_str(dxp.trim()).unwrap_or(0.0_f64),
            f64::from_str(ddxp.trim()).unwrap_or(0.0_f64),
        );
        let y = (
            f64::from_str(yp.trim()).unwrap_or(0.0_f64),
            f64::from_str(dyp.trim()).unwrap_or(0.0_f64),
            f64::from_str(ddyp.trim()).unwrap_or(0.0_f64),
        );
        let t_tm = f64::from_str(t_tm.trim()).unwrap_or(0.0_f64);
        let dut1 = (
            f64::from_str(dut.trim()).unwrap_or(0.0_f64),
            f64::from_str(ddut.trim()).unwrap_or(0.0_f64),
            f64::from_str(dddut.trim()).unwrap_or(0.0_f64),
        );

        Ok((epoch::Epoch {
            date,
            flag: epoch::EpochFlag::Ok,
        },
        Self {
            x,
            y,
            t_tm: t_tm as u32,
            dut1,
        }))
    }
}
