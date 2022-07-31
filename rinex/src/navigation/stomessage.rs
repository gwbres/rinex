//! `Navigation` new STO System Time Offset messages

/// System Time Message 
#[derive(Debug, Clone)]
#[derive(Default)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct Message {
    /// Time system
    pub system: String,
    pub utc: String,
    /// Message transmmission time [s] of GNSS week
    pub t_tm: u32,
    /// ([sec], [sec.sec⁻¹], [sec.sec⁻²])
    pub a: (f64,f64,f64),
}
