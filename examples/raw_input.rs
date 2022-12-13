#[tokio::main]
async fn main() {
    let (_window, mut rx) = witas::Window::builder()
        .title("witas raw_input")
        .inner_size(witas::LogicalSize::new(640, 480))
        .enable_raw_input(true)
        .await
        .unwrap();
    let mut raw_input = rx.take_raw_input_receiver().unwrap();
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
                }
                witas::raw_input::RawInputEvent::DeviceChange(dc) => {
                    println!("{:?}", dc);
                }
                witas::raw_input::RawInputEvent::Quit => break,
            }
        }
    });
    loop {
        let event = rx.recv().await;
        println!("{:?}", event);
        if let witas::Event::Quit = event {
            break;
        }
    }
    th.await.unwrap();
}
