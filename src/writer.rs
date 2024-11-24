use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use thiserror::Error;

use crate::core::ArchiveCompression;
use crate::core::ArchiveOptions;
use archive_sys::archive;
use archive_sys::archive_write_new;

type Archive = *mut archive;

#[derive(Debug, Error)]
pub enum ArchiveWriteError {
    #[error("failed to initialize archive")]
    Init,

    #[error("invalid compression")]
    Compression,

    #[error("invalid format")]
    Format,

    #[error("libarchive returned error code {0}: {1}")]
    Archive(i32, String),
}

pub struct ArchiveWriter {
    handle: *mut archive_sys::archive,
}

impl ArchiveWriter {
    pub fn new<P: AsRef<Path>>(
        file: P,
        options: &ArchiveOptions,
    ) -> Result<Self, ArchiveWriteError> {
        let archive = unsafe { archive_write_new() };
        let path_c = CString::new(file.as_ref().as_os_str().as_bytes())
            .expect("expected c_string to be created");

        if archive.is_null() {
            return Err(ArchiveWriteError::Init);
        }

        let code = unsafe {
            archive_sys::archive_write_set_format(archive, options.format.try_into().unwrap())
        };

        handle_error(archive, code)?;
        set_compression(archive, options.compression)?;

        let code = unsafe { archive_sys::archive_write_open_file(archive, path_c.as_ptr()) };
        handle_error(archive, code)?;

        Ok(Self { handle: archive })
    }
}
impl Drop for ArchiveWriter {
    fn drop(&mut self) {
        unsafe { archive_sys::archive_free(self.handle) };
    }
}

fn set_compression(
    handle: Archive,
    compression: ArchiveCompression,
) -> Result<(), ArchiveWriteError> {
    let result = unsafe {
        match compression {
            // archive_sys::ARCHIVE_COMPRESSION_PROGRAM => archive_sys::archive_write_add_filter_program(arg1, cmd)
            // archive_sys::ARCHIVE_COMPRESSION_RPM => archive_sys::archive_write_add_filter_
            archive_sys::ARCHIVE_COMPRESSION_BZIP2 => {
                Ok(archive_sys::archive_write_add_filter_bzip2(handle))
            }
            archive_sys::ARCHIVE_COMPRESSION_COMPRESS => {
                Ok(archive_sys::archive_write_add_filter_compress(handle))
            }
            archive_sys::ARCHIVE_COMPRESSION_GZIP => {
                Ok(archive_sys::archive_write_add_filter_gzip(handle))
            }
            archive_sys::ARCHIVE_COMPRESSION_LRZIP => {
                Ok(archive_sys::archive_write_add_filter_lrzip(handle))
            }
            archive_sys::ARCHIVE_COMPRESSION_LZIP => {
                Ok(archive_sys::archive_write_add_filter_lzip(handle))
            }
            archive_sys::ARCHIVE_COMPRESSION_LZMA => {
                Ok(archive_sys::archive_write_add_filter_lzma(handle))
            }
            archive_sys::ARCHIVE_COMPRESSION_NONE => {
                Ok(archive_sys::archive_write_add_filter_none(handle))
            }
            archive_sys::ARCHIVE_COMPRESSION_UU => {
                Ok(archive_sys::archive_write_add_filter_uuencode(handle))
            }
            archive_sys::ARCHIVE_COMPRESSION_XZ => {
                Ok(archive_sys::archive_write_add_filter_xz(handle))
            }

            _ => Err(ArchiveWriteError::Compression),
        }
    }?;

    handle_error(handle, result)
}

fn handle_error(handle: Archive, result: i32) -> Result<(), ArchiveWriteError> {
    if result != 0 {
        Err(ArchiveWriteError::Archive(
            result,
            crate::get_error(handle, result).to_string(),
        ))
    } else {
        Ok(())
    }
}

