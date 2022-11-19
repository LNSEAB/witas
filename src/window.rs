use crate::utility::adjust_window_rect;
use crate::*;
use tokio::sync::{mpsc, oneshot};
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::{
    Foundation::{HWND, POINT},
    Graphics::Gdi::{
        GetStockObject, MonitorFromPoint, HBRUSH, MONITOR_DEFAULTTOPRIMARY, WHITE_BRUSH,
    },
    System::LibraryLoader::GetModuleHandleW,
    UI::HiDpi::{GetDpiForMonitor, MDT_DEFAULT},
    UI::Shell::DragAcceptFiles,
    UI::WindowsAndMessaging::{
        CreateWindowExW, LoadCursorW, RegisterClassExW, ShowWindow, CS_HREDRAW, CS_VREDRAW,
        IDC_ARROW, SW_SHOW, WINDOW_EX_STYLE, WINDOW_STYLE, WNDCLASSEXW, WS_CAPTION, WS_MAXIMIZEBOX,
        WS_MINIMIZEBOX, WS_OVERLAPPED, WS_OVERLAPPEDWINDOW, WS_POPUP, WS_SYSMENU, WS_THICKFRAME,
    },
};

const WINDOW_CLASS_NAME: PCWSTR = windows::w!("witas_window_class");

pub(crate) fn register_class() {
    unsafe {
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as _,
            style: CS_VREDRAW | CS_HREDRAW,
            lpfnWndProc: Some(procedure::window_proc),
            hInstance: GetModuleHandleW(None).unwrap(),
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
            lpszClassName: WINDOW_CLASS_NAME,
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            ..Default::default()
        };
        if RegisterClassExW(&wc) == 0 {
            panic!("RegisterClassExW failed");
        }
    }
}

fn get_dpi_from_point(pt: ScreenPosition) -> u32 {
    let mut dpi_x = 0;
    let mut dpi_y = 0;
    unsafe {
        GetDpiForMonitor(
            MonitorFromPoint(POINT { x: pt.x, y: pt.y }, MONITOR_DEFAULTTOPRIMARY),
            MDT_DEFAULT,
            &mut dpi_x,
            &mut dpi_y,
        )
        .unwrap_or(());
    }
    dpi_x
}

pub(crate) struct WindowProperties {
    pub visible_ime_candidate_window: bool,
    pub imm_context: ime::ImmContext,
}

pub trait Style {
    fn style(&self) -> WINDOW_STYLE;
    fn ex_style(&self) -> WINDOW_EX_STYLE;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BorderlessStyle;

impl Style for BorderlessStyle {
    #[inline]
    fn style(&self) -> WINDOW_STYLE {
        WS_POPUP
    }

