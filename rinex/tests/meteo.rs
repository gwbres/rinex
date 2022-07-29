#[cfg(test)]
mod test {
    use rinex::*;
    use rinex::epoch;
    #[test]
    fn v2_gode0030_96m() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/MET/V2/gode0030.96m";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_meteo_rinex(), true);
        assert_eq!(rinex.header.obs.is_none(), true);
        assert_eq!(rinex.header.meteo.is_some(), true);
    }
    #[test]
    fn v4_example1() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/MET/V4/example1.txt";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_meteo_rinex(), true);
        assert_eq!(rinex.header.obs.is_none(), true);
        assert_eq!(rinex.header.meteo.is_some(), true);
        let record = rinex.record.as_meteo();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        assert_eq!(record.len(), 5);
        println!("{:#?}", rinex.header);
        println!("{:#?}", rinex.header.meteo);
        println!("{:#?}", record);

        // test epoch content
        for (_, obs) in record.iter() {
            for (obs, data) in obs.iter() {
                if *obs == meteo::observable::Observable::Pressure {
                    assert_eq!(*data, 993.3);
                } else if *obs == meteo::observable::Observable::HumidityRate {
                    assert_eq!(*data, 90.0);
                }
            }
        }
        let epoch = epoch::Epoch {
            date: epoch::str2date("2021 1 7 0 0 0").unwrap(),
            flag: epoch::EpochFlag::default(),
        };
        let e = record.get(&epoch).unwrap();
        for (obs, data) in e.iter() {
            if *obs == meteo::observable::Observable::Temperature {
                assert_eq!(*data, 23.0);
            }
        }
        let epoch = epoch::Epoch {
            date: epoch::str2date("2021 1 7 0 0 30").unwrap(),
            flag: epoch::EpochFlag::default(),
        };
        let e = record.get(&epoch).unwrap();
        for (obs, data) in e.iter() {
            if *obs == meteo::observable::Observable::Temperature {
                assert_eq!(*data, 23.0);
            }
        }
        let epoch = epoch::Epoch {
            date: epoch::str2date("2021 1 7 0 1 0").unwrap(),
            flag: epoch::EpochFlag::default(),
        };
        let e = record.get(&epoch).unwrap();
        for (obs, data) in e.iter() {
            if *obs == meteo::observable::Observable::Temperature {
                assert_eq!(*data, 23.1);
            }
        }
        let epoch = epoch::Epoch {
            date: epoch::str2date("2021 1 7 0 1 30").unwrap(),
            flag: epoch::EpochFlag::default(),
        };
        let e = record.get(&epoch).unwrap();
        for (obs, data) in e.iter() {
            if *obs == meteo::observable::Observable::Temperature {
                assert_eq!(*data, 23.1);
            }
        }
        let epoch = epoch::Epoch {
            date: epoch::str2date("2021 1 7 0 2 0").unwrap(),
            flag: epoch::EpochFlag::default(),
        };
        let e = record.get(&epoch).unwrap();
        for (obs, data) in e.iter() {
            if *obs == meteo::observable::Observable::Temperature {
                assert_eq!(*data, 23.1);
            }
        }
    }
}
