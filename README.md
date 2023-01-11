# witas

[![wita at crates.io](https://img.shields.io/crates/v/witas.svg)](https://crates.io/crates/witas)
[![wita at docs.rs](https://docs.rs/witas/badge.svg)](https://docs.rs/witas)

An asynchronous window library in Rust for Windows 

## The simple example

```rust
#[tokio::main]
async fn main() {
    let (_window, mut rx) = witas::Window::builder()
        .title("witas hello")
        .inner_size(witas::LogicalSize::new(640, 480))
        .await
        .unwrap();
    loop {
        let event = rx.recv().await; 
        println!("{:?}", event);
        if let witas::Event::Quit = event {
            break;
        }
    }
}
```

--------------------------------------------

Licensed under [MIT License](LICENSE)

Copylight (c) 2022 LNSEAB