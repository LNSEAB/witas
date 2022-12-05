//! An asynchronous window library in Rust for Windows

mod context;
mod device;
mod error;
pub mod events;
mod geometry;
pub mod ime;
mod procedure;
pub mod raw_input;
mod ui_thread;
mod utility;
mod window;

#[cfg(feature = "dialog")]
#[cfg_attr(docsrs, doc(cfg(feature = "dialog")))]
pub mod dialog;

use context::Context;

pub use device::*;
pub use error::{Error, Result};
#[doc(inline)]
pub use events::{Event, ResizingEdge};
pub use geometry::*;
pub use ui_thread::UiThread;
pub use window::{EventReceiver, Window, WindowBuilder, WindowStyle};

#[cfg(feature = "dialog")]
#[doc(inline)]
pub use dialog::{FileDialogOptions, FileOpenDialog, FileSaveDialog, FilterSpec};

#[inline]
pub fn init() {
    ui_thread::UiThread::init();
}
