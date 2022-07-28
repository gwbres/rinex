#[cfg(test)]
mod test {
    use rinex::*;
    //use std::str::FromStr;
    //use std::process::Command;
/*
    /// Runs `diff` to determines whether f1 & f2 
    /// are strictly identical or not
    fn diff_is_strictly_identical (f1: &str, f2: &str) -> Result<bool, std::string::FromUtf8Error> {
        let output = Command::new("diff")
            .arg("-q")
            .arg("-Z")
            .arg(f1)
            .arg(f2)
            .output()
            .expect("failed to execute \"diff\"");
        let output = String::from_utf8(output.stdout)?;
        Ok(output.len()==0)
    }
*/
    #[test]
    fn test_parser() {
        let test_resources = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/";
        let test_data = vec![
			//"ATX",
			//"CLK",
			//"CRNX",
			//"IONEX",
			//"MET",
			"NAV",
			//"OBS",
		];
        for data in test_data {
            let data_path = std::path::PathBuf::from(
                test_resources.to_owned() + data
            );
            for revision in std::fs::read_dir(data_path)
                .unwrap() {
                let rev = revision.unwrap();
                let rev_path = rev.path();
                let rev_fullpath = &rev_path.to_str().unwrap(); 
                for entry in std::fs::read_dir(rev_fullpath)
                    .unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    let full_path = &path.to_str().unwrap();
                    let is_hidden = entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .starts_with(".");
                    if is_hidden {
                        continue // not a test resource
                    }
                    let is_generated_file = entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .ends_with("-copy");
                    if is_generated_file {
                        continue // not a test resource
                    }
                    
                    let mut is_gzip_encoded = entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .ends_with(".gz");
                    is_gzip_encoded |= entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .ends_with(".Z");
                    if is_gzip_encoded && !cfg!(feature="with-gzip") {
                        continue // do not run in this build configuration
                    }
                    println!("Parsing file: \"{}\"", full_path);
                    let rinex = Rinex::from_file(full_path);
                    assert_eq!(rinex.is_ok(), true);
                    let rinex = rinex.unwrap();
                    match data {
                        "ATX" => { // ATX record
                            assert_eq!(rinex.header.obs.is_none(), true);
                            assert_eq!(rinex.is_navigation_rinex(), false);
                            assert_eq!(rinex.header.meteo.is_none(), true);
                            assert_eq!(rinex.is_antex_rinex(), true);
                        },
                        "NAV" => {
                            assert_eq!(rinex.header.obs.is_none(), true);
                            assert_eq!(rinex.is_navigation_rinex(), true);
                            assert_eq!(rinex.header.meteo.is_none(), true);
                            assert!(rinex.epochs().len() > 0);
                        },
                        "OBS" => {
                            assert_eq!(rinex.header.obs.is_some(), true);
                            assert_eq!(rinex.is_navigation_rinex(), false);
                            assert_eq!(rinex.header.meteo.is_none(), true);
                            assert_eq!(rinex.is_antex_rinex(), false);
                            assert!(rinex.epochs().len() > 0);
                        },
                        "CRNX" => {
                            assert_eq!(rinex.header.obs.is_some(), true);
                            assert_eq!(rinex.is_observation_rinex(), true);
                            assert_eq!(rinex.header.meteo.is_none(), true);
                            assert!(rinex.epochs().len() > 0);
                        },
                        "MET" => {
                            assert_eq!(rinex.header.obs.is_none(), true);
                            assert_eq!(rinex.is_meteo_rinex(), true);
                            assert_eq!(rinex.header.meteo.is_some(), true);
                            assert_eq!(rinex.header.obs.is_none(), true);
                            assert!(rinex.epochs().len() > 0);
                        },
                        "CLK" => {
                            assert_eq!(rinex.is_clocks_rinex(), true);
                            assert_eq!(rinex.header.meteo.is_none(), true);
                            //assert_eq!(rinex.header.obs.is_none(), true);
                            assert_eq!(rinex.header.clocks.is_some(), true);
                            //assert!(rinex.epochs().len() > 0);
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}
