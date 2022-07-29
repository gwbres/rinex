use rinex::sv;
use rinex::epoch;
use rinex::types::Type;
use rinex::constellation::Constellation;
//use itertools::Itertools;

fn main() {
    println!("**************************");
    println!("   (NAV) RINEX example    ");
    println!("**************************");

    // example file
    let navigation_file = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx";
    let rinex = rinex::Rinex::from_file(&navigation_file).unwrap();

    // header information
    assert_eq!(rinex.header.is_crinex(), false);
    assert_eq!(rinex.header.rinex_type, Type::NavigationData);
    assert_eq!(rinex.header.version.major, 3);
    assert_eq!(rinex.header.constellation, Some(Constellation::Mixed)); 
    // leap second field for instance
    // is a major > 3 optionnal field
    assert_eq!(rinex.header.leap.is_some(), true);

    // (NAV) record analysis
    let record = rinex.record
        .as_nav()
        .unwrap();
    //////////////////////////////
    // basic record browsing
    //////////////////////////////
                                // record = HashMap<Epoch <()>>
    for entry in record.iter() { // over all epochs
        let epoch = entry.0;
        println!("Found epoch: `{:#?}`", epoch); 
                                        // epoch = HashMap<Sv, ()>
        for vehicule in entry.1.iter() { // over all sat. vehicules
            let sv = vehicule.0;
            println!("Found sat vehicule: `{:#?}`", sv); 
            let data = vehicule.1;
            println!("Data: `{:#?}`", data); 
        }
    }

    //////////////////////////////////////
    // basic hashmap [indexing] 
    // is a quick method to grab some data
    //////////////////////////////////////

    // match a specific `epoch`
    //  * `epoch` is a chrono::NaiveDateTime alias
    //     therefore one can use any chrono::NaiveDateTime method
    let to_match = epoch::Epoch::new(
        epoch::str2date("21 01 01 00 00 00").unwrap(),
        epoch::EpochFlag::default());
    //    ---> retrieve all data for desired `epoch`
    //         using direct hashmap[indexing]
    let matched = &record[&to_match];
    println!("\n------------- Matching epoch \"{:?}\" ----------\n{:#?}", to_match, matched); 
    
    // ----> zoom in on `E01` vehicule for that particular `epoch` 
    let to_match = sv::Sv::new(Constellation::Galileo, 1);
    //let matched = &matched[&to_match];
    println!("\n------------- Adding Sv filter \"{:?}\" to previous epoch filter ----------\n{:#?}", to_match, matched); 
    // ----> zoom in on `E01` clock drift for that `epoch`
    //let matched = &matched["ClockDrift"];
    println!("\n------------- \"clockDrift\" data from previous set ----------\n{:#?}", matched); 
    
    ///////////////////////////////////////////////////
    // advanced:
    // iterators + filters allow complex
    // pattern matching, data filtering and extraction
    ///////////////////////////////////////////////////
    let record = rinex.record
        .as_nav()
        .unwrap();

    // list all epochs
    let epochs: Vec<_> = record
        .keys()
        .map(|k| k.date)
        .collect();
    println!("\n------------- Epochs ----------\n{:#?}", epochs); 
    
    // extract all data for `R24` vehicule 
    let to_match = sv::Sv::new(Constellation::Glonass, 24);
    /*
    let matched : Vec<_> = record
        .iter()
        .map(|(_epoch, sv)| { // dont care about epoch, sv filter
            sv.iter() // for all sv
                .find(|(&sv, _)| sv == to_match) // match `E04`
        })
        .flatten()
        .collect();
    println!("\n------------- \"{:?}\" data ----------\n{:#?}", to_match, matched); 
    
    // extract `clockbias` & `clockdrift` fields
    // for `R24` vehicule accross entire record
    let matched : Vec<_> = record
        .iter()
        .map(|(_epoch, sv)| {
            sv.iter() // for all sv
                .find(|(&sv, _)| sv == to_match) // match `R24`
                .map(|(_, data)| ( // create a tuple
                    data["ClockBias"]
                        .as_f32()
                        .unwrap(),
                    data["ClockDrift"]
                        .as_f32()
                        .unwrap(),
                ))
        })
        .flatten()
        .collect();
    println!("\n------------- \"{:?}\" (bias,drift)----------\n{:#?}", to_match, matched); 

    // build an array made of (satPosX,satPosY,satPosZ) for `R24` 
    let data : Vec<_> = record
        .iter()
        .map(|(_, sv)| {
            sv.iter()
                .find(|(&sv, _)| sv == to_match)
                .map(|(_, data)| ( // create a tuple
                    data["satPosX"]
                        .as_f32()
                        .unwrap(),
                    data["satPosY"]
                        .as_f32()
                        .unwrap(),
                    data["satPosZ"]
                        .as_f32()
                        .unwrap(),
                ))
        })
        .flatten()
        .collect();
    println!("\n------------- \"{:?}\" (PosX,PosY,PosZ)----------\n{:#?}", to_match, data); 

    */
}
