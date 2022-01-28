
pub type BtResult<T> = Result<T, Error>;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Error {
    // Catch-all for non-zero return codes, should be expanded eventually
    Failure(isize),
    // Catch-all for borrows that returned NULL, should be expanded eventually
    ResourceBorrow,
    Memory,
    CtfSourceRequiresInputs,
    CtfSourceMissingOutputPorts,
    ProxySinkMissingInputPort,
    Utf8Error,
    Uuid,
    EnvValue,
    NulError,
}
impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Failure(code) => {
                f.write_str("libbabeltrace returned a non-zero return code (")?;
                code.fmt(f)?;
                f.write_str(")")
            }
            Error::ResourceBorrow => { f.write_str("libbabeltrace returned NULL when attempting to borrow a resource") }
            Error::Memory => { f.write_str("libbabeltrace encountered a memory error") }
            Error::CtfSourceRequiresInputs => { f.write_str("At least one CTF-containing input directory is required") }
            Error::CtfSourceMissingOutputPorts => { f.write_str("At least one CTF output port is required, check that the input path contains at least one stream") }
            Error::ProxySinkMissingInputPort => { f.write_str("At least one sink output port is required") }
            Error::Utf8Error => { f.write_str("Encountered a libbabeltrace string with invalid UTF-8") }
            Error::Uuid => { f.write_str("Encountered an invalid libbabeltrace UUID") }
            Error::EnvValue => { f.write_str("Encountered an invalid trace environment value") }
            Error::NulError => { f.write_str("Invalid string with interior NULL byte") }
        }
    }
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
