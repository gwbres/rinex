RINEX 
=====

[![Rust](https://github.com/gwbres/rinex/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/rust.yml)
[![Audit](https://github.com/gwbres/rinex/actions/workflows/Audit.yml/badge.svg)](https://github.com/gwbres/rinex/actions/workflows/Audit.yml)
[![crates.io](https://docs.rs/rinex/badge.svg)](https://docs.rs/rinex/badge.svg)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/rinex/blob/main/LICENSE-MIT) 


Rust tool suites to parse, analyze, manipulate `RINEX` files

* [`rinex`](rinex/) files parser, this is the core library
* [`crx2rnx`](crx2rnx/) is a command line application to decompress CRINEX files
* [`sinex`](sinex/) files parser, core library

* [`rinex-cli`](rinex-cli/) is a command line application that intends to expose
all the core libraries capacities to the end user, in an easy to use and efficient fashion.
It can be used to analyze files or perform operations like `teqc`. See the **CLI**
table down below for more information.

* [`ublox-rnx`](ublox-rnx) is an application (CLI) that connects to a `Ublox`
receiver and generates RINEX data quickly & easily.
It is made possible by combining the [ublox](https://github.com/lkolbly/ublox) crate
and the [rinex](rinex/) crate.

* `rnx2crx`: RINEX file compressor is currently under development

## Supported `RINEX` types

| `types::Type`            | Trusted           | Untrusted          | CLI                    | UBX                  | Production    |          Notes          |
|--------------------------|-------------------|--------------------|------------------------|----------------------|---------------|-------------------------
| `NavigationData` (NAV)   | V2, V3            |   V4               |  :heavy_check_mark:    | :construction:       |:construction: | `epoch` iteration |
| `ObservationData` (OBS)  | :heavy_check_mark:| :heavy_minus_sign: |  :heavy_check_mark:    | :construction:       |:construction: | `epoch` iteration |
| `CRINEX` (Compressed OBS)| :heavy_check_mark:| :heavy_minus_sign: |  :heavy_check_mark:    | :construction:       |:construction: | `epoch` iteration |
| `MeteoData` (MET)        | :heavy_check_mark:| :heavy_minus_sign: |  :heavy_check_mark:    | :heavy_minus_sign:   |:heavy_check_mark: | `epoch` iteration |  
| `ClocksData` (CLK)       | V3                |   V4               |  :construction:        | :question:           |:construction: | `epoch` iteration |
| `AntennaData` (ATX)      | :heavy_check_mark:| :heavy_minus_sign: |  :construction:        | :heavy_minus_sign:   |:construction: | `ATX` records are not indexed by `epochs` |
| `IonosphereMaps` (IONEX) | :construction:    |                    |  :construction:        | :question:           |:construction: | `epoch` iteration |
| `SINEX` (SNX)            | :construction:    |                    |  :construction:        | :heavy_minus_sign:   |:construction: |   `SINEX` are special `RINEX`, they are managed by a dedicated [core library](sinex/) |
| `Troposphere` (TRO)      | :construction:    |                    |  :construction:        | :question:           |:construction: |   `Troposphere` are one possible SINEX declination |
| `Bias` (BIA)             | :heavy_check_mark:| :heavy_minus_sign: |  :construction:        | :question:           |:construction: |   `Bias` solutions are one possible SINEX declination |

**Production** means file generation (_to_file()_) of `trusted` revisions  
**CLI** means exposed to [`rinex-cli`](rinex-cli/) for easy parsing & quick analysis  
**UBX** means exposed to [`ublox-rnx`](ublox-rnx/) for to produce data with a UBLOX receiver  

:heavy_check_mark: supported   
:heavy_minus_sign: not applicable   
:construction: under development  

## Supported file format / compressions

| Format   | File name restrictions  |    Support          |
|----------|-------------------------|---------------------|
| CRINEX   | :heavy_minus_sign: | :heavy_check_mark:  | 
| Others   | :heavy_minus_sign: | Refer to first table |
| CRINEX + `gzip` | Must end with `.gz` | Compile with `--with-gzip` or uncompress yourself |
| Others + `gzip` | Must end with `.gz` | Refer to first table, compile with `--with-gzip` or uncompress yourself |
| CRINEX + `zlib` | Must end with `.Z` | :construction:  |
| Others + `zlib` | Must end with `.Z` | :construction:  |

:heavy_minus_sign: no restrictions. We can parse a  CRINEX or a IONEX named foo.txt as long as it follows the standards.      
:heavy_check_mark: natively supported   
:construction: under development  

## Record (high level) operations

| Rinex::method          | Status            | 
|------------------------|-------------------|
| `decimate_by_interval` | :heavy_check_mark:|
| `decimate_by_ratio`    | :heavy_check_mark:|
| `data_gaps`            ||
| `lli_mask_filter`      ||
| `epoch_ok_filter`      | :heavy_check_mark:|
| `epoch_nok_filter`     | :heavy_check_mark:|
| `epoch_anomalies`      ||
| `constellation_filter` |:heavy_check_mark:|
| `space_vehicule_filter` |:heavy_check_mark:|

## `teqc` special operations

|Rinex::method | Status          | 
|--------------|-----------------|
| `Merge`      | :construction:   |
| `Splice`     | :construction:  | 



## Features

* `--with-serde`   
enables `Serialization` and `Deserialization` of key RINEX structures

<img align="right" width="400" src="https://upload.wikimedia.org/wikipedia/commons/4/46/SBAS_Service_Areas.png">

* `--with-geo`   
includes the `rust::geo` crate, 
and unlocks the    
`augmentation::sbas_selection_helper()` method,
to select the most appropriate `SBAS` augmentation system for
a given (usually current..) location on Earth.
See [constellation](doc/constellation.md) for example of use.

* `--with-gzip`  
allow native parsing of .gz compressed RINEX files. Otherwise, user must uncompress manualy the `.gz` extension first.

## Contributions

Contributions, raw data and tests methods are welcomed.  
There is still a lot to achieve with this lib, especially regarding the command line applications (high level usage of the library cores).

### Introducing new RINEX types

Follow the existing architecture:

* introduce `types::Type::foo`
* provide new `record::Record` declination
* create `rinex/src/foo` sub directory and provide at least a rinex/src/foo/record.rs for the file body
* add new specific header fields if need be, define them in `rinex/src/foo`
* attach unit tests to the new `rinex/src/foo` structures & methods
* provide relevant (but truncated, to keep repo size reasonnable) raw data, under `test_resources/`
* add new type to `test_resources` testbench in `tests/parser.rs`
* add a focused testbench, in `tests/foo.rs` with specific fields test

### Adding more RINEX data

* only introduce non existing RINEX declinations
* truncate huge files to maintain a reasonnable repo size 

### TODO 

- [ ] Something to do with EpochFlag::HeaderInformationFollows flag
and smart header update ?
- [ ] Cycle Slip Events ?
