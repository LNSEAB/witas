#[tokio::main]
async fn main() {
    let (_window, mut rx) = witas::Window::builder()
        .title("witas dialog")
        .inner_size(witas::LogicalSize::new(640, 480))
        .build()
        .await
        .unwrap();
    loop {
        let event = rx.recv().await;
        match event {
            witas::Event::KeyInput(input) if input.key_state == witas::KeyState::Pressed => {
                match input.key_code.vkey {
                    witas::VirtualKey::O => {
                        let path = witas::FileOpenDialog::new().show().await.unwrap();
                        println!("Open: {:?}", path);
                    }
                    witas::VirtualKey::M => {
                        let paths = witas::FileOpenDialog::new()
                            .allow_multi_select()
                            .show()
                            .await
                            .unwrap();
                        println!("Open(Multi): {:?}", paths);
                    }
                    witas::VirtualKey::S => {
                        let path = witas::FileSaveDialog::new().show().await.unwrap();
                        println!("Save: {:?}", path);
                    }
                    _ => {}
                }
            }
            witas::Event::Quit => break,
            _ => {}
        }
    }
}
