use crate::window::WindowProperties;
use crate::*;
use std::collections::HashMap;
use std::sync::Mutex;
use std::task::Waker;
use tokio::sync::mpsc;
use windows::Win32::Foundation::HWND;

pub(crate) struct Object {
    pub props: WindowProperties,
    pub sender: mpsc::UnboundedSender<Event>,
    pub waker: Option<Waker>,
    pub raw_input_sender: Option<mpsc::UnboundedSender<raw_input::RawInputEvent>>,
    pub raw_input_waker: Option<Waker>,
}

pub(crate) struct Context {
    window_map: HashMap<isize, Object>,
}

static CONTEXT: once_cell::sync::Lazy<Mutex<Context>> =
    once_cell::sync::Lazy::new(|| Mutex::new(Context::new()));

impl Context {
    fn new() -> Self {
        Self {
            window_map: HashMap::new(),
        }
    }

    pub fn is_empty() -> bool {
        let ctx = CONTEXT.lock().unwrap();
        ctx.window_map.is_empty()
    }

    pub fn register_window(
        hwnd: HWND,
        props: WindowProperties,
        sender: mpsc::UnboundedSender<Event>,
        raw_input_sender: Option<mpsc::UnboundedSender<raw_input::RawInputEvent>>,
    ) {
        let mut ctx = CONTEXT.lock().unwrap();
        ctx.window_map.insert(
            hwnd.0,
            Object {
                props,
                sender,
                waker: None,
                raw_input_sender,
                raw_input_waker: None,
            },
        );
    }

    pub fn send_event(hwnd: HWND, event: Event) {
        let mut ctx = CONTEXT.lock().unwrap();
        let Some(obj) = ctx.window_map.get_mut(&hwnd.0) else { return };
        obj.sender.send(event).unwrap_or(());
        if let Some(waker) = obj.waker.take() {
            waker.wake();
        }
    }

    pub fn send_raw_input_event(hwnd: HWND, event: raw_input::RawInputEvent) {
        let mut ctx = CONTEXT.lock().unwrap();
        let Some(obj) = ctx.window_map.get_mut(&hwnd.0) else { return };
        if let Some(sender) = obj.raw_input_sender.as_ref() {
            sender.send(event).unwrap_or(());
        }
        if let Some(waker) = obj.raw_input_waker.take() {
            waker.wake();
        }
    }

    pub fn quit() {
        let mut ctx = CONTEXT.lock().unwrap();
        for (_, obj) in ctx.window_map.iter_mut() {
            obj.sender.send(Event::Quit).unwrap_or(());
            if let Some(waker) = obj.waker.take() {
                waker.wake();
            }
        }
    }

    pub fn remove_window(hwnd: HWND) -> Option<Object> {
        let mut ctx = CONTEXT.lock().unwrap();
        ctx.window_map.remove(&hwnd.0)
    }

    pub fn set_waker(hwnd: HWND, waker: Waker) {
        let mut ctx = CONTEXT.lock().unwrap();
        let Some(mut obj) = ctx.window_map.get_mut(&hwnd.0) else { return };
        obj.waker = Some(waker);
    }

    pub fn set_raw_input_waker(hwnd: HWND, waker: Waker) {
        let mut ctx = CONTEXT.lock().unwrap();
        let Some(mut obj) = ctx.window_map.get_mut(&hwnd.0) else { return };
        obj.raw_input_waker = Some(waker);
    }

    pub fn get_window_property<F, T>(hwnd: HWND, f: F) -> Option<T>
    where
        F: FnOnce(&WindowProperties) -> T,
    {
        let ctx = CONTEXT.lock().unwrap();
        ctx.window_map.get(&hwnd.0).map(|obj| f(&obj.props))
    }

    pub fn set_window_property<F>(hwnd: HWND, f: F)
    where
        F: FnOnce(&mut WindowProperties),
    {
        let mut ctx = CONTEXT.lock().unwrap();
        if let Some(obj) = ctx.window_map.get_mut(&hwnd.0) {
            f(&mut obj.props);
        }
    }

    pub fn window_is_closed(hwnd: HWND) -> bool {
        let ctx = CONTEXT.lock().unwrap();
        !ctx.window_map.contains_key(&hwnd.0)
    }
}
