//! Meteo sensor
use thiserror::Error;
use crate::meteo::observable::Observable;

/// Meteo Observation Sensor
#[derive(Clone, Debug)]
#[derive(PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Sensor {
	/// Model of this sensor
	pub model: String,
	/// Type of sensor
	pub sensor_type: String,
	/// Sensor accuracy [°C,..]
	pub accuracy: Option<f32>,
	/// Physics measured by this sensor
	pub observable: Observable,
    /// Posible sensor location (ECEF) and possible
    /// height eccentricity
    pub position: Option<(f64,f64,f64,f64)>,
}

#[derive(Error, Debug)]
pub enum ParseSensorError {
    #[error("failed to identify observable")]
    ParseObservableError(#[from] strum::ParseError),
    #[error("failed to parse accuracy field")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

impl Default for Sensor {
    fn default() -> Sensor {
        Sensor {
            model: String::new(),
            sensor_type: String::new(),
            observable: Observable::default(), 
            accuracy: None,
            position: None,
        }
    }
}

impl std::str::FromStr for Sensor {
    type Err = ParseSensorError;
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        let (model, rem) = content.split_at(20);
        let (s_type, rem) = rem.split_at(20 +6);
        let (accuracy, rem) = rem.split_at(7 +4);
        let (observable, _) = rem.split_at(2);
        Ok(Self {
            model: {
                if model.trim().len() == 0 {
                    String::from("Unknown")
                } else {
                    model.trim().to_string()
                }
            },
            sensor_type: {
                if s_type.trim().len() == 0 {
                    String::from("Unknown")
                } else {
                    s_type.trim().to_string()
                }
            },
            accuracy: {
                if let Ok(f) = f32::from_str(accuracy.trim()) {
                    Some(f)
                } else {
                    None
                }
            },
            observable: Observable::from_str(observable.trim())?, 
            position: None,
        })
    }
}

impl std::fmt::Display for Sensor {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:<20}", self.model)?; 
        write!(f, "{:<30}", self.sensor_type)?; 
        if let Some(acc) = self.accuracy {
            write!(f, "{:1.1}", acc)?;
        } else {
            write!(f, "  ")?;
        }
        write!(f, "    {} ", self.observable)?;
        write!(f, "SENSOR MOD/TYPE/ACC\n")?;
        if let Some((x,y,z,h)) = self.position {
            write!(f, "        {:.4}        {:.4}        {:.4}        {:.4}", x, y, z, h)?;
            write!(f, "{} SENSOR POS XYZ/H", self.observable)?
        }
        Ok(())
    }
}

impl Sensor {
    pub fn with_position (&self, pos: (f64,f64,f64,f64)) -> Self {
        let mut s = self.clone();
        s.position = Some(pos);
        s
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_sensor() {
        let s = Sensor::from_str("                                                  0.0    PR ");
        assert_eq!(s.is_ok(), true);
        let s = s.unwrap();
        assert_eq!(s.model, "Unknown");
        assert_eq!(s.sensor_type, "Unknown");
        assert_eq!(s.accuracy, Some(0.0));

        let s = Sensor::from_str("PAROSCIENTIFIC      740-16B                       0.2    PR SENSOR MOD/TYPE/ACC");
        assert_eq!(s.is_ok(), true);
        let s = Sensor::from_str("                                                  0.0    PR SENSOR MOD/TYPE/ACC");
        assert_eq!(s.is_ok(), true);
    }
}
