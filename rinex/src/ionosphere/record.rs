use crate::epoch;
use thiserror::Error;
use std::str::FromStr;
use std::collections::BTreeMap;

pub fn is_new_tec_map (line: &str) -> bool {
    line.contains("START OF TEC MAP") 
}

pub fn is_new_rms_map (line: &str) -> bool {
    line.contains("START OF RMS MAP") 
}

pub fn is_new_height_map (line: &str) -> bool {
    line.contains("START OF HEIGHT MAP") 
}

pub fn is_new_map (line: &str) -> bool {
    is_new_tec_map(line)
    || is_new_rms_map(line)
    || is_new_height_map(line)
}

/// `IONEX` record is, for a given epoch,
/// a TEC map (always given), an optionnal RMS map
/// and an optionnal height map
pub type Record = BTreeMap<epoch::Epoch, (Map, Option<Map>, Option<Map>)>;

#[derive(Debug, Clone, Default)]
#[derive(PartialEq, PartialOrd)]
pub struct Coordinates {
    pub lat: f32,
    pub lon1: f32,
    pub lon2: f32,
    pub dlon: f32,
    pub h: f32,
}

/// A map is a list of data indexed by Coordinates
pub type Map = Vec<(Coordinates, Vec<f32>)>;

/*
impl Maps {
    /// Returns (properly scaled) TEC maps
    pub fn tec_maps (&self) -> Vec<(f32, f32, f32, f32, f32, Vec<f32>)> {
        self
            .tec
            .iter()
            .map(|(lat, lon1, lon2, dlon, h, data)| {
                (
                *lat, 
                *lon1, 
                *lon2, 
                *dlon, 
                *h, 
                data.iter()
                    .map(|value| {
                        *value as f32
                    })
                    .collect()
                )
            })
            .collect()
    }
    /*pub fn add_rms_map (&mut self, map: Map) {
        if let Some(mut rms) = self.rms {
            rms.push(map)
        } else {
            self.rms = Some(vec![map])
        }
    }
    pub fn add_height_map (&mut self, map: Map) {
        if let Some(mut m) = self.height {
            m.push(map)
        } else {
            self.height = Some(vec![map])
        }
    }*/
}*/

#[derive(Debug, Error)]
pub enum Error {
    #[error("nothing wrong")]
    NoError,
}

/// Builds list of identified maps and associated epoch 
pub fn build_record_entry (content: &str, exponent: i8) -> Result<(epoch::Epoch, Map), Error> {
    let lines = content.lines();
    let mut exp = exponent.clone();
    let mut epoch = epoch::Epoch::default();
    let mut coords = Coordinates::default();
    let mut map = Map::new();
    let mut data :Vec<f32> = Vec::new();
    for line in lines {
        let (content, marker) = line.split_at(60);
        if marker.contains("LAT/LON1/LON2/DLON/H") {
            if data.len() > 0 {
                // got some data buffered
                // --> append to map being built 
                map.push((coords.clone(), data.clone()));
            }
            let items : Vec<&str> = content
                .split_ascii_whitespace()
                .collect();
            if let Ok(lat) = f32::from_str(items[0].trim()) {
                if let Ok(lon1) = f32::from_str(items[1].trim()) {
                    if let Ok(lon2) = f32::from_str(items[2].trim()) {
                        if let Ok(dlon) = f32::from_str(items[3].trim()) {
                            if let Ok(h) = f32::from_str(items[3].trim()) {
                                coords = Coordinates {
                                    lat,
                                    lon1,
                                    lon2,
                                    dlon,
                                    h
                                }
                            }
                        }
                    }
                }
            }
            data.clear(); // clear for next time

        } else if marker.contains("EPOCH OF CURRENT MAP") {
            let items : Vec<&str> = content
                .trim()
                .split_ascii_whitespace()
                .collect();
            let mut datestr = items[0].to_owned(); // Y
            datestr.push_str(" ");
            datestr.push_str(items[1]); // m
            datestr.push_str(" ");
            datestr.push_str(items[2]); // d
            datestr.push_str(" ");
            datestr.push_str(items[3]); // h
            datestr.push_str(" ");
            datestr.push_str(items[4]); // m
            datestr.push_str(" ");
            datestr.push_str(items[5]); // s
            if let Ok(e) = epoch::str2date(&datestr) {
                epoch.date = e
            }

        } else if marker.contains("EXPONENT") {
            if let Ok(e) = i8::from_str_radix(content.trim(), 10) {
                exp = e
            }

        } else if content.contains("...") { // actually, this only exists in example files..
            continue

        } else if marker.contains("END OF") && marker.contains("MAP") {
            // got some residues?
            // --> terminate map being built
            if data.len() > 0 {
                map.push((coords.clone(), data.clone()))
            }
            break

        } else { // inside map; parse data from this line, append to current list
            for item in line
                    .split_ascii_whitespace()
                    .into_iter() 
            {
                if let Ok(value) = i32::from_str_radix(item, 10) {
                    let v = (value as f32) * 10.0_f32.powf(exp as f32);
                    data.push(v)
                }
            }
        }
    }
    Ok((epoch, map))
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_new_tec_map() {
        assert_eq!(is_new_tec_map("1                                                      START OF TEC MAP   "), true); 
        assert_eq!(is_new_tec_map("1                                                      START OF RMS MAP   "), false); 
    }

    #[test]
    fn test_ionex_v1_example1() {
        let content =
"     1                                                      START OF TEC MAP    
  1995    10    15     0     0     0                        EPOCH OF CURRENT MAP
    -3                                                      EXPONENT            
    85.0   0.0 355.0   5.0 200.0                            LAT/LON1/LON2/DLON/H
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000                                        
    80.0   0.0 355.0   5.0 200.0                            LAT/LON1/LON2/DLON/H
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000                                        
...                                                                             
   -85.0   0.0 355.0   5.0 200.0                            LAT/LON1/LON2/DLON/H
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000 1000
 1000 1000 1000 1000 1000 1000 1000 1000                                        
     5                                                      END OF TEC MAP     "; 
        let entry = build_record_entry(content, -1);
        //println!("{:#?}", entry);
    }
}
