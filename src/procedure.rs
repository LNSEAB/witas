use crate::utility::*;
use crate::*;
use std::any::Any;
use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::mpsc;
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM},
    Graphics::Gdi::{BeginPaint, EndPaint, GetUpdateRect, PAINTSTRUCT},
    UI::Controls::WM_MOUSELEAVE,
    UI::HiDpi::{EnableNonClientDpiScaling, GetDpiForWindow},
    UI::Input::Ime::{ISC_SHOWUIALLCANDIDATEWINDOW, ISC_SHOWUICOMPOSITIONWINDOW},
    UI::Input::KeyboardAndMouse::{
        ReleaseCapture, SetCapture, TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT, VIRTUAL_KEY,
    },
    UI::Shell::{DragFinish, DragQueryFileW, DragQueryPoint, HDROP},
    UI::WindowsAndMessaging::*,
};

fn loword(x: i32) -> i16 {
    (x & 0xffff) as _
}

fn hiword(x: i32) -> i16 {
    ((x >> 16) & 0xffff) as _
}

fn get_x_lparam(lp: LPARAM) -> i16 {
    (lp.0 & 0xffff) as _
}

fn get_y_lparam(lp: LPARAM) -> i16 {
    ((lp.0 >> 16) & 0xffff) as _
}

fn get_xbutton_wparam(wp: WPARAM) -> u16 {
    ((wp.0 >> 16) & 0xffff) as _
}

fn lparam_to_point<C>(lparam: LPARAM) -> Position<i32, C> {
    Position::new(get_x_lparam(lparam) as _, get_y_lparam(lparam) as _)
}

fn lparam_to_size(lparam: LPARAM) -> PhysicalSize<u32> {
    Size::new(get_x_lparam(lparam) as _, get_y_lparam(lparam) as _)
}

fn wparam_to_button(wparam: WPARAM) -> MouseButton {
    match get_xbutton_wparam(wparam) {
        0x0001 => MouseButton::Ex(0),
        0x0002 => MouseButton::Ex(1),
        _ => unreachable!(),
    }
}

thread_local! {
    static UNWIND: RefCell<Option<Box<dyn Any + Send>>> = RefCell::new(None);
    static ENTERED: RefCell<Option<HWND>> = RefCell::new(None);
}

fn set_unwind(e: Box<dyn Any + Send>) {
    UNWIND.with(|u| {
        *u.borrow_mut() = Some(e);
    });
}

pub(crate) fn get_unwind() -> Option<Box<dyn Any + Send>> {
    UNWIND.with(|u| u.borrow_mut().take())
}

unsafe fn on_paint(hwnd: HWND) -> LRESULT {
    let mut rc = RECT::default();
    GetUpdateRect(hwnd, Some(&mut rc), false);
    let mut ps = PAINTSTRUCT::default();
    let _hdc = BeginPaint(hwnd, &mut ps);
    EndPaint(hwnd, &ps);
    Context::send_event(
        hwnd,
        Event::Draw(events::Draw {
            invalid_rect: rc.into(),
        }),
    );
    LRESULT(0)
}

unsafe fn on_mouse_move(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let position = lparam_to_point(lparam);
    let buttons = MouseButtons::from(wparam);
    let entered = ENTERED.with(|entered| *entered.borrow());
    if entered.is_none() {
        TrackMouseEvent(&mut TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as _,
            dwFlags: TME_LEAVE,
            hwndTrack: hwnd,
            dwHoverTime: 0,
        });
        ENTERED.with(|entered| {
            *entered.borrow_mut() = Some(hwnd);
        });
        Context::send_event(
            hwnd,
            events::Event::CursorEntered(events::CursorEntered {
                mouse_state: MouseState { position, buttons },
            }),
        );
    } else {
        Context::send_event(
            hwnd,
            events::Event::CursorMoved(events::CursorMoved {
                mouse_state: MouseState { position, buttons },
            }),
        );
    }

    LRESULT(0)
}

