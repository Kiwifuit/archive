use archive::ArchiveReader;
use log::info;
use std::path::PathBuf;

fn main() {
    let mut logger = colog::default_builder();

    logger.filter(Some("archive"), log::LevelFilter::Debug);
    logger.init();

    let mut reader = ArchiveReader::builder().path("archive.tar.gz").build();
    let base_dir = PathBuf::from("target/run");

    reader.open().unwrap();

    info!("Hello!");

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
