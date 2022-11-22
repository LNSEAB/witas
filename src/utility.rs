use crate::*;
use windows::Win32::{
    Foundation::{HWND, RECT},
    UI::HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow},
    UI::WindowsAndMessaging::{GetClientRect, GetWindowRect, WINDOW_EX_STYLE, WINDOW_STYLE},
};

#[inline]
pub fn adjust_window_rect(
    size: impl ToPhysical<u32, Output<u32> = PhysicalSize<u32>>,
    style: WINDOW_STYLE,
    has_menu: bool,
    ex_style: WINDOW_EX_STYLE,
    dpi: u32,
) -> RECT {
    let size = size.to_physical(dpi);
    let mut rc = RECT {
        right: size.width as _,
        bottom: size.height as _,
        ..Default::default()
    };
    unsafe {
        AdjustWindowRectExForDpi(&mut rc, style, has_menu, ex_style, dpi);
    }
    rc
}

#[inline]
pub fn get_dpi_for_window(hwnd: HWND) -> u32 {
    unsafe { GetDpiForWindow(hwnd) }
}

#[inline]
pub fn get_client_rect(hwnd: HWND) -> RECT {
    let mut rc = RECT::default();
    unsafe {
        GetClientRect(hwnd, &mut rc);
    }
    rc
}

#[inline]
pub fn get_window_rect(hwnd: HWND) -> RECT {
    let mut rc = RECT::default();
    unsafe {
        GetWindowRect(hwnd, &mut rc);
    }
    rc
}