    #[inline]
    fn ex_style(&self) -> WINDOW_EX_STYLE {
        WINDOW_EX_STYLE::default()
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct WindowStyle {
    style: WINDOW_STYLE,
    ex_style: WINDOW_EX_STYLE,
}

impl WindowStyle {
    #[inline]
    pub fn dialog() -> Self {
        Self {
            style: WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU,
            ex_style: WINDOW_EX_STYLE::default(),
        }
    }

    #[inline]
    pub fn borderless() -> BorderlessStyle {
        BorderlessStyle
    }

    #[inline]
    pub fn resizable(mut self, resizable: bool) -> Self {
        if resizable {
            self.style |= WS_THICKFRAME;
        } else {
            self.style &= !WS_THICKFRAME;
        }
        self
    }

    #[inline]
    pub fn has_minimize_box(mut self, has_box: bool) -> Self {
        if has_box {
            self.style |= WS_MINIMIZEBOX;
        } else {
            self.style &= !WS_MINIMIZEBOX;
        }
        self
    }

    #[inline]
    pub fn has_maximize_box(mut self, has_box: bool) -> Self {
        if has_box {
            self.style |= WS_MAXIMIZEBOX;
        } else {
            self.style &= !WS_MAXIMIZEBOX;
        }
        self
    }
}

impl Default for WindowStyle {
    #[inline]
    fn default() -> Self {
        Self {
            style: WS_OVERLAPPEDWINDOW,
            ex_style: WINDOW_EX_STYLE::default(),
        }
    }
}

impl Style for WindowStyle {
    #[inline]
    fn style(&self) -> WINDOW_STYLE {
        self.style
    }

    #[inline]
    fn ex_style(&self) -> WINDOW_EX_STYLE {
        self.ex_style
    }
}

pub struct WindowBuilder<Title = (), Sz = ()> {
    title: Title,
    position: ScreenPosition,
    size: Sz,
    style: Box<dyn Style + Send>,
    visibility: bool,
    enable_ime: bool,
    visible_ime_candidate_window: bool,
    accept_drop_files: bool,
    enable_raw_input: bool,
}

impl WindowBuilder<(), ()> {
    fn new() -> Self {
        Self {
            title: (),
            position: (0, 0).into(),
            size: (),
            style: Box::new(WindowStyle::default()),
            visibility: true,
            enable_ime: true,
            visible_ime_candidate_window: true,
            accept_drop_files: false,
            enable_raw_input: false,
        }
    }
}

impl<Title, Sz> WindowBuilder<Title, Sz> {
    #[inline]
    pub fn title(self, title: impl Into<String>) -> WindowBuilder<String, Sz> {
        WindowBuilder {
            title: title.into(),
            position: self.position,
            size: self.size,
            style: self.style,
            visibility: self.visibility,
            enable_ime: self.enable_ime,
            visible_ime_candidate_window: self.visible_ime_candidate_window,
            accept_drop_files: self.accept_drop_files,
            enable_raw_input: self.enable_raw_input,
        }
    }

    #[inline]
    pub fn position(mut self, position: impl Into<ScreenPosition>) -> Self {
        self.position = position.into();
        self
    }

    #[inline]
    pub fn inner_size<T>(self, size: T) -> WindowBuilder<Title, T>
    where
        T: ToPhysical<u32, Output<u32> = PhysicalSize<u32>>,
    {
        WindowBuilder {
            title: self.title,
            position: self.position,
            size,
            style: self.style,
            visibility: self.visibility,
            enable_ime: self.enable_ime,
            visible_ime_candidate_window: self.visible_ime_candidate_window,
            accept_drop_files: self.accept_drop_files,
            enable_raw_input: self.enable_raw_input,
        }
    }

    #[inline]
    pub fn style(mut self, style: impl Style + Send + 'static) -> Self {
        self.style = Box::new(style);
        self
    }

    #[inline]
    pub fn visible(mut self, visibility: bool) -> Self {
        self.visibility = visibility;
        self
    }

    #[inline]
    pub fn enable_ime(mut self, enable: bool) -> Self {
        self.enable_ime = enable;
        self
    }

    #[inline]
    pub fn visible_ime_candidate_window(mut self, visible: bool) -> Self {
        self.visible_ime_candidate_window = visible;
        self
    }

    #[inline]
    pub fn accept_drop_files(mut self, accept: bool) -> Self {
        self.accept_drop_files = accept;
        self
    }

    #[inline]
    pub fn enable_raw_input(mut self, enable: bool) -> Self {
        self.enable_raw_input = enable;
        self
    }
}

impl<Sz> WindowBuilder<String, Sz>
where
    Sz: ToPhysical<u32, Output<u32> = PhysicalSize<u32>>,
{
    #[inline]
    pub fn build(self) -> Build<Sz> {
        Build {
            builder: Some(self),
            rx: None,
        }
    }
}

pub struct Recv<'a, T> {
    hwnd: HWND,
    rx: &'a mut mpsc::UnboundedReceiver<T>,
}

impl<'a> std::future::Future for Recv<'a, Event> {
    type Output = Event;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();
        match this.rx.try_recv() {
            Ok(event) => std::task::Poll::Ready(event),
            Err(mpsc::error::TryRecvError::Empty) => {
                Context::set_waker(this.hwnd, cx.waker().clone());
                std::task::Poll::Pending
            }
            Err(mpsc::error::TryRecvError::Disconnected) => std::task::Poll::Ready(Event::Quit),
        }
    }
}

impl<'a> std::future::Future for Recv<'a, raw_input::RawInputEvent> {
    type Output = raw_input::RawInputEvent;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();
        match this.rx.try_recv() {
            Ok(event) => std::task::Poll::Ready(event),
            Err(mpsc::error::TryRecvError::Empty) => {
                Context::set_raw_input_waker(this.hwnd, cx.waker().clone());
                std::task::Poll::Pending
            }
            Err(mpsc::error::TryRecvError::Disconnected) => {
                std::task::Poll::Ready(raw_input::RawInputEvent::Quit)
            }
        }
    }
}

