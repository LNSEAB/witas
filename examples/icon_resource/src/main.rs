#[tokio::main]
async fn main() {
    let (_window, mut rx) = witas::Window::builder()
        .title("witas icon_resource")
        .inner_size(witas::LogicalSize::new(640, 480))
        .icon(witas::Icon::Resource(111))
        .await
        .unwrap();
    loop {
        let event = rx.recv().await;
        if let witas::Event::Quit = event {
            break;
        }
    }
}
