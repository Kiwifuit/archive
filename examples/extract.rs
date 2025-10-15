use archive::ArchiveReader;
use log::info;
use std::path::PathBuf;

fn main() {
    colog::init();

    info!("Reading `test.tar`");
    let reader = ArchiveReader::builder().build();
    let base_dir = PathBuf::from("target/run");

    reader.open("test.tar").expect("Failed to open archive");

    for file in reader.entries() {
        info!("Found: {}", file);

        let extracted = file.extract(&base_dir);

        assert!(extracted.is_ok());

        println!(
            "Extracted {} ({} bytes)",
            file.path().unwrap().display(),
            extracted.unwrap()
        );
    }
}
