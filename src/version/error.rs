use std::io;
use failure::{Error as FatalError, err_msg};
use std::path::Path;


/// Versioning error
#[derive(Debug, Fail)]
pub enum Error {
    /// Hash sum can't be calculated because some files need to be
    /// generated during build
    #[fail(display="dependencies are not ready")]
    New,
    #[fail(display="{}", _0)]
    Fatal(FatalError),
}

impl Error {
    pub fn io<P: AsRef<Path>>(e: io::Error, path: P) -> Error {
        return Error::Fatal(format_err!("{:?}: {}", path.as_ref(), e));
    }
}

impl From<FatalError> for Error {
    fn from(e: FatalError) -> Error {
        Error::Fatal(e)
    }
}

#[cfg(feature="containers")]
impl From<::git2::Error> for Error {
    fn from(e: ::git2::Error) -> Error {
        Error::Fatal(e.into())
    }
}

#[cfg(feature="containers")]
impl From<Vec<::path_filter::FilterError>> for Error {
    fn from(e: Vec<::path_filter::FilterError>) -> Error {
        // TODO(tailhook) improve display
        Error::Fatal(format_err!("{:?}", e))
    }
}

impl From<String> for Error {
    fn from(e: String) -> Error {
        Error::Fatal(err_msg(e))
    }
}

impl From<&'static str> for Error {
    fn from(e: &'static str) -> Error {
        Error::Fatal(err_msg(e))
    }
}
