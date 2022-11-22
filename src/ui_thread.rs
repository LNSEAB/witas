use crate::*;
use once_cell::sync::OnceCell;
use std::os::windows::prelude::*;
use std::sync::{mpsc, Arc, Mutex};
use windows::Win32::{
    Foundation::{BOOL, HANDLE, HWND, LPARAM, WPARAM},
    System::Threading::GetThreadId,
    UI::HiDpi::{
        SetProcessDpiAwareness, SetProcessDpiAwarenessContext,
        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        PROCESS_PER_MONITOR_DPI_AWARE,
    },
    UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, IsGUIThread, PostThreadMessageW, TranslateMessage, MSG,
        WM_USER,
    },
};

fn enable_dpi_awareness() {
    unsafe {
        if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).as_bool() {
            return;
        };
        if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE).as_bool() {
            return;
        };
        SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE).unwrap_or(());
    }
}

pub(crate) const WM_SEND_TASK: u32 = WM_USER + 1;

struct Task(Box<dyn FnOnce() + Send>);

struct Thread {
    th: Option<std::thread::JoinHandle<()>>,
    tx_task: mpsc::Sender<Task>,
    join_waker: Arc<Mutex<Option<std::task::Waker>>>,
}

impl Thread {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel::<Task>();
        let (tmp_tx, tmp_rx) = mpsc::channel::<()>();
        let join_waker = Arc::new(Mutex::new(Option::<std::task::Waker>::None));
        let th_join_waker = join_waker.clone();
        let th = std::thread::spawn(move || unsafe {
            #[cfg(feature = "coinit")]
            let _coinit =
                coinit::init(coinit::APARTMENTTHREADED | coinit::DISABLE_OLE1DDE).unwrap();
            IsGUIThread(true);
            window::register_class();
            {
                tmp_tx.send(()).unwrap_or(());
            }
            let mut msg = MSG::default();
            loop {
                let ret = GetMessageW(&mut msg, HWND(0), 0, 0);
                if ret == BOOL(0) || ret == BOOL(-1) {
                    break;
                }
                match msg.message {
                    WM_SEND_TASK => {
                        let ret = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            if let Ok(task) = rx.recv() {
                                task.0();
                            }
                        }));
                        if let Err(e) = ret {
                            Context::set_ui_thread_unwind(e);
                            Context::quit();
                            break;
                        }
                    }
                    _ => {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
                if let Some(e) = procedure::get_unwind() {
                    Context::set_ui_thread_unwind(e);
                    Context::quit();
                    break;
                }
            }
            let mut join_waker = th_join_waker.lock().unwrap();
            if let Some(waker) = join_waker.take() {
                waker.wake();
            }
        });
        tmp_rx.recv().unwrap_or(());
        Self {
            th: Some(th),
            tx_task: tx,
            join_waker,
        }
    }

    fn post_message(&self, msg: u32) {
        unsafe {
            let th = GetThreadId(HANDLE(self.th.as_ref().unwrap().as_raw_handle() as _));
            PostThreadMessageW(th, msg, WPARAM(0), LPARAM(0));
        }
    }

    fn send_task(&self, f: impl FnOnce() + Send + 'static) {
        self.tx_task.send(Task(Box::new(f))).unwrap_or(());
        self.post_message(WM_SEND_TASK);
    }
}

static THREAD: OnceCell<Mutex<Thread>> = OnceCell::new();

pub struct JoinHandle;

impl std::future::Future for JoinHandle {
    type Output = Result<(), Box<dyn std::any::Any + Send>>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> std::task::Poll<Self::Output> {
        let t = THREAD.get().unwrap().lock().unwrap();
        let Some(th) = t.th.as_ref() else { return std::task::Poll::Ready(Ok(())) };
        let mut join_waker = t.join_waker.lock().unwrap();
        if th.is_finished() {
            std::task::Poll::Ready(
                Context::get_ui_thread_unwind()
                    .take()
                    .map(Err)
                    .unwrap_or(Ok(())),
            )
        } else {
            *join_waker = Some(cx.waker().clone());
            std::task::Poll::Pending
        }
    }
}

pub struct UiThread;

impl UiThread {
    pub(crate) fn init() {
        THREAD.get_or_init(|| {
            enable_dpi_awareness();
            Mutex::new(Thread::new())
        });
    }

    #[inline]
    pub fn send_task(f: impl FnOnce() + Send + 'static) {
        let thread = THREAD.get().unwrap().lock().unwrap();
        thread.send_task(f);
    }

    #[inline]
    pub fn join() -> JoinHandle {
        JoinHandle
    }
}
