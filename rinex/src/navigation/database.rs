//! `NavigationData` parser and related methods
use itertools::Itertools;

use crate::version;
use crate::constellation::Constellation;

include!(concat!(env!("OUT_DIR"),"/nav_db.rs"));

/// Identifies closest revision contained in NAV database.   
/// Closest content is later used to identify data payload.    
/// Returns None if no database entries found for requested constellation or   
/// only newer revisions found for this constellation (older revisions are always prefered) 
pub fn closest_revision (constell: Constellation, desired_rev: version::Version) -> Option<version::Version> {
    let db = &NAV_MESSAGES;
    let revisions : Vec<_> = db.iter() // match requested constellation
        .filter(|rev| rev.constellation == constell.to_3_letter_code())
        .map(|rev| &rev.revisions)
        .flatten()
        .collect();
    if revisions.len() == 0 {
        return None // ---> constell not found in dB
    }
    let major_matches : Vec<_> = revisions.iter()
        .filter(|r| i8::from_str_radix(r.major,10).unwrap() - desired_rev.major as i8 == 0)
        .collect();
    if major_matches.len() > 0 {
        // --> desired_rev.major perfectly matched
        // --> try to match desired_rev.minor perfectly
        let minor_matches : Vec<_> = major_matches.iter()
            .filter(|r| i8::from_str_radix(r.minor,10).unwrap() - desired_rev.minor as i8 == 0)
            .collect();
        if minor_matches.len() > 0 {
            // [+] .major perfectly matched
            // [+] .minor perfectly matched
            //     -> item is unique (if dB declaration is correct)
            //     -> return directly
            let major = u8::from_str_radix(minor_matches[0].major, 10).unwrap();
            let minor = u8::from_str_radix(minor_matches[0].minor, 10).unwrap();
            Some(version::Version::new(major, minor))
        } else {
            // [+] .major perfectly matched
            // [+] .minor not perfectly matched
            //    --> use closest older minor revision
            let mut to_sort : Vec<_> = major_matches
                .iter()
                .map(|r| (
                    u8::from_str_radix(r.major,10).unwrap(), // to later build object
                    u8::from_str_radix(r.minor,10).unwrap(), // to later build object
                    i8::from_str_radix(r.minor,10).unwrap() - desired_rev.minor as i8 // for filter op
                )).collect();
            to_sort
                .sort_by(|a, b| b.2.cmp(&a.2)); // sort by delta value
            let to_sort : Vec<_> = to_sort
                .iter()
                .filter(|r| r.2 < 0) // retain negative deltas : only older revisions
                .collect();
            Some(version::Version::new(to_sort[0].0, to_sort[0].1))
        }
    } else {
        // ---> desired_rev.major not perfectly matched
        // ----> use closest older major revision
        let mut to_sort : Vec<_> = revisions
            .iter()
            .map(|r| (
                u8::from_str_radix(r.major,10).unwrap(), // to later build object
                i8::from_str_radix(r.major,10).unwrap() - desired_rev.major as i8, // for filter op
                u8::from_str_radix(r.minor,10).unwrap(), // to later build object
                i8::from_str_radix(r.minor,10).unwrap() - desired_rev.minor as i8, // for filter op
            )).collect(); 
        to_sort
            .sort_by(|a,b| b.1.cmp(&a.1)); // sort by major delta value
        let to_sort : Vec<_> = to_sort
            .iter()
            .filter(|r| r.1 < 0) // retain negative deltas only : only older revisions
            .collect();
        if to_sort.len() > 0 {
            // one last case:
            //   several minor revisions for given closest major revision
            //   --> prefer highest value
            let mut to_sort : Vec<_> = to_sort
                .iter()
                .duplicates_by(|r| r.1) // identical major deltas
                .collect();
            to_sort
                .sort_by(|a,b| b.3.cmp(&a.3)); // sort by minor deltas
            let last = to_sort.last().unwrap();
            Some(version::Version::new(last.0, last.2))
        } else {
            None // only newer revisions available, 
               // older revisions are always prefered
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::navigation::record::ComplexEnum;
    use std::str::FromStr;
    #[test]
    fn test_db_sanity() {
        for n in super::NAV_MESSAGES.iter() { 
            let c = Constellation::from_str(n.constellation);
            assert_eq!(c.is_ok(), true);
            for r in n.revisions.iter() {
                let major = u8::from_str_radix(r.major, 10);
                assert_eq!(major.is_ok(), true);
                let minor = u8::from_str_radix(r.minor, 10);
                assert_eq!(minor.is_ok(), true);
                for item in r.items.iter() {
                    let (k, v) = item;
                    if !k.eq(&"spare") {
                        let test : String;
                        if v.eq(&"f32") {
                            test = String::from("0.0")
                        } else if v.eq(&"f64") {
                            test = String::from("0.0")
                        } else if v.eq(&"u8") {
                            test = String::from("10")
                        } else {
                            test = String::from("hello")
                        }
                        let e = ComplexEnum::new(v, &test);
                        assert_eq!(e.is_ok(), true);
                    }
                }
            }
        }
    }
    #[test]
    fn test_revision_finder() {
        let found = closest_revision(Constellation::Mixed, version::Version::default());
        assert_eq!(found, None); // Constellation::Mixed is not contained in db!
        // test GPS 1.0
        let target = version::Version::new(1, 0);
        let found = closest_revision(Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test GPS 4.0
        let target = version::Version::new(4, 0);
        let found = closest_revision(Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(4, 0)));
        // test GPS 1.1 ==> 1.0
        let target = version::Version::new(1, 1);
        let found = closest_revision(Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test GPS 1.2 ==> 1.0
        let target = version::Version::new(1, 2);
        let found = closest_revision(Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test GPS 1.3 ==> 1.0
        let target = version::Version::new(1, 3);
        let found = closest_revision(Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test GPS 1.4 ==> 1.0
        let target = version::Version::new(1, 4);
        let found = closest_revision(Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test GLO 4.2 ==> 4.0
        let target = version::Version::new(4, 2);
        let found = closest_revision(Constellation::Glonass, target); 
        assert_eq!(found, Some(version::Version::new(4, 0)));
        // test GLO 1.4 ==> 1.0
        let target = version::Version::new(1, 4);
        let found = closest_revision(Constellation::Glonass, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test BDS 1.0 ==> does not exist 
        let target = version::Version::new(1, 0);
        let found = closest_revision(Constellation::BeiDou, target); 
        assert_eq!(found, None); 
        // test BDS 1.4 ==> does not exist 
        let target = version::Version::new(1, 4);
        let found = closest_revision(Constellation::BeiDou, target); 
        assert_eq!(found, None); 
        // test BDS 2.0 ==> does not exist 
        let target = version::Version::new(2, 0);
        let found = closest_revision(Constellation::BeiDou, target); 
        assert_eq!(found, None); 
    }
}