unsafe fn on_set_cursor(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if loword(lparam.0 as _) != HTCLIENT as _ {
        return DefWindowProcW(hwnd, WM_SETCURSOR, wparam, lparam);
    }
    Context::get_window_property(hwnd, |props| props.cursor.set());
    LRESULT(0)
}

unsafe fn on_mouse_leave(hwnd: HWND, wparam: WPARAM, _lparam: LPARAM) -> LRESULT {
    ENTERED.with(|entered| {
        *entered.borrow_mut() = None;
    });
    let mut position = POINT::default();
    GetCursorPos(&mut position);
    let buttons = MouseButtons::from(wparam);
    Context::send_event(
        hwnd,
        Event::CursorLeft(events::CursorLeft {
            mouse_state: MouseState {
                position: (position.x, position.y).into(),
                buttons,
            },
        }),
    );
    LRESULT(0)
}

unsafe fn on_mouse_input(
    hwnd: HWND,
    button: MouseButton,
    button_state: ButtonState,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match button_state {
        ButtonState::Pressed => {
            SetCapture(hwnd);
        }
        ButtonState::Released => {
            ReleaseCapture();
        }
    }
    let position = lparam_to_point(lparam);
    Context::send_event(
        hwnd,
        Event::MouseInput(events::MouseInput {
            button,
            button_state,
            mouse_state: MouseState {
                position,
                buttons: wparam.into(),
            },
        }),
    );
    LRESULT(0)
}

unsafe fn on_mouse_wheel(
    hwnd: HWND,
    axis: MouseWheelAxis,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let delta = hiword(wparam.0 as _);
    let buttons = MouseButtons::from(wparam);
    let position = lparam_to_point(lparam);
    Context::send_event(
        hwnd,
        Event::MouseWheel(events::MouseWheel {
            axis,
            distance: delta as i32,
            mouse_state: MouseState { position, buttons },
        }),
    );
    LRESULT(0)
}

unsafe fn on_key_input(hwnd: HWND, key_state: KeyState, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let vkey = as_virtual_key(VIRTUAL_KEY(wparam.0 as _));
    let scan_code = ScanCode(((lparam.0 >> 16) & 0x7f) as u32);
    let prev_pressed = (lparam.0 >> 30) & 0x01 != 0;
    Context::send_event(
        hwnd,
        Event::KeyInput(events::KeyInput {
            key_code: KeyCode::new(vkey, scan_code),
            key_state,
            prev_pressed,
        }),
    );
    LRESULT(0)
}

unsafe fn on_char(hwnd: HWND, wparam: WPARAM, _lparam: LPARAM) -> LRESULT {
    if let Some(c) = char::from_u32(wparam.0 as _) {
        Context::send_event(hwnd, Event::CharInput(events::CharInput { c }));
    }
    LRESULT(0)
}

unsafe fn on_ime_set_context(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let lparam = {
        let mut value = lparam.0 as u32;
        value &= !ISC_SHOWUICOMPOSITIONWINDOW;
        let candidate =
            Context::get_window_property(hwnd, |prop| prop.visible_ime_candidate_window);
        if !candidate.unwrap_or(true) {
            value &= !ISC_SHOWUIALLCANDIDATEWINDOW;
        }
        LPARAM(value as _)
    };
    DefWindowProcW(hwnd, WM_IME_SETCONTEXT, wparam, lparam)
}

unsafe fn on_ime_start_composition(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let imc = ime::Imc::get(hwnd);
    let (tx, rx) = mpsc::channel();
    Context::send_event(
        hwnd,
        Event::ImeStartComposition(events::ImeStartComposition::new(tx)),
    );
    if let Ok(position) = rx.recv() {
        imc.set_candidate_window_position(position, false);
    }
    DefWindowProcW(hwnd, WM_IME_STARTCOMPOSITION, wparam, lparam)
}

