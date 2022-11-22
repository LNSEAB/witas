#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Api(windows::core::Error),
    #[error("ui thread closed")]
    UiThreadClosed,
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

pub type Result<T> = ::core::result::Result<T, Error>;
