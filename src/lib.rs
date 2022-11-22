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

use context::Context;

pub use device::*;
pub use error::{Error, Result};
#[doc(inline)]
pub use events::{Event, ResizingEdge};
pub use geometry::*;
pub use ui_thread::UiThread;
pub use window::{EventReceiver, Window, WindowBuilder, WindowStyle};

#[inline]
pub fn init() {
    ui_thread::UiThread::init();
}
