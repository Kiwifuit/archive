use archive::ArchiveReader;
use log::info;
use std::path::PathBuf;

fn main() {
    colog::init();

    let mut reader = ArchiveReader::builder().build();
    let base_dir = PathBuf::from("target/run");

    reader
        .open("archive.tar.gz")
        .expect("Failed to open archive");

    for file in reader.entries() {
        info!("Found: {:?}", file.archive_path().display(),);

        let extracted = file.extract(Some(&base_dir));

        dbg!(&extracted);
        assert!(extracted.is_ok());

        info!(
            "Extracted {} ({} bytes)",
            file.archive_path().display(),
            extracted.unwrap()
        );
    }
}
