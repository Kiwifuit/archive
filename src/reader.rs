use std::ffi::CString;
use std::ffi::OsStr;

use archive_sys::archive;

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

    pub fn list(&self) -> impl Iterator<Item = String> {
        let mut entry = std::ptr::null_mut();
        let mut files = vec![];

        while unsafe { archive_sys::archive_read_next_header(self.handle, &mut entry) }
            == archive_sys::ARCHIVE_OK as i32
        {
            if entry.is_null() {
                break;
            }

            let entry_name = unsafe { archive_sys::archive_entry_pathname(entry) };
            if !entry_name.is_null() {
                let name = unsafe { std::ffi::CStr::from_ptr(entry_name) }
                    .to_string_lossy()
                    .into_owned();
                files.push(name);
            }
        }

        files.into_iter()
    }
}

impl Drop for ArchiveReader {
    fn drop(&mut self) {
        let ret = unsafe { archive_sys::archive_read_close(self.handle) } as i32;

        if ret != archive_sys::ARCHIVE_OK as i32 {
            panic!(
                "failed to drop archive reader: {}",
                crate::get_error(self.handle, ret)
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader() {
        let reader = ArchiveReader::new("test.tar");

        assert!(reader.is_some());

        for file in reader.unwrap().list() {
            println!("Found: {}", file);
        }
    }
}
