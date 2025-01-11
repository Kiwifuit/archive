use archive::ArchiveReader;
use log::info;
use std::path::PathBuf;

fn main() {
    colog::init();

    info!("Reading `test.tar`");
    let reader = ArchiveReader::new("test.tar", None);
    let base_dir = PathBuf::from("target/run");

    assert!(reader.is_some());

    for file in reader.unwrap().entries() {
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
