//! `MeteoData` related structures & methods
use std::io::Write;
use thiserror::Error;
use std::str::FromStr;
use std::collections::{BTreeMap, HashMap};
use crate::epoch;
use crate::header;
use crate::version;
use crate::header::Header;

use crate::meteo::observable::Observable;

/// `MET` record comprises raw data sorted by observable code
/// and by epoch
pub type Record = BTreeMap<epoch::Epoch, HashMap<Observable, f32>>;

/// Returns true if given line matches a new Meteo Record `epoch`
pub fn is_new_epoch (line: &str, v: version::Version) -> bool {
    if v.major < 4 {
        let min_len = " 15  1  1  0  0  0";
        if line.len() < min_len.len() { // minimum epoch descriptor 
            return false
        }
        let datestr = &line[1..min_len.len()]; 
        epoch::str2date(datestr).is_ok() // valid epoch descriptor
    } else {
        let min_len = " 2021  1  7  0  0  0";
        if line.len() < min_len.len() { // minimum epoch descriptor
            return false
        }
        let datestr = &line[1..min_len.len()]; 
        epoch::str2date(datestr).is_ok() // valid epoch descriptor
    }
}

#[derive(Error, Debug)]
/// Meteo Data `Record` parsing specific errors
pub enum Error {
    #[error("failed to parse date")]
    ParseDateError(#[from] epoch::ParseDateError),
    #[error("failed to integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

/// Builds `Record` entry for `MeteoData`
pub fn build_record_entry (header: &Header, content: &str) 
        -> Result<(epoch::Epoch, HashMap<Observable, f32>), Error> 
{
    let mut lines = content.lines();
    let mut line = lines.next()
        .unwrap();

	let mut map : HashMap<Observable, f32> = HashMap::with_capacity(3);

	// epoch.secs is not f32 as usual
	// Y is 4 digit number as usual for V > 2
	//let (date, rem) = line.split_at(offset);
	let (mut y, m, d, h, min, sec, mut offset) : (i32, u32, u32, u32, u32, u32, usize) 
		= match header.version.major > 2 {
		true => {
			(i32::from_str_radix(line[0..5].trim(),10)?, // Y: 4 digit
			u32::from_str_radix(line[5..8].trim(),10)?, // m
			u32::from_str_radix(line[8..11].trim(),10)?, // d
			u32::from_str_radix(line[11..14].trim(),10)?, // h
			u32::from_str_radix(line[14..17].trim(),10)?, // m
			u32::from_str_radix(line[17..20].trim(),10)?, // s
			20)
		},
		false => {
			(i32::from_str_radix(line[0..3].trim(),10)?, // Y: 2 digit
			u32::from_str_radix(line[3..6].trim(),10)?, // m
			u32::from_str_radix(line[6..9].trim(),10)?,// d
			u32::from_str_radix(line[9..12].trim(),10)?,// h
			u32::from_str_radix(line[12..15].trim(),10)?,// m
			u32::from_str_radix(line[15..18].trim(),10)?,// s
			18)
		},
	};
	if y < 100 { // 2 digit nb case
    	if y > 90 {
        	y += 1900
    	} else {
			y += 2000
		}
	}
	let date = chrono::NaiveDate::from_ymd(y,m,d)
		.and_hms(h,min,sec);
	let flag = epoch::EpochFlag::default();
	let epoch = epoch::Epoch::new(date, flag);

	let codes = &header.meteo
        .as_ref()
        .unwrap()
        .codes;
	let n_codes = codes.len();
	let nb_lines : usize = num_integer::div_ceil(n_codes, 8).into(); 
	let mut code_index : usize = 0;

	for i in 0..nb_lines {
		for _ in 0..8 {
			let code = &codes[code_index];
			let obs : Option<f32> = match f32::from_str(&line[offset..offset+7].trim()) {
				Ok(f) => Some(f),
				Err(_) => None,
			};

			if let Some(obs) = obs {
				map.insert(code.clone(), obs); 
			}
			code_index += 1;
			if code_index >= n_codes {
				break
			}

			offset += 7;
			if offset >= line.len() {
				break
			}
		} // 1:8

		if i < nb_lines-1 {
			if let Some(l) = lines.next() {
				line = l;
			} else {
				break
			}
		}
	} // nb lines
	Ok((epoch, map))
}

/// Pushes meteo record into given file writer
pub fn to_file (header: &header::Header, record: &Record, mut writer: std::fs::File) -> std::io::Result<()> {
    let obscodes = &header.meteo.as_ref().unwrap().codes;
    for (epoch, obs) in record.iter() {
        if header.version.major > 3 {
            let _ = write!(writer, " {}", epoch.date.format("%Y %_m %_d %_H %_M %_S").to_string());
        } else {
            let _ = write!(writer, " {}", epoch.date.format("%y %_m %_d %_H %_M %_S").to_string());
        }
        let mut index = 0;
        for code in obscodes.iter() { 
            if let Some(data) = obs.get(code) {
                let _ = write!(writer, "{:7.1}", data);
            } else {
                let _ = write!(writer, "       ");
            }
            if (index+1) %8 == 0 {
                let _ = write!(writer, "\n");
            }
            index += 1;
        }
        write!(writer, "\n")?
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn new_epoch() {
        let content = " 22  1  4  0  0  0  993.4   -6.8   52.9    1.6  337.0    0.0    0.0";
        assert_eq!(is_new_epoch(content, 
            version::Version {
                major: 2,
                minor: 0,
            }), true);
        let content = " 22  1  4  0  0  0  993.4   -6.8   52.9    1.6  337.0    0.0    0.0";
        assert_eq!(is_new_epoch(content, 
            version::Version {
                major: 2,
                minor: 0,
            }), true);
        let content = " 22  1  4  9 55  0  997.9   -6.4   54.2    2.9  342.0    0.0    0.0";
        assert_eq!(is_new_epoch(content, 
            version::Version {
                major: 2,
                minor: 0,
            }), true);
        let content = " 22  1  4 10  0  0  997.9   -6.3   55.4    3.4  337.0    0.0    0.0";
        assert_eq!(is_new_epoch(content, 
            version::Version {
                major: 2,
                minor: 0,
            }), true);
        let content = " 08  1  1  0  0  1 1018.0   25.1   75.9    1.4   95.0    0.0    0.0";
        assert_eq!(is_new_epoch(content, 
            version::Version {
                major: 2,
                minor: 0,
            }), true);
        let content = " 2021  1  7  0  0  0  993.3   23.0   90.0";
        assert_eq!(is_new_epoch(content,
            version::Version {
                major: 4,
                minor: 0,
            }), true);
    }
}
