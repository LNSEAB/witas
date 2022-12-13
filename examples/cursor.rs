#[tokio::main]
async fn main() {
    let (window, mut rx) = witas::Window::builder()
        .title("witas cursor")
        .inner_size(witas::LogicalSize::new(640, 480))
        .await
        .unwrap();
    loop {
        let event = rx.recv().await;
        match event {
            witas::Event::CharInput(input) => {
                let cursor = match input.c {
                    'd' => witas::Cursor::Arrow,
                    'h' => witas::Cursor::Hand,
                    'i' => witas::Cursor::IBeam,
                    'w' => witas::Cursor::Wait,
                    _ => continue,
                };
                window.set_cursor(cursor);
            }
            witas::Event::Quit => break,
            _ => {}
        }
    }
}
