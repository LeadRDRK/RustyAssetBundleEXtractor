# runirip [![Build Status]][actions] [![Latest Version]][crates.io] [![Docs]][docs.rs] [![License_MIT]][license_mit] [![License_APACHE]][license_apache] 

[Build Status]: https://img.shields.io/github/actions/workflow/status/LeadRDRK/runirip/ci.yml?branch=main
[actions]: https://github.com/LeadRDRK/runirip/actions?query=branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/runirip
[crates.io]: https://crates.io/crates/runirip
[Docs]: https://docs.rs/runirip/badge.svg
[docs.rs]: https://docs.rs/crate/runirip/
[License_MIT]: https://img.shields.io/badge/License-MIT-yellow.svg
[license_mit]: https://raw.githubusercontent.com/LeadRDRK/runirip/main/LICENSE-MIT
[License_APACHE]: https://img.shields.io/badge/License-Apache%202.0-blue.svg
[license_apache]: https://raw.githubusercontent.com/LeadRDRK/runirip/main/LICENSE-APACHE


runirip is a Rust library for manipulating various Unity asset file formats. It is a fork of [rabex](https://github.com/UniversalGameExtraction/RustyAssetBundleEXtractor).

## Feature flags
All of these features are enabled by default.
- Compression: `lzma`, `lz4`, `brotli`
- Encryption: `unitycn_encryption`

## Examples

### Parsing an asset bundle and dumping its objects

```rust
use std::{
    fs::{DirBuilder, File},
    io::{Seek, Write, BufWriter},
    path::Path,
};

use runirip::files::{BundleFile, SerializedFile};
use runirip::config::ExtractionConfig;

let mut reader = File::open(fp).unwrap();
let export_dir = Path::new("dump");

// parse the bundle file
let config = ExtractionConfig::new();
let mut bundle = BundleFile::from_reader(&mut reader, &config).unwrap();

// iterate over the files in the bundle
for directory in &bundle.m_DirectoryInfo {
    // generate export dir for cab
    let export_cab_dir = export_dir.join(&directory.path);
    // seek to the start of the file in the bundle
    bundle
        .m_BlockReader
        .seek(std::io::SeekFrom::Start(directory.offset as u64))
        .unwrap();

    // try to parse the file as a SerializedFile
    match SerializedFile::from_reader(&mut bundle.m_BlockReader, &config) {
        Ok(serialized) => {
            // iterate over objects
            for object_info in &serialized.m_Objects {
                // get a helper object to parse the object
                let mut reader =
                    serialized.get_object_reader(object_info, &mut bundle.m_BlockReader);

                // try to get the name
                let name = match reader.peek_name() {
                    Ok(name) => format!("{}_{}", object_info.m_PathID, name),
                    Err(_) => format!("{}", object_info.m_PathID),
                };

                // ensure that the parent directory exists
                let dst_path = export_cab_dir.join(name);
                DirBuilder::new()
                    .recursive(true)
                    .create(dst_path.parent().unwrap())
                    .unwrap_or_else(|_| panic!("Failed to create {:?}", dst_path.parent()));

                // read the object
                let object = reader.read().unwrap();

                // export as json
                let json_file = File::create(format!("{}.json", dst_path.to_string_lossy())).unwrap();
                serde_json::to_writer_pretty(&mut BufWriter::new(json_file), &object).unwrap();

                // export as yaml
                let yaml_file = File::create(format!("{}.yaml", dst_path.to_string_lossy())).unwrap();
                serde_yaml::to_writer(&mut BufWriter::new(yaml_file), &object).unwrap();

                // parse the object as msgpack
                let msgpack = rmp_serde::to_vec(&object).unwrap();
                File::create(format!("{}.msgpack", dst_path.to_string_lossy()))
                    .unwrap()
                    .write_all(&msgpack)
                    .unwrap();

                // serialize as actual class
                // note: a small part of the object classes isn't implemented yet
                // TODO: cant actually do this now
                if object.m_ClassID == runirip::objects::map::AssetBundle {
                    let ab = reader
                        .parse::<runirip::objects::classes::AssetBundle>()
                        .unwrap();
                    println!("{:?}", ab);
                }
            }
        }
        Err(e) => {
            // TODO - try to filter out resource files
            println!(
                "Failed to parse {} as SerializedFile.",
                &directory.path.to_string()
            );
        }
    }
}
```

### Reading a UnityCN encrypted asset bundle

```rust"
use runirip::files::BundleFile;

let mut reader = File::open(fp).unwrap();
let config = ExtractionConfig {
    unitycn_key: Some("Decryption Key".as_bytes().try_into().unwrap()),
    fallback_unity_version: "2020.3.0f1".to_owned(),
};
let bundle = BundleFile::from_reader(&mut reader, &config).unwrap();
```

## Notes

### TODO

- Parsers:

  - [x] SerializedFile
  - [x] BundleFile
  - [ ] WebFile

- Object Classes:

  - [x] Generator
  - [x] Parser
  - [ ] Writer
  - [ ] Export Functions

- Tests:

  - [ ] Normal Tests
  - [ ] Artificing Test Files
  - [ ] 100% Coverage

- Other:
  - [x] Feature config

## License

runirip is primarily distributed under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE), [LICENSE-MIT](LICENSE-MIT), and
[COPYRIGHT](COPYRIGHT) for details.