// fn set_format(handle: Archive, format: ArchiveFormat) -> Result<(), ArchiveWriteError> {
//     match format {
//         archive_sys::ARCHIVE_FORMAT_7ZIP => archive_sys::archive_write_set_format_7zip,
//         archive_sys::ARCHIVE_FORMAT_AR => archive_sys::archive_write_set_format_ar_svr4,
//         archive_sys::ARCHIVE_FORMAT_AR_BSD => archive_sys::archive_write_set_format_ar_bsd,
//         archive_sys::ARCHIVE_FORMAT_AR_GNU => archive_sys::archive_write_set_format_gnu,
//         archive_sys::ARCHIVE_FORMAT_BASE_MASK => archive_sys::archive_write_set_format_base_mask,
//         archive_sys::ARCHIVE_FORMAT_CAB => archive_sys::archive_write_set_format_cab,
//         archive_sys::ARCHIVE_FORMAT_CPIO => archive_sys::archive_write_set_format_cpio,
//         archive_sys::ARCHIVE_FORMAT_CPIO_AFIO_LARGE => {
//             archive_sys::archive_write_set_format_cpio_afio_large
//         }
//         archive_sys::ARCHIVE_FORMAT_CPIO_BIN_BE => {
//             archive_sys::archive_write_set_format_cpio_bin_be
//         }
//         archive_sys::ARCHIVE_FORMAT_CPIO_BIN_LE => {
//             archive_sys::archive_write_set_format_cpio_bin_le
//         }
//         archive_sys::ARCHIVE_FORMAT_CPIO_POSIX => archive_sys::archive_write_set_format_cpio_posix,
//         archive_sys::ARCHIVE_FORMAT_CPIO_PWB => archive_sys::archive_write_set_format_cpio_pwb,
//         archive_sys::ARCHIVE_FORMAT_CPIO_SVR4_CRC => {
//             archive_sys::archive_write_set_format_cpio_svr4_crc
//         }
//         archive_sys::ARCHIVE_FORMAT_CPIO_SVR4_NOCRC => {
//             archive_sys::archive_write_set_format_cpio_svr4_nocrc
//         }
//         archive_sys::ARCHIVE_FORMAT_EMPTY => archive_sys::archive_write_set_format_empty,
//         archive_sys::ARCHIVE_FORMAT_ISO9660 => archive_sys::archive_write_set_format_iso9660,
//         archive_sys::ARCHIVE_FORMAT_ISO9660_ROCKRIDGE => {
//             archive_sys::archive_write_set_format_iso9660_rockridge
//         }
//         archive_sys::ARCHIVE_FORMAT_LHA => archive_sys::archive_write_set_format_lha,
//         archive_sys::ARCHIVE_FORMAT_MTREE => archive_sys::archive_write_set_format_mtree,
//         archive_sys::ARCHIVE_FORMAT_RAR => archive_sys::archive_write_set_format_rar,
//         archive_sys::ARCHIVE_FORMAT_RAR_V5 => archive_sys::archive_write_set_format_rar_v5,
//         archive_sys::ARCHIVE_FORMAT_RAW => archive_sys::archive_write_set_format_raw,
//         archive_sys::ARCHIVE_FORMAT_SHAR => archive_sys::archive_write_set_format_shar,
//         archive_sys::ARCHIVE_FORMAT_SHAR_BASE => archive_sys::archive_write_set_format_shar_base,
//         archive_sys::ARCHIVE_FORMAT_SHAR_DUMP => archive_sys::archive_write_set_format_shar_dump,
//         archive_sys::ARCHIVE_FORMAT_TAR => archive_sys::archive_write_set_format_tar,
//         archive_sys::ARCHIVE_FORMAT_TAR_GNUTAR => archive_sys::archive_write_set_format_tar_gnutar,
//         archive_sys::ARCHIVE_FORMAT_TAR_PAX_INTERCHANGE => {
//             archive_sys::archive_write_set_format_tar_pax_interchange
//         }
//         archive_sys::ARCHIVE_FORMAT_TAR_PAX_RESTRICTED => {
//             archive_sys::archive_write_set_format_tar_pax_restricted
//         }
//         archive_sys::ARCHIVE_FORMAT_TAR_USTAR => archive_sys::archive_write_set_format_tar_ustar,
//         archive_sys::ARCHIVE_FORMAT_WARC => archive_sys::archive_write_set_format_warc,
//         archive_sys::ARCHIVE_FORMAT_XAR => archive_sys::archive_write_set_format_xar,
//         archive_sys::ARCHIVE_FORMAT_ZIP => archive_sys::archive_write_set_format_zip,
//     }
// }

#[cfg(test)]
mod tests {
    use archive_sys::ARCHIVE_FORMAT_TAR;

    use super::*;
    use crate::core::ArchiveOptions;

    #[test]
    fn archive_works() {
        // Should be format = TAR and compression = NONE
        let options = ArchiveOptions::default().with_format(ARCHIVE_FORMAT_TAR);
        let archive = ArchiveWriter::new("file.tar", &options);

        assert!(archive.is_ok());
    }
}
