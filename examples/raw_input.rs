#[tokio::main]
async fn main() {
    let (_window, mut rx, raw_input) = witas::Window::builder()
        .title("witas hello")
        .inner_size(witas::LogicalSize::new(640, 480))
        .accept_drop_files(true)
        .enable_raw_input(true)
        .build()
        .await
        .unwrap();
    let mut raw_input = raw_input.unwrap();
    let th = tokio::spawn(async move {
        let device_list = witas::raw_input::get_device_list().unwrap();
        for device in &device_list {
            println!("{}", device.name());
            println!("{:?}", device.get_info());
        }
        loop {
            match raw_input.recv().await {
                witas::raw_input::RawInputEvent::Input(input) => {
                    println!("{:?}", input);
                },
                witas::raw_input::RawInputEvent::DeviceChange(dc) => {
                    println!("{:?}", dc);
                },
                witas::raw_input::RawInputEvent::Quit => break,
            }
        }
    });
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
    th.await.unwrap();
    if let Err(e) = witas::UiThread::join().await {
        std::panic::resume_unwind(e);
    }
}

