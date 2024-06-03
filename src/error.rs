use thiserror::Error;

pub type BtResult<T> = Result<T, Error>;

#[derive(Clone, Eq, PartialEq, Debug, Error)]
pub enum Error {
    // Catch-all for non-zero return codes, should be expanded eventually
    #[error("libbabeltrace returned a non-zero return code ({0})")]
    Failure(isize),
    // Catch-all for borrows that returned NULL, should be expanded eventually
    #[error("libbabeltrace returned NULL when attempting to borrow a resource")]
    ResourceBorrow,
    #[error("libbabeltrace encountered a memory error")]
    Memory,
    #[error("At least one CTF-containing input directory is required")]
    CtfSourceRequiresInputs,
    #[error("At least one CTF output port is required, check that the input path contains at least one stream")]
    CtfSourceMissingOutputPorts,
    #[error("At least one sink input port is required")]
    ProxySinkMissingInputPort,
    #[error("Encountered a libbabeltrace string with invalid UTF-8")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("Encountered an invalid libbabeltrace UUID")]
    Uuid,
    #[error("Encountered an invalid trace environment value")]
    EnvValue,
    #[error("Invalid string with interior NULL byte")]
    NulError(#[from] std::ffi::NulError),
    #[error("The metadata file path '{0}' doesn't exist")]
    NonExistentMetadataPath(String),
    #[error("The metadata file path '{0}' isn't a file")]
    MetadataPathNotFile(String),
    #[error("Encountered an error opening the metadata file '{0}'")]
    MetadataFileOpen(String),
    #[error("libbabeltrace returned NULL when attempting to create a CTF metadata encoder")]
    CtfMetadataDecoderCreate,
    #[error("CTF metadata encoder returned an error ({0}) when decoding the metadata file")]
    CtfMetadataDecoderStatus(isize),
    #[error("libbabeltrace returned NULL when attempting to create a CTF message iterator")]
    CtfMessageIterCreate,
    #[error("Error in user plugin. {0}")]
    PluginError(String),
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
