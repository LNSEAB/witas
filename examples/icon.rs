#[tokio::main]
async fn main() {
    let (_window, mut rx) = witas::Window::builder()
        .title("witas icon")
        .inner_size(witas::LogicalSize::new(640, 480))
        .icon(witas::Icon::from_path("examples/icon_resource/icon.ico"))
        .build()
        .await
        .unwrap();
    loop {
        let event = rx.recv().await;
        if let witas::Event::Quit = event {
            break;
        }
    }
}

