//! `archive`: Safe Rust bindings to `libarchive`

pub mod core;
pub mod reader;
pub mod writer;

pub use core::ArchiveOptions;
use std::io;
use std::path::Ancestors;
pub use writer::ArchiveWriter;

use std::borrow::Cow;
use std::ffi::CStr;

fn get_error<'a>(handle: *mut archive_sys::archive, result: i32) -> Cow<'a, str> {
    if result == 0 {
        return Cow::from("");
    }

    let e_ptr = unsafe { archive_sys::archive_error_string(handle) };

    if e_ptr.is_null() {
        return Cow::from("unknown error");
    }

    let err = unsafe { CStr::from_ptr(e_ptr) };

    err.to_string_lossy()
}
