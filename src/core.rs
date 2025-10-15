use bon::Builder;

#[derive(Builder, Default)]
pub struct ArchiveOptions {
    #[builder(default)]
    pub(crate) filter: ArchiveFilter,

    #[builder(default)]
    pub(crate) format: ArchiveFormat,

    #[builder(default = 10240)]
    pub(crate) handle_block_size: usize,
}

#[repr(u32)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ArchiveFilter {
    #[default]
    Auto = u32::MAX,
    None = archive_sys::ARCHIVE_FILTER_NONE,
    Gzip = archive_sys::ARCHIVE_FILTER_GZIP,
    Bzip2 = archive_sys::ARCHIVE_FILTER_BZIP2,
    Compress = archive_sys::ARCHIVE_FILTER_COMPRESS,
    Program = archive_sys::ARCHIVE_FILTER_PROGRAM,
    Lzma = archive_sys::ARCHIVE_FILTER_LZMA,
    Xz = archive_sys::ARCHIVE_FILTER_XZ,
    Uu = archive_sys::ARCHIVE_FILTER_UU,
    Rpm = archive_sys::ARCHIVE_FILTER_RPM,
    Lzip = archive_sys::ARCHIVE_FILTER_LZIP,
    Lrzip = archive_sys::ARCHIVE_FILTER_LRZIP,
    Lzop = archive_sys::ARCHIVE_FILTER_LZOP,
    Grzip = archive_sys::ARCHIVE_FILTER_GRZIP,
    Lz4 = archive_sys::ARCHIVE_FILTER_LZ4,
    Zstd = archive_sys::ARCHIVE_FILTER_ZSTD,
}

#[repr(u32)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ArchiveFormat {
    #[default]
    Auto = u32::MAX,
    Cpio = archive_sys::ARCHIVE_FORMAT_CPIO,
    CpioPosix = archive_sys::ARCHIVE_FORMAT_CPIO_POSIX,
    CpioBinLe = archive_sys::ARCHIVE_FORMAT_CPIO_BIN_LE,
    CpioBinBe = archive_sys::ARCHIVE_FORMAT_CPIO_BIN_BE,
    CpioSvr4Nocrc = archive_sys::ARCHIVE_FORMAT_CPIO_SVR4_NOCRC,
    CpioSvr4Crc = archive_sys::ARCHIVE_FORMAT_CPIO_SVR4_CRC,
    CpioAfioLarge = archive_sys::ARCHIVE_FORMAT_CPIO_AFIO_LARGE,
    CpioPwb = archive_sys::ARCHIVE_FORMAT_CPIO_PWB,
    Shar = archive_sys::ARCHIVE_FORMAT_SHAR,
    SharBase = archive_sys::ARCHIVE_FORMAT_SHAR_BASE,
    SharDump = archive_sys::ARCHIVE_FORMAT_SHAR_DUMP,
    Tar = archive_sys::ARCHIVE_FORMAT_TAR,
    TarUstar = archive_sys::ARCHIVE_FORMAT_TAR_USTAR,
    TarPaxInterchange = archive_sys::ARCHIVE_FORMAT_TAR_PAX_INTERCHANGE,
    TarPaxRestricted = archive_sys::ARCHIVE_FORMAT_TAR_PAX_RESTRICTED,
    TarGnutar = archive_sys::ARCHIVE_FORMAT_TAR_GNUTAR,
    Iso9660 = archive_sys::ARCHIVE_FORMAT_ISO9660,
    Iso9660Rockridge = archive_sys::ARCHIVE_FORMAT_ISO9660_ROCKRIDGE,
    Zip = archive_sys::ARCHIVE_FORMAT_ZIP,
    Empty = archive_sys::ARCHIVE_FORMAT_EMPTY,
    Ar = archive_sys::ARCHIVE_FORMAT_AR,
    ArGnu = archive_sys::ARCHIVE_FORMAT_AR_GNU,
    ArBsd = archive_sys::ARCHIVE_FORMAT_AR_BSD,
    Mtree = archive_sys::ARCHIVE_FORMAT_MTREE,
    Raw = archive_sys::ARCHIVE_FORMAT_RAW,
    Xar = archive_sys::ARCHIVE_FORMAT_XAR,
    Lha = archive_sys::ARCHIVE_FORMAT_LHA,
    Cab = archive_sys::ARCHIVE_FORMAT_CAB,
    Rar = archive_sys::ARCHIVE_FORMAT_RAR,
    P7zip = archive_sys::ARCHIVE_FORMAT_7ZIP,
    Warc = archive_sys::ARCHIVE_FORMAT_WARC,
    RarV5 = archive_sys::ARCHIVE_FORMAT_RAR_V5,
}
