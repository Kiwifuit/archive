use std::cell::UnsafeCell;
use std::ffi::CString;
use std::marker::PhantomData;
use std::path::PathBuf;

use bon::Builder;

use archive_sys::archive;
use log::warn;

use crate::core::ArchiveFilter;
use crate::error::Result;
use crate::ArchiveOptions;

#[derive(Builder)]
pub struct ArchiveWriter {
    #[builder(skip = std::ptr::null_mut())]
    handle: *mut archive,

    #[builder(into)]
    path: PathBuf,

    #[builder(default)]
    handle_opts: ArchiveOptions,

    #[builder(skip)]
    _marker: PhantomData<UnsafeCell<archive>>,
}

impl ArchiveWriter {
    pub fn open(&mut self) -> Result<()> {
        if !self.handle.is_null() {
            warn!("Attempting to re-open an already opened archive!");
            return Err(crate::error::Error::AlreadyOpen);
        }

        let handle = unsafe { archive_sys::archive_write_new() };
        if handle.is_null() {
            return Err(crate::error::Error::Initialization);
        }

        self.handle = handle;
        self.set_options()?;
        self.open_file()?;

        Ok(())
    }

    fn open_file(&mut self) -> Result<()> {
        let open_result = unsafe {
            let filename = CString::new(self.path.as_mut_os_string().as_encoded_bytes())?.into_raw()
                as *const i8;

            archive_sys::archive_write_open_filename(self.handle, filename)
        };

        if open_result != archive_sys::ARCHIVE_OK as i32 {
            return Err(crate::error::Error::Archive {
                message: crate::get_error(self.handle, open_result).to_string(),
                code: open_result,
            });
        }

        Ok(())
    }

    fn set_options(&mut self) -> Result<()> {
        let filter_result = unsafe {
            archive_sys::archive_write_add_filter(self.handle, self.handle_opts.filter as i32)
        };

        if filter_result != archive_sys::ARCHIVE_OK as i32 {
            return Err(crate::error::Error::Archive {
                message: crate::get_error(self.handle, filter_result).to_string(),
                code: filter_result,
            });
        }

        let format_result = unsafe {
            archive_sys::archive_write_set_format(self.handle, self.handle_opts.format as i32)
        };

        if format_result != archive_sys::ARCHIVE_OK as i32 {
            return Err(crate::error::Error::Archive {
                message: crate::get_error(self.handle, format_result).to_string(),
                code: format_result,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ArchiveFilter, ArchiveFormat};

    #[test]
    fn archive() {
        let opts = ArchiveOptions::builder()
            .filter(ArchiveFilter::Gzip)
            .format(ArchiveFormat::Tar)
            .build();

        let _writer = ArchiveWriter::builder()
            .handle_opts(opts)
            .path("very-cool-archive.tar.gz")
            .build();
    }
}
