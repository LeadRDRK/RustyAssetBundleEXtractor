use std::{
    fs::{DirBuilder, File},
    io::{Seek, Write, BufWriter},
    path::Path,
};

use runirip::{files::{BundleFile, SerializedFile}, objects::classes::AssetBundle};
use runirip::config::ExtractionConfig;

fn main() {
    let path = std::env::args().skip(1).next().unwrap();
    let mut reader = File::open(path).unwrap();
    let export_dir = Path::new("dump");

    // parse the bundle file
    let config = ExtractionConfig::default();
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
                    let mut reader =
                        serialized.get_object_reader(object_info, &mut bundle.m_BlockReader);

                    // read the object
                    let object = reader.read().unwrap();
                    let name = object.class()
                        .unwrap()
                        .get("m_Name")
                        .map(|v| v.string())
                        .flatten()
                        .map(|name| format!("{}_{}", object_info.m_PathID, name))
                        .unwrap_or_else(|| format!("{}", object_info.m_PathID));
                    println!("{}", name);

                    // ensure that the parent directory exists
                    let dst_path = export_cab_dir.join(name);
                    DirBuilder::new()
                        .recursive(true)
                        .create(dst_path.parent().unwrap())
                        .unwrap_or_else(|_| panic!("Failed to create {:?}", dst_path.parent()));

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
                    if object_info.m_ClassID == runirip::class_ids::AssetBundle {
                        let ab: AssetBundle = object.parse().unwrap();
                        println!("{:?}", ab);
                    }
                }
            }
            Err(_) => {
                // TODO - try to filter out resource files
                println!(
                    "Failed to parse {} as SerializedFile.",
                    &directory.path.to_string()
                );
            }
        }
    }
}