unsafe fn on_ime_composition(hwnd: HWND, _wparam: WPARAM, _lparam: LPARAM) -> LRESULT {
    let imc = ime::Imc::get(hwnd);
    let Some(composition) = events::ImeComposition::new(&imc) else { return LRESULT(0) };
    Context::send_event(hwnd, Event::ImeComposition(composition));
    LRESULT(0)
}

unsafe fn on_ime_end_composition(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let imc = ime::Imc::get(hwnd);
    Context::send_event(
        hwnd,
        Event::ImeEndComposition(events::ImeEndComposition {
            result: imc.get_composition_result(),
        }),
    );
    DefWindowProcW(hwnd, WM_IME_ENDCOMPOSITION, wparam, lparam)
}

unsafe fn on_sizing(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let d = {
        let wrc = get_window_rect(hwnd);
        let crc = get_client_rect(hwnd);
        PhysicalSize::new(
            (wrc.right - wrc.left) - (crc.right - crc.left),
            (wrc.bottom - wrc.top) - (crc.bottom - crc.top),
        )
    };
    let rc = (lparam.0 as *mut RECT).as_mut().unwrap();
    let size = PhysicalSize::new(
        (rc.right - rc.left - d.width) as u32,
        (rc.bottom - rc.left - d.height) as u32,
    );
    let edge = match wparam.0 as u32 {
        WMSZ_LEFT => ResizingEdge::Left,
        WMSZ_RIGHT => ResizingEdge::Right,
        WMSZ_TOP => ResizingEdge::Top,
        WMSZ_BOTTOM => ResizingEdge::Bottom,
        WMSZ_TOPLEFT => ResizingEdge::TopLeft,
        WMSZ_TOPRIGHT => ResizingEdge::TopRight,
        WMSZ_BOTTOMLEFT => ResizingEdge::BottomLeft,
        WMSZ_BOTTOMRIGHT => ResizingEdge::BottomLRight,
        _ => unreachable!(),
    };
    Context::send_event(hwnd, Event::Resizing(events::Resizing { size, edge }));
    DefWindowProcW(hwnd, WM_SIZING, wparam, lparam)
}

unsafe fn on_size(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match wparam.0 as u32 {
        SIZE_MINIMIZED => {
            Context::send_event(hwnd, Event::Minimized);
            Context::set_window_property(hwnd, |props| props.minimized = true);
        }
        SIZE_MAXIMIZED => {
            let size = lparam_to_size(lparam);
            Context::send_event(hwnd, Event::Maximized(events::Maximized { size }));
            Context::set_window_property(hwnd, |props| props.maximized = true);
        }
        SIZE_RESTORED => {
            let min_or_max =
                Context::get_window_property(hwnd, |props| props.minimized | props.maximized);
            if min_or_max.unwrap_or(false) {
                let size = lparam_to_size(lparam);
                Context::send_event(hwnd, Event::Restored(events::Restored { size }));
                Context::set_window_property(hwnd, |props| {
                    props.minimized = false;
                    props.maximized = false;
                });
            }
        }
        _ => {}
    }
    LRESULT(0)
}

unsafe fn on_window_pos_changed(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let pos = (lparam.0 as *const WINDOWPOS).as_ref().unwrap();
    if pos.flags.0 & SWP_NOMOVE.0 == 0 {
        Context::send_event(
            hwnd,
            Event::Moved(events::Moved {
                position: ScreenPosition::new(pos.x, pos.y),
            }),
        );
    }
    DefWindowProcW(hwnd, WM_WINDOWPOSCHANGED, wparam, lparam)
}

unsafe fn on_exit_size_move(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let size = get_client_rect(hwnd);
    Context::send_event(
        hwnd,
        Event::Resized(events::Resized {
            size: Size::new((size.right - size.left) as _, (size.bottom - size.top) as _),
        }),
    );
    DefWindowProcW(hwnd, WM_EXITSIZEMOVE, wparam, lparam)
}

