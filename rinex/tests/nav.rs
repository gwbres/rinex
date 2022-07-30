#[cfg(test)]
mod test {
    use rinex::*;
    use rinex::sv::Sv;
    use rinex::constellation::Constellation;
    use rinex::navigation::record::MsgType;
    use rinex::navigation::record::FrameClass;
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
        let record = rinex.record.as_nav();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        assert_eq!(record.len(), 4);
        let expected_epochs : Vec<&str> = vec![
            "2020 12 31 23 45 0",
            "2021 01 01 11 15 0",
            "2021 01 01 11 45 0",
            "2021 01 01 16 15 0",
        ];
        let expected_prns : Vec<u8> = vec![1, 2, 7, 3, 4, 5];
        let mut expected_vehicules : Vec<Sv> = Vec::new();
        for prn in expected_prns.iter() {
            expected_vehicules.push(
                Sv {
                    constellation: Constellation::Glonass,
                    prn: *prn,
                }
            );
        }
        let mut index = 0;
        for (e, classes) in record.iter() {
            let expected_e = epoch::Epoch {
                date: epoch::str2date(expected_epochs[index]).unwrap(),
                flag: epoch::EpochFlag::default(),
            };
            assert_eq!(*e, expected_e);
            for (class, frames) in classes.iter() {
                // only Legacy Ephemeris in V2
                assert_eq!(*class, FrameClass::Ephemeris);
                for frame in frames.iter() {
                    let ephemeris = frame.as_eph();
                    assert_eq!(ephemeris.is_some(), true);
                    let (msgtype, sv, clk_offset, clk_dr, clk_drr, data) = ephemeris.unwrap();
                    assert_eq!(msgtype, MsgType::LNAV); // legacy NAV
                    assert_eq!(expected_vehicules.contains(&sv), true);            
                    if sv.prn == 1 {

                    } else if sv.prn == 2 {

                    } else if sv.prn == 7 {

                    } else if sv.prn == 3 {

                    } else if sv.prn == 4 {

                    } else if sv.prn == 5 {

                    }
                }
            }
            index += 1
        }
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
            assert_eq!(*epoch, expected_e);
            /*for (sv, _) in sv.iter() {
                if *sv != expected_vehicules[index] {
                    panic!("decoded unexpected sv {:#?}", sv)
                }
            }*/
            index += 1;
        }
    }
}
