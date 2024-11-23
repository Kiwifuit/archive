// use archive_sys::archive;
// use std::path::Path;
// use std::path::PathBuf;

// Compression algorithms
pub type ArchiveCompression = u32;
// Archive formats
pub type ArchiveFormat = u32;

#[derive(Default)]
pub struct ArchiveOptions {
    pub(crate) compression: u32,
    pub(crate) format: u32,
}

impl ArchiveOptions {
    pub fn with_format(mut self, format: ArchiveFormat) -> Self {
        self.format = format;

        self
    }

    pub fn with_compression(mut self, compression: ArchiveCompression) -> Self {
        self.compression = compression;

        self
    }
}
