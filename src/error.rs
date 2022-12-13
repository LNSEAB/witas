#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("{0}")]
    Api(windows::core::Error),
    #[error("ui thread closed")]
    UiThreadClosed,
    #[error("{0}")]
    Io(std::io::Error),
}

impl Error {
    pub(crate) fn from_win32() -> Self {
        windows::core::Error::from_win32().into()
    }
}

impl From<windows::core::Error> for Error {
    fn from(src: windows::core::Error) -> Self {
        Self::Api(src)
    }
}

impl From<std::io::Error> for Error {
    fn from(src: std::io::Error) -> Self {
        Self::Io(src)
    }
}

pub type Result<T> = ::core::result::Result<T, Error>;
