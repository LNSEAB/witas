#[tokio::main]
async fn main() {
    let (_window, mut rx) = witas::Window::builder()
        .title("witas hello")
        .inner_size(witas::LogicalSize::new(640, 480))
        .accept_drop_files(true)
        .build()
        .await
        .unwrap();
    loop {
        tokio::select! {
            event = rx.recv() => {
                println!("{:?}", event);
                if let witas::Event::Quit = event {
                    break;
                }
            }
        }
    }
    if let Err(e) = witas::UiThread::join().await {
        std::panic::resume_unwind(e);
    }
}
