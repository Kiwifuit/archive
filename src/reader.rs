use std::ffi::OsStr;
use std::ffi::{CStr, CString};
use std::fmt::Display;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::marker::PhantomData;
use std::path::PathBuf;

use archive_sys::archive;
use archive_sys::archive_entry;

use bon::Builder;
use log::debug;

use crate::core::{ArchiveFilter, ArchiveFormat};
use crate::error::Result;
use crate::ArchiveOptions;

#[derive(Builder)]
pub struct ArchiveReader {
    #[builder(skip = false)]
    open: bool,

    #[builder(skip = std::ptr::null_mut())]
    handle: *mut archive,

    #[builder(default = 1024)]
    /// The size of the buffer to be used when
    /// extracting files. Bigger buffers mean
    /// less system calls (especially for large
    /// files) at the cost of memory consumption.
    ///
    /// This value is used in the Rust library
    /// when extracting files from the archive,
    /// and is set to 1KiB by default.
    chunk_size: usize,

    #[builder(default)]
    /// Set of options to be passed for the
    /// handle. Refer to [`crate::core::ArchiveOptions`]
    /// for more information
    handle_opts: ArchiveOptions,

    #[builder(skip)]
    _marker: PhantomData<*mut UnsafeCell<archive>>,
}

impl ArchiveReader {
/// Opens `file_path` and marks the current instance as
    /// having opened a file. This means that even after closing
    pub fn open<P: AsRef<OsStr>>(&mut self, file_path: P) -> Result<()> {
        if self.open {
            return Err(crate::error::Error::AlreadyOpen);
        } else if self.handle.is_null() {
            let handle = unsafe { archive_sys::archive_read_new() };

            if handle.is_null() {
                return Err(crate::error::Error::Initialization);
            }

            self.handle = handle;
        }

        self.set_options()?;

        let open_result = unsafe {
            archive_sys::archive_read_open_filename(
                self.handle,
                CString::new(file_path.as_ref().as_encoded_bytes())
                    .unwrap()
                    .into_raw() as *const i8,
                self.handle_opts.handle_block_size,
            )
        };

        if open_result != archive_sys::ARCHIVE_OK as i32 {
            return Err(crate::error::Error::Archive {
                message: crate::get_error(self.handle, open_result).to_string(),
                code: open_result,
            });
        }

        Ok(())
    }

    fn set_options(&self) -> Result<()> {
        let filter_result = if self.handle_opts.filter == ArchiveFilter::Auto {
            unsafe { archive_sys::archive_read_support_filter_all(self.handle) }
        } else {
            unsafe {
                archive_sys::archive_read_support_filter_by_code(
                    self.handle,
                    self.handle_opts.filter as i32,
                )
            }
        };

        if filter_result != archive_sys::ARCHIVE_OK as i32 {
            return Err(crate::error::Error::Archive {
                message: crate::get_error(self.handle, filter_result).to_string(),
                code: filter_result,
            });
        }

        let format_result = if self.handle_opts.format == ArchiveFormat::Auto {
            unsafe { archive_sys::archive_read_support_format_all(self.handle) }
        } else {
            unsafe {
                archive_sys::archive_read_support_format_by_code(
                    self.handle,
                    self.handle_opts.format as i32,
                )
            }
        };

        if format_result != archive_sys::ARCHIVE_OK as i32 {
            return Err(crate::error::Error::Archive {
                message: crate::get_error(self.handle, format_result).to_string(),
                code: format_result,
            });
        }

        Ok(())
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

impl<'a> Iterator for ArchiveIterator<'a> {
    type Item = ArchiveEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.archive.get_next_header()?;

        Some(ArchiveEntry {
            entry,
            archive: self.archive,
            _marker: PhantomData,
        })
    }
}

pub struct ArchiveEntry<'a> {
    archive: &'a ArchiveReader,
    entry: *mut archive_entry,
    
    _marker: PhantomData<&'a archive_entry>,
}

impl ArchiveEntry<'_> {
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
        let mut chunk = vec![0; self.archive.chunk_size];
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
                    self.archive.handle,
                    chunk.as_mut_ptr() as *mut std::ffi::c_void,
                    self.archive.chunk_size,
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
        let options = ArchiveOptions::builder()
            .filter(ArchiveFilter::Gzip)
            .format(ArchiveFormat::Tar)
            .build();

        let mut reader = ArchiveReader::builder().handle_opts(options).build();

        let result = reader.open("archive.tar.gz");
        dbg!(&result);
        assert!(result.is_ok());

        for file in reader.entries() {
            println!(
                "Found\t: {} ({} bytes)",
                file.path().unwrap().display(),
                file.size()
            );

            file.path();

            assert!(file.path().is_some());
        }
    }
}
