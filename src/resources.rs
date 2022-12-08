use crate::*;
use std::path::{Path, PathBuf};
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

#[derive(Clone, PartialEq, Debug)]
pub enum Icon {
    Resource(u16),
    File(PathBuf),
}

impl Icon {
    #[inline]
    pub fn from_path(path: impl AsRef<Path>) -> Icon {
        Icon::File(path.as_ref().into())
    }
}

fn load_icon_impl(hinst: HINSTANCE, icon: &Icon, cx: i32, cy: i32) -> Result<HICON> {
    let icon = unsafe {
        match icon {
            Icon::Resource(id) => {
                LoadImageW(hinst, PCWSTR(*id as _), IMAGE_ICON, cx, cy, LR_SHARED)?
            }
            Icon::File(path) => {
                let path = path.to_string_lossy();
                LoadImageW(
                    HINSTANCE(0),
                    &HSTRING::from(path.as_ref()),
                    IMAGE_ICON,
                    cx,
                    cy,
                    LR_SHARED | LR_LOADFROMFILE,
                )?
            }
        }
    };
    Ok(HICON(icon.0))
}

pub(crate) fn load_icon(icon: &Icon, hinst: HINSTANCE) -> Result<HICON> {
    unsafe {
        load_icon_impl(
            hinst,
            icon,
            GetSystemMetrics(SM_CXICON),
            GetSystemMetrics(SM_CYICON),
        )
    }
}

pub(crate) fn load_small_icon(icon: &Icon, hinst: HINSTANCE) -> Result<HICON> {
    unsafe {
        load_icon_impl(
            hinst,
            icon,
            GetSystemMetrics(SM_CXSMICON),
            GetSystemMetrics(SM_CYSMICON),
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cursor {
    AppStarting,
    Arrow,
    Cross,
    Hand,
    Help,
    IBeam,
    No,
    SizeAll,
    SizeNESW,
    SizeNS,
    SizeNWSE,
    SizeWE,
    SizeUpArrow,
    Wait,
}

impl Cursor {
    pub(crate) fn name(&self) -> PCWSTR {
        match self {
            Self::AppStarting => IDC_APPSTARTING,
            Self::Arrow => IDC_ARROW,
            Self::Cross => IDC_CROSS,
            Self::Hand => IDC_HAND,
            Self::Help => IDC_HELP,
            Self::IBeam => IDC_IBEAM,
            Self::No => IDC_NO,
            Self::SizeAll => IDC_SIZEALL,
            Self::SizeNESW => IDC_SIZENESW,
            Self::SizeNS => IDC_SIZENS,
            Self::SizeNWSE => IDC_SIZENWSE,
            Self::SizeWE => IDC_SIZEWE,
            Self::SizeUpArrow => IDC_UPARROW,
            Self::Wait => IDC_WAIT,
        }
    }

    pub(crate) fn set(&self) {
        unsafe {
            SetCursor(LoadCursorW(HINSTANCE::default(), self.name()).unwrap());
        }
    }
}

impl Default for Cursor {
    #[inline]
    fn default() -> Self {
        Self::Arrow
    }
}