unsafe fn on_activate(hwnd: HWND, wparam: WPARAM, _lparam: LPARAM) -> LRESULT {
    let active = (wparam.0 as u32 & (WA_ACTIVE | WA_CLICKACTIVE)) != 0;
    if active {
        Context::send_event(hwnd, Event::Activated);
    } else {
        Context::send_event(hwnd, Event::Inactivated);
    }
    LRESULT(0)
}

unsafe fn on_dpi_changed(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let rc = *(lparam.0 as *const RECT);
    SetWindowPos(
        hwnd,
        HWND(0),
        rc.left,
        rc.top,
        rc.right - rc.left,
        rc.bottom - rc.top,
        SWP_NOZORDER | SWP_NOACTIVATE,
    );
    let new_dpi = hiword(wparam.0 as _) as u32;
    Context::send_event(hwnd, Event::DpiChanged(events::DpiChanged { new_dpi }));
    LRESULT(0)
}

unsafe fn on_get_dpi_scaled_size(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let prev_dpi = GetDpiForWindow(hwnd) as i32;
    let next_dpi = wparam.0 as i32;
    let rc = get_client_rect(hwnd);
    let size = PhysicalSize::new(
        ((rc.right - rc.left) * next_dpi / prev_dpi) as u32,
        ((rc.bottom - rc.top) * next_dpi / prev_dpi) as u32,
    );
    let rc = adjust_window_rect(
        size,
        WINDOW_STYLE(GetWindowLongPtrW(hwnd, GWL_STYLE) as _),
        false,
        WINDOW_EX_STYLE(GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as _),
        next_dpi as _,
    );
    let mut ret = (lparam.0 as *mut SIZE).as_mut().unwrap();
    ret.cx = rc.right - rc.left;
    ret.cy = rc.bottom - rc.top;
    LRESULT(1)
}

unsafe fn on_drop_files(hwnd: HWND, wparam: WPARAM, _lparam: LPARAM) -> LRESULT {
    let hdrop = HDROP(wparam.0 as _);
    let file_count = DragQueryFileW(hdrop, u32::MAX, None);
    let mut paths = Vec::with_capacity(file_count as _);
    let mut buffer = Vec::new();
    for i in 0..file_count {
        let len = DragQueryFileW(hdrop, i, None) as usize + 1;
        buffer.resize(len, 0);
        DragQueryFileW(hdrop, i, Some(&mut buffer));
        buffer.pop();
        let path = PathBuf::from(String::from_utf16_lossy(&buffer));
        paths.push(path);
    }
    let mut position = POINT::default();
    DragQueryPoint(hdrop, &mut position);
    Context::send_event(
        hwnd,
        Event::DropFiles(events::DropFiles {
            paths,
            position: position.into(),
        }),
    );
    DragFinish(hdrop);
    LRESULT(0)
}

unsafe fn on_nc_create(hwnd: HWND, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    EnableNonClientDpiScaling(hwnd);
    DefWindowProcW(hwnd, WM_NCCREATE, wparam, lparam)
}

unsafe fn on_destroy(hwnd: HWND) -> LRESULT {
    Context::send_event(hwnd, Event::Closed);
    let obj = Context::remove_window(hwnd);
    if Context::is_empty() {
        if let Some(obj) = obj {
            obj.sender.send(Event::Quit).unwrap_or(());
            if let Some(waker) = obj.waker {
                waker.wake();
            }
            if let Some(sender) = obj.raw_input_sender {
                sender.send(raw_input::RawInputEvent::Quit).unwrap_or(());
            }
            if let Some(waker) = obj.raw_input_waker {
                waker.wake();
            }
        }
        PostQuitMessage(0);
    }
    LRESULT(0)
}

