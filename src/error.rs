use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("libarchive error: {message} (error {code})")]
    Archive { message: String, code: i32 },

    #[error("Failed to initialize a read or write handle")]
    Initialization,

    #[error("Archive cannot be opened more than once")]
    AlreadyOpen,
}
