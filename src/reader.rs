use std::ffi::OsStr;
use std::ffi::{CStr, CString};
use std::fmt::Display;
use std::path::{Path, PathBuf};

use archive_sys::archive;
use archive_sys::archive_entry;

use crate::get_error;

pub struct ArchiveReader {
    handle: *mut archive,
}

impl ArchiveReader {
    pub fn new<P: AsRef<OsStr>>(filename: P) -> Option<Self> {
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
            Some(Self { handle })
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

pub struct ArchiveEntry {
    entry: *mut archive_entry,
}

impl Display for ArchiveEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ArchiveEntry(filename = {})",
            self.path()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or(String::from("<unknown>"))
        )
    }
}

impl ArchiveEntry {
    pub fn path(&self) -> Option<PathBuf> {
        let raw_path = unsafe { archive_sys::archive_entry_pathname(self.entry) };

        if !raw_path.is_null() {
            let path = unsafe { CStr::from_ptr(raw_path) }
                .to_string_lossy()
                .to_string();

            Some(PathBuf::from(path))
        } else {
            None
        }
    }
}

pub struct ArchiveIterator<'a> {
    archive: &'a ArchiveReader,
}

impl<'a> Iterator for ArchiveIterator<'a> {
    type Item = ArchiveEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.archive.get_next_header()?;

        Some(ArchiveEntry { entry })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader() {
        let reader = ArchiveReader::new("test.tar");

        assert!(reader.is_some());

        for file in reader.unwrap().entries() {
            println!("Found: {}", file);

            file.path();

            assert!(file.path().is_some());
        }
    }
}