pub(crate) extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ret = std::panic::catch_unwind(|| unsafe {
        match msg {
            WM_INPUT => raw_input::on_input(hwnd, wparam, lparam),
            WM_PAINT => on_paint(hwnd),
            WM_MOUSEMOVE => on_mouse_move(hwnd, wparam, lparam),
            WM_SETCURSOR => on_set_cursor(hwnd, wparam, lparam),
            WM_MOUSELEAVE => on_mouse_leave(hwnd, wparam, lparam),
            WM_LBUTTONDOWN => on_mouse_input(
                hwnd,
                MouseButton::Left,
                ButtonState::Pressed,
                wparam,
                lparam,
            ),
            WM_RBUTTONDOWN => on_mouse_input(
                hwnd,
                MouseButton::Right,
                ButtonState::Pressed,
                wparam,
                lparam,
            ),
            WM_MBUTTONDOWN => on_mouse_input(
                hwnd,
                MouseButton::Middle,
                ButtonState::Pressed,
                wparam,
                lparam,
            ),
            WM_XBUTTONDOWN => on_mouse_input(
                hwnd,
                wparam_to_button(wparam),
                ButtonState::Pressed,
                wparam,
                lparam,
            ),
            WM_LBUTTONUP => on_mouse_input(
                hwnd,
                MouseButton::Left,
                ButtonState::Released,
                wparam,
                lparam,
            ),
            WM_RBUTTONUP => on_mouse_input(
                hwnd,
                MouseButton::Right,
                ButtonState::Released,
                wparam,
                lparam,
            ),
            WM_MBUTTONUP => on_mouse_input(
                hwnd,
                MouseButton::Middle,
                ButtonState::Released,
                wparam,
                lparam,
            ),
            WM_XBUTTONUP => on_mouse_input(
                hwnd,
                wparam_to_button(wparam),
                ButtonState::Released,
                wparam,
                lparam,
            ),
            WM_MOUSEWHEEL => on_mouse_wheel(hwnd, MouseWheelAxis::Vertical, wparam, lparam),
            WM_MOUSEHWHEEL => on_mouse_wheel(hwnd, MouseWheelAxis::Horizontal, wparam, lparam),
            WM_KEYDOWN => on_key_input(hwnd, KeyState::Pressed, wparam, lparam),
            WM_KEYUP => on_key_input(hwnd, KeyState::Released, wparam, lparam),
            WM_CHAR => on_char(hwnd, wparam, lparam),
            WM_IME_SETCONTEXT => on_ime_set_context(hwnd, wparam, lparam),
            WM_IME_STARTCOMPOSITION => on_ime_start_composition(hwnd, wparam, lparam),
            WM_IME_COMPOSITION => on_ime_composition(hwnd, wparam, lparam),
            WM_IME_ENDCOMPOSITION => on_ime_end_composition(hwnd, wparam, lparam),
            WM_SIZING => on_sizing(hwnd, wparam, lparam),
            WM_SIZE => on_size(hwnd, wparam, lparam),
            WM_WINDOWPOSCHANGED => on_window_pos_changed(hwnd, wparam, lparam),
            WM_EXITSIZEMOVE => on_exit_size_move(hwnd, wparam, lparam),
            WM_ACTIVATE => on_activate(hwnd, wparam, lparam),
            WM_DPICHANGED => on_dpi_changed(hwnd, wparam, lparam),
            WM_GETDPISCALEDSIZE => on_get_dpi_scaled_size(hwnd, wparam, lparam),
            WM_DROPFILES => on_drop_files(hwnd, wparam, lparam),
            WM_NCCREATE => on_nc_create(hwnd, wparam, lparam),
            WM_DESTROY => on_destroy(hwnd),
            WM_INPUT_DEVICE_CHANGE => raw_input::on_input_device_change(hwnd, wparam, lparam),
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    });
    ret.unwrap_or_else(|e| {
        set_unwind(e);
        LRESULT(0)
    })
}