pub struct EventReceiver {
    hwnd: HWND,
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventReceiver {
    #[inline]
    pub fn recv(&mut self) -> Recv<Event> {
        Recv {
            hwnd: self.hwnd,
            rx: &mut self.rx,
        }
    }
}

pub struct RawInputEventRecevier {
    hwnd: HWND,
    rx: mpsc::UnboundedReceiver<raw_input::RawInputEvent>,
}

impl RawInputEventRecevier {
    #[inline]
    pub fn recv(&mut self) -> Recv<raw_input::RawInputEvent> {
        Recv {
            hwnd: self.hwnd,
            rx: &mut self.rx,
        }
    }
}

type BuildResult = anyhow::Result<(
    HWND,
    mpsc::UnboundedReceiver<Event>,
    Option<mpsc::UnboundedReceiver<raw_input::RawInputEvent>>,
)>;

pub struct Build<Sz> {
    builder: Option<WindowBuilder<String, Sz>>,
    rx: Option<mpsc::UnboundedReceiver<BuildResult>>,
}

impl<Sz> std::future::Future for Build<Sz>
where
    Sz: ToPhysical<u32, Output<u32> = PhysicalSize<u32>> + std::marker::Unpin + Send + 'static,
{
    type Output = anyhow::Result<(Window, EventReceiver, Option<RawInputEventRecevier>)>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();
        if let Some(builder) = this.builder.take() {
            let (tx, rx) = mpsc::unbounded_channel::<BuildResult>();
            let waker = cx.waker().clone();
            let create_window = move || unsafe {
                let title: HSTRING = builder.title.into();
                let dpi = get_dpi_from_point(builder.position);
                let size = builder.size.to_physical(dpi);
                let style = builder.style.style();
                let ex_style = builder.style.ex_style();
                let rc = adjust_window_rect(size, style, false, ex_style, dpi);
                let hwnd = CreateWindowExW(
                    ex_style,
                    WINDOW_CLASS_NAME,
                    &title,
                    style,
                    builder.position.x,
                    builder.position.y,
                    rc.right - rc.left,
                    rc.bottom - rc.top,
                    None,
                    None,
                    GetModuleHandleW(None).unwrap(),
                    None,
                );
                DragAcceptFiles(hwnd, builder.accept_drop_files);
                let props = WindowProperties {
                    visible_ime_candidate_window: builder.visible_ime_candidate_window,
                    imm_context: ime::ImmContext::new(hwnd),
                };
                if builder.enable_ime {
                    props.imm_context.enable();
                } else {
                    props.imm_context.disable();
                }
                let (event_rx, raw_input_event_rx) = {
                    let (event_tx, event_rx) = mpsc::unbounded_channel();
                    let raw_input_event_rx = if builder.enable_raw_input {
                        if let Ok(_) =
                            raw_input::register_devices(hwnd, raw_input::WindowState::Foreground)
                        {
                            let (raw_input_event_tx, raw_input_event_rx) =
                                mpsc::unbounded_channel();
                            Context::register_window(
                                hwnd,
                                props,
                                event_tx,
                                Some(raw_input_event_tx),
                            );
                            Some(raw_input_event_rx)
                        } else {
                            None
                        }
                    } else {
                        Context::register_window(hwnd, props, event_tx, None);
                        None
                    };
                    (event_rx, raw_input_event_rx)
                };
                if builder.visibility {
                    ShowWindow(hwnd, SW_SHOW);
                }
                tx.send(Ok((hwnd, event_rx, raw_input_event_rx)))
                    .unwrap_or(());
                waker.wake();
            };
            UiThread::send_task(create_window);
            this.rx = Some(rx);
            return std::task::Poll::Pending;
        }
        match this.rx.as_mut().unwrap().try_recv() {
            Ok(ret) => std::task::Poll::Ready(ret.map(|(hwnd, rx, raw_input_rx)| {
                let rx = EventReceiver {
                    hwnd: hwnd.clone(),
                    rx,
                };
                let raw_input_rx = raw_input_rx.map(|rx| RawInputEventRecevier {
                    hwnd: hwnd.clone(),
                    rx,
                });
                (Window { hwnd }, rx, raw_input_rx)
            })),
            Err(mpsc::error::TryRecvError::Empty) => std::task::Poll::Pending,
            Err(mpsc::error::TryRecvError::Disconnected) => {
                std::task::Poll::Ready(Err(anyhow::anyhow!("disconnected")))
            }
        }
    }
}

pub struct Window {
    hwnd: HWND,
}

impl Window {
    #[inline]
    pub fn builder() -> WindowBuilder {
        crate::init();
        WindowBuilder::new()
    }

    #[inline]
    pub async fn position(&self) -> Option<ScreenPosition> {
        let hwnd = self.hwnd;
        let (tx, rx) = oneshot::channel();
        UiThread::send_task(move || {
            let rc = utility::get_window_rect(hwnd);
            tx.send((rc.left, rc.top).into()).unwrap_or(());
        });
        rx.await.ok()
    }

    #[inline]
    pub async fn inner_size(&self) -> Option<PhysicalSize<u32>> {
        let hwnd = self.hwnd;
        let (tx, rx) = oneshot::channel();
        UiThread::send_task(move || {
            let rc = utility::get_client_rect(hwnd);
            tx.send(((rc.right - rc.left) as u32, (rc.bottom - rc.top) as u32).into())
                .unwrap_or(());
        });
        rx.await.ok()
    }

    #[inline]
    pub async fn accept_drop_files(&self, accept: bool) {
        let hwnd = self.hwnd;
        UiThread::send_task(move || unsafe {
            DragAcceptFiles(hwnd, accept);
        });
    }

    #[inline]
    pub fn raw_handle(&self) -> *const std::ffi::c_void {
        self.hwnd.0 as _
    }
}
