Constellation:
* [ ] constellation::Geo ?    
p44   
"is almost identical to Constellation == glonass
but the records contain the satellite position, velocity, accel, health.."

General :
* [ ] add to::file production method
* [ ] simplify line interations with "for line in lines.next()"

Header:
* [ ] header.antenna.model sometimes appear as dirty, check this out
* [ ] coords [m] system ?
* [x] rcvr - clock offset applied
 * [ ] data compensation to do with this?
* [ ] GnssTime + possible conversion needed ?
* [x] interval
* [ ] missing fields: 
 * [ ] WaveLength fact L1/2 ?
 * [ ] Glonass COD/PHS/BIS ?
 * [ ] SYS / PHASE Shift
 * [ ] Glonass carriers related fields
 * [ ] Glonass SLOT /freq channel ?
 * [ ] signal strength informations
 * [ ] First and Last epoch time stamps
 * [ ] Ionospheric compensation, conversions & operations

Record:
* [ ] improve object with a IntoIter trait implementation,
for each record types and enable high level / efficiency interation.
This will allow operations like self.merge(other) at the `Rinex` object level.
* [ ] provide keys() iterator
* [ ] provide values() iterator

Leap + Time modules :
* [ ] leap conversion / apply methods
* [ ] gnss-time conversion method 
* [ ] gnss time related operations

Merge op (teqc):
* [ ] implement Merge()
* [ ] check behav. when parsing a merged file 
* merge_boundaries() : will fail if op is described in the record, not in header

Slice op (teqc):
* ?

Epochs:
* [ ] replace custom macro by datetime::from_str() 
* [ ] replace custom macro by datetime::format!
* [ ] epoch flag mask operation & special bitmask operations
 * [ ] --> abnormal epochs masking 

Observation Data:
* [ ] Carrier code ->to measurement code conversion method
* [ ] rescale raw phase data exceeding F14.3 format by +/- 10E9 accordingly
* [ ] SYS PHASE Shift ?

Ci:
* [ ] OBS: if rcvr clock offsets applied: check epochs do have this field
* [ ] move src/lib.rs tests to dedicated tests/ folder 

Hatanaka:
* [ ] Provide compress() [RNX2CRX operation]
* [ ] Create Header::CRINEX infos accordingly
* [ ] find some CRINEX with special epoch events (flag>2) and test them
* [ ] CRINEX 1|3 special epoch content (flag>2)
will be mishandled / corrupted if they are not only made of COMMENTS
* [ ] decompress(): improve f32/f64 formatting to match standard 

Meteo Data:
* [ ] parse METEO V > 3
* [ ] Sensor Geo position (x,y,z)

Clocks Data:
* [ ] V1, V2 
* [ ] V3
* [ ] V4
