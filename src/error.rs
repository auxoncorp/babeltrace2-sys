pub type BtResult<T> = Result<T, Error>;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, err_derive::Error)]
pub enum Error {
    // Catch-all for non-zero return codes, should be expanded eventually
    #[error(display = "libbabeltrace returned a non-zero return code ({})", _0)]
    Failure(isize),

    // Catch-all for borrows that returned NULL, should be expanded eventually
    #[error(display = "libbabeltrace returned NULL when attempting to borrow a resource")]
    ResourceBorrow,

    #[error(display = "libbabeltrace encountered a memory error")]
    Memory,

    #[error(display = "At least one CTF-containing input directory is required")]
    CtfSourceRequiresInputs,

    #[error(
        display = "At least one CTF output port is required, check that the input path contains at least one stream"
    )]
    CtfSourceMissingOutputPorts,

    #[error(display = "At least one sink output port is required")]
    ProxySinkMissingInputPort,

    #[error(display = "Encountered a libbabeltrace string with invalid UTF-8")]
    Utf8Error,

    #[error(display = "Encountered an invalid libbabeltrace UUID")]
    Uuid,

    #[error(display = "Encountered an invalid trace environment value")]
    EnvValue,

    #[error(display = "Invalid string with interior NULL byte")]
    NulError,
}

impl From<std::str::Utf8Error> for Error {
    fn from(_: std::str::Utf8Error) -> Self {
        Error::Utf8Error
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(_: std::ffi::NulError) -> Self {
        Error::NulError
    }
}

pub trait BtResultExt {
    fn capi_result(self) -> BtResult<()>;
}

impl BtResultExt for std::os::raw::c_int {
    fn capi_result(self) -> BtResult<()> {
        if self == 0 {
            Ok(())
        } else {
            Err(Error::Failure(self as _))
        }
    }
}
