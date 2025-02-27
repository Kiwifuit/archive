use std::ffi::OsStr;
use std::ffi::{CStr, CString};
use std::fmt::Display;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use archive_sys::archive;
use archive_sys::archive_entry;
use log::debug;

const DEFAULT_CHUNK_SIZE: usize = 1024;

pub struct ArchiveReader {
    handle: *mut archive,
    chunk_size: usize,
}

impl ArchiveReader {
    pub fn new<P: AsRef<OsStr>>(filename: P, chunk_size: Option<usize>) -> Option<Self> {
        let handle = unsafe { archive_sys::archive_read_new() };

        unsafe {
            archive_sys::archive_read_support_filter_all(handle);
            archive_sys::archive_read_support_format_all(handle);
        };

        let result = unsafe {
            archive_sys::archive_read_open_filename(
                handle,
                CString::new(filename.as_ref().as_encoded_bytes())
                    .unwrap()
                    .into_raw() as *const i8,
                10240,
            )
        };

        if result != 0 {
            None
        } else {
            let chunk_size = chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE);
            Some(Self { handle, chunk_size })
        }
    }

    pub fn entries(&self) -> ArchiveIterator<'_> {
        ArchiveIterator { archive: self }
    }

    fn get_next_header(&self) -> Option<*mut archive_entry> {
        let mut entry: *mut archive_entry = std::ptr::null_mut();
        let ret = unsafe { archive_sys::archive_read_next_header(self.handle, &mut entry) };

        if ret == archive_sys::ARCHIVE_OK as i32 && !entry.is_null() {
            Some(entry)
        } else {
            None
        }
    }
}

impl Drop for ArchiveReader {
    fn drop(&mut self) {
        let ret = unsafe { archive_sys::archive_read_close(self.handle) };

        if ret != archive_sys::ARCHIVE_OK as i32 {
            panic!(
                "failed to drop archive reader: {}",
                crate::get_error(self.handle, ret)
            );
        }

        let ret = unsafe { archive_sys::archive_free(self.handle) };

        if ret != archive_sys::ARCHIVE_OK as i32 {
            panic!(
                "failed to drop archive reader: {}",
                crate::get_error(self.handle, ret)
            );
        }
    }
}

pub struct ArchiveIterator<'a> {
    archive: &'a ArchiveReader,
}

impl Iterator for ArchiveIterator<'_> {
    type Item = ArchiveEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.archive.get_next_header()?;

        Some(ArchiveEntry {
            entry,
            archive: self.archive.handle,
            chunk_size: self.archive.chunk_size,
        })
    }
}

pub struct ArchiveEntry {
    archive: *mut archive,
    entry: *mut archive_entry,
    chunk_size: usize,
}

impl Display for ArchiveEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ArchiveEntry(filename = {}, size = {})",
            self.path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or(String::from("<unknown>")),
            self.size()
        )
    }
}

impl ArchiveEntry {
    pub fn path(&self) -> Option<PathBuf> {
        let raw_path: *const i8 = unsafe { archive_sys::archive_entry_pathname(self.entry) };

        if !raw_path.is_null() {
            let path = unsafe { CStr::from_ptr(raw_path) }
                .to_string_lossy()
                .to_string();

            Some(PathBuf::from(path))
        } else {
            None
        }
    }

    pub fn size(&self) -> i64 {
        unsafe { archive_sys::archive_entry_size(self.entry) }
    }

    pub fn extract<P: AsRef<std::path::Path>>(&self, base_dir: P) -> std::io::Result<usize> {
        let mut total_read_bytes = 0;
        let total_size = self.size();
        let mut chunk = vec![0; self.chunk_size];
        let out_path = base_dir.as_ref().join(self.path().unwrap());

        if let Some(parent) = out_path.parent() {
            create_dir_all(parent)?;
        }

        debug!("Extracting file `{}`", self.path().unwrap().display());

        let mut out_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(out_path)?;

        while total_read_bytes != total_size {
            let bytes_read = unsafe {
                archive_sys::archive_read_data(
                    self.archive,
                    chunk.as_mut_ptr() as *mut std::ffi::c_void,
                    self.chunk_size,
                )
            };

            total_read_bytes += bytes_read as i64;

            if bytes_read > 0 {
                assert!(!chunk.is_empty(), "chunk is empty when reported otherwise");

                let read_slice = &chunk[..bytes_read as usize];
                out_file.write_all(read_slice)?;
            }
        }

        assert_eq!(
            total_read_bytes, total_size,
            "mismatch between file size in archive and total read data"
        );

        debug!("File `{}` extracted", self.path().unwrap().display());
        Ok(total_read_bytes as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader() {
        let reader = ArchiveReader::new("test.tar", None);

        assert!(reader.is_some());

        for file in reader.unwrap().entries() {
            println!("Found: {}", file);

            file.path();

            assert!(file.path().is_some());
        }
    }
}
