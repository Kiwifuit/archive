use std::cell::UnsafeCell;
use std::ffi::OsStr;
use std::ffi::{CStr, CString};
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use archive_sys::archive;
use archive_sys::archive_entry;

use bon::Builder;
use log::{debug, error, info, warn};

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
            warn!("Archive already opened a file! Skipping...");
            return Err(crate::error::Error::AlreadyOpen);
        } else if self.handle.is_null() {
            let handle = unsafe { archive_sys::archive_read_new() };

            if handle.is_null() {
                return Err(crate::error::Error::Initialization);
            }

            self.handle = handle;
        }

        self.set_options()?;

        info!("Opening: {:?}", file_path.as_ref().display());
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

        self.open = true;
        Ok(())
    }

    /// Returns an iterator containing the contents of
    /// the archive.
    pub fn entries<'a>(&'a self) -> ArchiveIterator<'a> {
        ArchiveIterator { archive: self }
    }

    /// Closes the file and frees the resources
    /// used by this struct.
    ///
    /// Normally, this function would be called
    /// on `Drop`, so this shouldn't be called
    /// unless this `ArchiveReader` is going
    /// to be re-used.
    ///
    /// Does nothing when the archive isn't
    /// `.open()`
    pub fn close(&mut self) -> Result<()> {
        if !self.open {
            return Ok(());
        }

        let ret = unsafe { archive_sys::archive_read_close(self.handle) };

        if ret != archive_sys::ARCHIVE_OK as i32 {
            return Err(crate::error::Error::Archive {
                message: crate::get_error(self.handle, ret).to_string(),
                code: ret,
            });
        }

        let ret = unsafe { archive_sys::archive_free(self.handle) };

        if ret != archive_sys::ARCHIVE_OK as i32 {
            return Err(crate::error::Error::Archive {
                message: crate::get_error(self.handle, ret).to_string(),
                code: ret,
            });
        }

        self.open = false;
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
        if let Err(e) = self.close() {
            panic!("failed to drop archive reader: {}", e);
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
            path: OnceLock::new(),
            metadata: OnceLock::new(),

            _marker: PhantomData,
        })
    }
}

// pub struct EntryTimeData {
//     pub tv_sec: ::std::os::raw::c_long,
//     pub tv_nsec: ::std::os::raw::c_long,
// }

#[derive(Default)]
pub struct EntryMetadata {
    // st_dev: std::os::raw::c_ulong,
    // st_ino: std::os::raw::c_ulong,
    // st_nlink: std::os::raw::c_ulong,
    st_mode: ::std::os::raw::c_uint,
    // st_uid: ::std::os::raw::c_uint,
    // st_gid: ::std::os::raw::c_uint,
    // __pad0: ::std::os::raw::c_int,
    // st_rdev: std::os::raw::c_ulong,
    st_size: ::std::os::raw::c_long,
    // st_atim: EntryTimeData,
    // st_mtim: EntryTimeData,
    // st_ctim: EntryTimeData,
}

impl EntryMetadata {
    pub fn is_dir(&self) -> bool {
        (self.st_mode & archive_sys::S_IFMT) == archive_sys::S_IFDIR
    }

    pub fn is_file(&self) -> bool {
        (self.st_mode & archive_sys::S_IFMT) == archive_sys::S_IFREG
    }
}

pub struct ArchiveEntry<'a> {
    archive: &'a ArchiveReader,
    entry: *mut archive_entry,

    // Memoized fields:
    path: OnceLock<PathBuf>,
    metadata: OnceLock<EntryMetadata>,

    _marker: PhantomData<&'a archive_entry>,
}

impl AsRef<Path> for ArchiveEntry<'_> {
    fn as_ref(&self) -> &Path {
        self.archive_path()
    }
}

impl ArchiveEntry<'_> {
    /// Fetches the path of this entry within the archive.
    ///
    /// This operation is only expensive on the first call.
    /// Subsequent calls clone the original data stored
    /// after the first fetch
    ///
    /// # Panics
    ///
    /// This function panics if the underlying call to
    /// `archive_entry_pathname(3)` fails
    pub fn archive_path(&self) -> &Path {
        self.path.get_or_init(|| {
            let raw_path: *const i8 = unsafe { archive_sys::archive_entry_pathname(self.entry) };

            if raw_path.is_null() {
                error!("An unknown error occurred whilst trying to retrieve the pathname of an archive entry.");
                panic!("An unknown error occurred whilst trying to retrieve the pathname of an archive entry.");
            }
            let path = unsafe { CStr::from_ptr(raw_path) }
                .to_string_lossy()
                .to_string();

            PathBuf::from(path)
        })
    }

    /// Fetches the size of the entry
    pub fn size(&self) -> i64 {
        self.metadata().st_size
    }

    pub fn metadata(&self) -> &EntryMetadata {
        self.metadata.get_or_init(|| {
            let raw_stat = unsafe { archive_sys::archive_entry_stat(self.entry) };

            if raw_stat.is_null() {
                EntryMetadata::default()
            } else {
                let ent_stat: archive_sys::stat = unsafe { *raw_stat };
                EntryMetadata {
                    st_mode: ent_stat.st_mode,
                    st_size: ent_stat.st_size,
                }
            }
        })
    }

    /// Extracts the current entry onto an optional
    /// `base_dir`. When unset, `base_dir` defaults
    /// to the program's working directory
    pub fn extract<P: AsRef<std::path::Path>>(
        &self,
        base_dir: Option<P>,
    ) -> std::io::Result<usize> {
        let mut total_read_bytes = 0;
        let total_size = self.size();
        let mut chunk = vec![0; self.archive.chunk_size];

        let out_path = if let Some(base_dir) = base_dir {
            base_dir.as_ref().join(self.archive_path())
        } else {
            std::env::current_dir()?.join(self.archive_path())
        };

        if let Some(parent) = out_path.parent() {
            create_dir_all(parent)?;
        }

        debug!("Extracting file `{}`", self.archive_path().display());

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

        debug!("File `{}` extracted", self.archive_path().display());
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
                file.archive_path().display(),
                file.size()
            );

            file.archive_path();
        }
    }
}
