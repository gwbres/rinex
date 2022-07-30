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
                    let (msgtype, sv, clk, clk_dr, clk_drr, data) = ephemeris.unwrap();
                    assert_eq!(msgtype, MsgType::LNAV); // legacy NAV
                    assert_eq!(expected_vehicules.contains(&sv), true);            
                    if sv.prn == 1 {
                        assert_eq!(clk, 7.282570004460E-5);
                        assert_eq!(clk_dr, 0.0);
                        assert_eq!(clk_drr, 7.380000000000E+04);
                        let posx = data.get("satPosX").unwrap();
                        assert_eq!(posx.as_f64(), Some(-1.488799804690E+03));
                        let posy = data.get("satPosY").unwrap();
                        assert_eq!(posy.as_f64(), Some( 1.292880712890E+04));
                        let posz = data.get("satPosZ").unwrap();
                        assert_eq!(posz.as_f64(), Some( 2.193169775390E+04));
                        let health = data.get("health").unwrap();
                        assert_eq!(health.as_f64(), Some(0.0));
                        let freq = data.get("freqNum").unwrap();
                        assert_eq!(freq.as_f64(), Some(1.0));
                        let ageop = data.get("ageOp").unwrap();
                        assert_eq!(ageop.as_f64(), Some(0.0));
                    } else if sv.prn == 2 {
                        assert_eq!(clk, 4.610531032090E-04);
                        assert_eq!(clk_dr, 1.818989403550E-12);
                        assert_eq!(clk_drr,  4.245000000000E+04);
                        let posx = data.get("satPosX").unwrap();
                        assert_eq!(posx.as_f64(), Some(-8.955041992190E+03));
                        let posy = data.get("satPosY").unwrap();
                        assert_eq!(posy.as_f64(), Some(-1.834875292970E+04));
                        let posz = data.get("satPosZ").unwrap();
                        assert_eq!(posz.as_f64(), Some( 1.536620703130E+04));
                        let freq = data.get("freqNum").unwrap();
                        assert_eq!(freq.as_f64(), Some(-4.0));
                        let ageop = data.get("ageOp").unwrap();
                        assert_eq!(ageop.as_f64(), Some(0.0));
                    } else if sv.prn == 3 {
                        assert_eq!(clk, 2.838205546140E-05); 
                        assert_eq!(clk_dr, 0.0); 
                        assert_eq!(clk_drr, 4.680000000000E+04);
                        let posx = data.get("satPosX").unwrap();
                        assert_eq!(posx.as_f64(), Some(1.502522949220E+04));
                        let posy = data.get("satPosY").unwrap();
                        assert_eq!(posy.as_f64(), Some(-1.458877050780E+04));
                        let posz = data.get("satPosZ").unwrap();
                        assert_eq!(posz.as_f64(), Some( 1.455863281250E+04));
                        let health = data.get("health").unwrap();
                        assert_eq!(health.as_f64(), Some(0.0));
                        let freq = data.get("freqNum").unwrap();
                        assert_eq!(freq.as_f64(), Some(5.0));
                        let ageop = data.get("ageOp").unwrap();
                        assert_eq!(ageop.as_f64(), Some(0.0));
                    } else if sv.prn == 4 {
                        assert_eq!(clk,  6.817653775220E-05);
                        assert_eq!(clk_dr, 1.818989403550E-12);
                        assert_eq!(clk_drr, 4.680000000000E+04);
                        let posx = data.get("satPosX").unwrap();
                        assert_eq!(posx.as_f64(), Some(-1.688173828130E+03));
                        let posy = data.get("satPosY").unwrap();
                        assert_eq!(posy.as_f64(), Some(-1.107156738280E+04));
                        let posz = data.get("satPosZ").unwrap();
                        assert_eq!(posz.as_f64(), Some( 2.293745361330E+04));
                        let health = data.get("health").unwrap();
                        assert_eq!(health.as_f64(), Some(0.0));
                        let freq = data.get("freqNum").unwrap();
                        assert_eq!(freq.as_f64(), Some(6.0));
                        let ageop = data.get("ageOp").unwrap();
                        assert_eq!(ageop.as_f64(), Some(0.0));
                    } else if sv.prn == 5 {
                        assert_eq!(clk, 6.396882236000E-05);
                        assert_eq!(clk_dr, 9.094947017730E-13);
                        assert_eq!(clk_drr, 8.007000000000E+04); 
                        let posx = data.get("satPosX").unwrap();
                        assert_eq!(posx.as_f64(), Some( -1.754308935550E+04));
                        let posy = data.get("satPosY").unwrap();
                        assert_eq!(posy.as_f64(), Some(-1.481773437500E+03));
                        let posz = data.get("satPosZ").unwrap();
                        assert_eq!(posz.as_f64(), Some(  1.847386083980E+04));
                        let health = data.get("health").unwrap();
                        assert_eq!(health.as_f64(), Some(0.0));
                        let freq = data.get("freqNum").unwrap();
                        assert_eq!(freq.as_f64(), Some(1.0));
                        let ageop = data.get("ageOp").unwrap();
                        assert_eq!(ageop.as_f64(), Some(0.0));
                    } else if sv.prn == 7 {
                        assert_eq!(clk, -4.201009869580E-05);
                        assert_eq!(clk_dr, 0.0);
                        assert_eq!(clk_drr, 2.88E4);
                        let posx = data.get("satPosX").unwrap();
                        assert_eq!(posx.as_f64(), Some( 1.817068505860E+04)); 
                        let posy = data.get("satPosY").unwrap();
                        assert_eq!(posy.as_f64(), Some(1.594814404300E+04));
                        let posz = data.get("satPosZ").unwrap();
                        assert_eq!(posz.as_f64(), Some(8.090271484380E+03));
                        let health = data.get("health").unwrap();
                        assert_eq!(health.as_f64(), Some(0.0));
                        let freq = data.get("freqNum").unwrap();
                        assert_eq!(freq.as_f64(), Some(5.0));
                        let ageop = data.get("ageOp").unwrap();
                        assert_eq!(ageop.as_f64(), Some(0.0));
                    }
                }
            }
            index += 1
        }
    }
    #[test]
    fn v3_amel00nld_r_2021() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_navigation_rinex(), true);
        assert_eq!(rinex.header.obs.is_none(), true);
        assert_eq!(rinex.header.meteo.is_none(), true);
        let record = rinex.record.as_nav();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        assert_eq!(record.len(), 6);
        let expected_epochs : Vec<&str> = vec![
            "2021 01 01 00 00 0",
            "2021 01 01 00 15 0",
            "2021 01 01 05 00 0",
            "2021 01 01 09 45 0",
            "2021 01 01 10 10 0",
            "2021 01 01 15 40 0",
        ];
        let mut index = 0;
        for (e, classes) in record.iter() {
            let expected_e = epoch::Epoch {
                date: epoch::str2date(expected_epochs[index]).unwrap(),
                flag: epoch::EpochFlag::default(),
            };
            assert_eq!(*e, expected_e);
            for (class, frames) in classes.iter() {
                // only Legacy Ephemeris in V3
                assert_eq!(*class, FrameClass::Ephemeris);
                for frame in frames.iter() {
                    let ephemeris = frame.as_eph();
                    assert_eq!(ephemeris.is_some(), true);
                    let (msgtype, sv, clk, clk_dr, clk_drr, data) = ephemeris.unwrap();
                    assert_eq!(msgtype, MsgType::LNAV); // legacy NAV
                    match sv.constellation {
                        Constellation::BeiDou => {
                            match sv.prn {
                                5 => {
                                    assert_eq!(clk, -0.426337239332e-03);
                                    assert_eq!(clk_dr,  -0.752518047875e-10);
                                    assert_eq!(clk_drr, 0.0);
                                    let aode = data.get("aode").unwrap();
                                    assert_eq!(aode.as_f64(), Some(0.100000000000e+01));
                                    let crs = data.get("crs").unwrap();
                                    assert_eq!(crs.as_f64(), Some(0.118906250000e+02));
                                    let m0 = data.get("m0").unwrap();
                                    assert_eq!(m0.as_f64(), Some(-0.255139531119e+01));
                                    let i0 = data.get("i0").unwrap();
                                    assert_eq!(i0.as_f64(), Some(0.607169709798e-01));
                                    let acc = data.get("svAccuracy").unwrap();
                                    assert_eq!(acc.as_f64(), Some(0.200000000000e+01));
                                    let sath1 = data.get("satH1").unwrap();
                                    assert_eq!(sath1.as_f64(), Some(0.0));
                                    let tgd1 = data.get("tgd1b1b3").unwrap();
                                    assert_eq!(tgd1.as_f64(), Some( -0.599999994133e-09));
                                },
                                21 => {
                                    assert_eq!(clk,  -0.775156309828e-03);
                                    assert_eq!(clk_dr, -0.144968481663e-10);
                                    assert_eq!(clk_drr,  0.000000000000e+0);
                                    let aode = data.get("aode").unwrap();
                                    assert_eq!(aode.as_f64(), Some(0.100000000000e+01));
                                    let crs = data.get("crs").unwrap();
                                    assert_eq!(crs.as_f64(), Some(-0.793437500000e+02));
                                    let m0 = data.get("m0").unwrap();
                                    assert_eq!(m0.as_f64(), Some(0.206213212749e+01));
                                    let i0 = data.get("i0").unwrap();
                                    assert_eq!(i0.as_f64(), Some(0.964491154768e+00)); 
                                    let acc = data.get("svAccuracy").unwrap();
                                    assert_eq!(acc.as_f64(), Some(0.200000000000e+01));
                                    let sath1 = data.get("satH1").unwrap();
                                    assert_eq!(sath1.as_f64(), Some(0.0));
                                    let tgd1 = data.get("tgd1b1b3").unwrap();
                                    assert_eq!(tgd1.as_f64(), Some(0.143000002950e-07));
                                },
                                _ => panic!("identified unexpected BDS vehicule \"{}\"", sv.prn)
                            }
                        },
                        Constellation::Glonass => {
                            match sv.prn {
                                19 => {
                                    assert_eq!(clk,  -0.126023776829e-03);
                                    assert_eq!(clk_dr,  -0.909494701773e-12); 
                                    assert_eq!(clk_drr, 0.0);
                                    let pos = data.get("satPosX").unwrap();
                                    assert_eq!(pos.as_f64(), Some(0.783916601562e+04));
                                    let pos = data.get("satPosY").unwrap();
                                    assert_eq!(pos.as_f64(), Some(-0.216949155273e+05));
                                    let pos = data.get("satPosZ").unwrap();
                                    assert_eq!(pos.as_f64(), Some(0.109021518555e+05)); 
                                },
                                7 => {
                                    assert_eq!(clk, -0.420100986958E-04); 
                                    assert_eq!(clk_dr, 0.0); 
                                    assert_eq!(clk_drr, 0.342000000000e+05); 
                                    let pos = data.get("satPosX").unwrap();
                                    assert_eq!(pos.as_f64(), Some(0.124900639648e+05));
                                    let pos = data.get("satPosY").unwrap();
                                    assert_eq!(pos.as_f64(), Some(0.595546582031e+04));
                                    let pos = data.get("satPosZ").unwrap();
                                    assert_eq!(pos.as_f64(), Some(0.214479208984e+05));
                                },
                                _ => panic!("identified unexpected GLO vehicule \"{}\"", sv.prn)
                            }
                        },
                        Constellation::Galileo => {
                            match sv.prn {
                                1 => {
                                    assert_eq!(clk, -0.101553811692e-02);
                                    assert_eq!(clk_dr,  -0.804334376880e-11);
                                    assert_eq!(clk_drr, 0.0);
                                    let iodnav = data.get("iodnav").unwrap();
                                    assert_eq!(iodnav.as_f64(), Some(0.130000000000e+02));
                                    let crs = data.get("crs").unwrap();
                                    assert_eq!(crs.as_f64(), Some( 0.435937500000e+02));
                                    let cis = data.get("cis").unwrap();
                                    assert_eq!(cis.as_f64(), Some(0.409781932831e-07));
                                    let omega_dot = data.get("omegaDot").unwrap();
                                    assert_eq!(omega_dot.as_f64(), Some( -0.518200156545e-08));
                                    let idot = data.get("idot").unwrap();
                                    assert_eq!(idot.as_f64(), Some( -0.595381942905e-09));
                                    let sisa = data.get("sisa").unwrap();
                                    assert_eq!(sisa.as_f64(), Some(0.312000000000e+01));
                                    let bgd =  data.get("bgdE5aE1").unwrap();
                                    assert_eq!(bgd.as_f64(), Some( 0.232830643654e-09));
                                },
                                3 => {
                                    assert_eq!(clk, -0.382520200219e-03);
                                    assert_eq!(clk_dr,  -0.422062385041e-11);
                                    assert_eq!(clk_drr, 0.0);
                                    let iodnav = data.get("iodnav").unwrap();
                                    assert_eq!(iodnav.as_f64(), Some(0.460000000000e+02));
                                    let crs = data.get("crs").unwrap();
                                    assert_eq!(crs.as_f64(), Some( -0.103750000000e+02));
                                    let cis = data.get("cis").unwrap();
                                    assert_eq!(cis.as_f64(), Some(0.745058059692e-08));
                                    let omega_dot = data.get("omegaDot").unwrap();
                                    assert_eq!(omega_dot.as_f64(), Some( -0.539986778331e-08));
                                    let idot = data.get("idot").unwrap();
                                    assert_eq!(idot.as_f64(), Some(0.701814947695e-09));
                                    let sisa = data.get("sisa").unwrap();
                                    assert_eq!(sisa.as_f64(), Some(0.312000000000e+01));
                                    let bgd =  data.get("bgdE5aE1").unwrap();
                                    assert_eq!(bgd.as_f64(), Some( 0.302679836750e-08));
                                },
                                _ => panic!("identified unexpected GAL vehicule \"{}\"", sv.prn)
                            }
                        },
                        _ => panic!("falsely identified \"{}\"", sv.to_string())
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
