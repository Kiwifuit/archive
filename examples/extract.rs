use archive::ArchiveReader;
use std::path::PathBuf;

fn main() {
    let reader = ArchiveReader::new("test.tar");
    let base_dir = PathBuf::from("target/run");

    assert!(reader.is_some());

    for file in reader.unwrap().entries() {
        println!("Found: {}", file);

        let extracted = file.extract(&base_dir, None);

        assert!(extracted.is_ok());

        println!(
            "Extracted {} ({} bytes)",
            file.path().unwrap().display(),
            extracted.unwrap()
        );
    }
}
