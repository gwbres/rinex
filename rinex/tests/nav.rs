#[cfg(test)]
mod test {
    use rinex::*;
    #[test]
    fn v2_amel0010_21g() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/NAV/V2/amel0010.21g";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_navigation_rinex(), true);
        assert_eq!(rinex.header.obs.is_none(), true);
        assert_eq!(rinex.header.meteo.is_none(), true);
        /*
        // RECORD
        match data {
        "NAV" => {
            // NAV files checks
            assert_eq!(rinex.header.obs.is_none(), true);
            assert_eq!(rinex.is_navigation_rinex(), true);
            assert_eq!(rinex.header.meteo.is_none(), true);
            let record = rinex.record.as_nav().unwrap();
            println!("----- EPOCHs ----- \n{:#?}", record.keys());
            let mut epochs = record.keys();
            // Testing event description finder
            if let Some(event) = epochs.nth(0) {
                // [!] with dummy t0 = 1st epoch timestamp
                    //     this will actually return `header section` timestamps
                    println!("EVENT @ {:#?} - description: {:#?}", event, rinex.event_description(*event)); 
                }
            },
        }*/
    }
    #[cfg(feature = "with-gzip")]
    use std::str::FromStr;
    #[test]
    #[cfg(feature = "with-gzip")]
    fn v3_brdc00gop_r_2021_gz() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/NAV/V3/BRDC00GOP_R_20210010000_01D_MN.rnx.gz";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_navigation_rinex(), true);
        assert_eq!(rinex.header.obs.is_none(), true);
        assert_eq!(rinex.header.meteo.is_none(), true);
        //assert_eq!(rinex.epochs().len(), 4); 
        let record = rinex.record
            .as_nav();
        assert_eq!(record.is_some(), true);
        let record = record
            .unwrap();
        println!("{:#?}", record);
        let mut index = 0;
        let expected_epochs : Vec<&str> = vec![
            "2021 01 01 00 00 00",
            "2021 01 01 01 28 00",
            "2021 01 01 07 15 00",
            "2021 01 01 08 20 00"];
        let expected_vehicules : Vec<rinex::sv::Sv> = vec![
            rinex::sv::Sv::from_str("C01").unwrap(),
            rinex::sv::Sv::from_str("S36").unwrap(),
            rinex::sv::Sv::from_str("R10").unwrap(),
            rinex::sv::Sv::from_str("E03").unwrap()];
        for (epoch, frames) in record.iter() {
            let expected_e = epoch::Epoch {
                date: epoch::str2date(expected_epochs[index]).unwrap(),
                flag: epoch::EpochFlag::default(),
            };
            if *epoch != expected_e {
                panic!("decoded unexpected epoch {:#?}", epoch)
            }
            /*for (sv, _) in sv.iter() {
                if *sv != expected_vehicules[index] {
                    panic!("decoded unexpected sv {:#?}", sv)
                }
            }*/
            index += 1;
        }
    }
}
