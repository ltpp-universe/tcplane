## tcplane

[![](https://img.shields.io/crates/v/tcplane.svg)](https://crates.io/crates/tcplane)
[![](https://docs.rs/tcplane/badge.svg)](https://docs.rs/tcplane)
[![](https://img.shields.io/crates/l/tcplane.svg)](./LICENSE)
[![](https://github.com/ltpp-universe/tcplane/workflows/Rust/badge.svg)](https://github.com/ltpp-universe/tcplane/actions?query=workflow:Rust)

[Official Documentation](https://docs.ltpp.vip/tcplane/)

[Api Docs](https://docs.rs/tcplane/latest/tcplane/)

> tcplane is a lightweight and high-performance Rust TCP server library designed to simplify network service development. It supports TCP communication, data stream management, and connection handling, focusing on providing efficient low-level network connections and data transmission capabilities, making it ideal for building modern network services.

## Installation

To use this crate, you can run cmd:

```shell
cargo add tcplane
```

## Use

```rust
use tcplane::*;

fn sync_middleware(arc_lock_controller_data: ArcRwLockControllerData) {
    let mut controller_data: RwLockWriteControllerData =
        get_rw_lock_write_controller_data(&arc_lock_controller_data);
    {
        let request: &mut Vec<u8> = controller_data.get_mut_request();
        let mut new_request: Vec<u8> = request.clone();
        let ext: Vec<u8> = "test".as_bytes().to_vec();
        new_request.extend(ext);
        *request = new_request;
    }
    let request: Request = controller_data.get_request().clone();
    let stream: ArcTcpStream = controller_data.get_stream().clone().unwrap();
    let host: String = stream
        .peer_addr()
        .and_then(|host| Ok(host.to_string()))
        .unwrap_or("Unknown".to_owned());
    controller_data.get_log().debug(
        format!(
            "Request host => {}\n{:#?}\n",
            host,
            String::from_utf8_lossy(&request),
        ),
        log_debug_format_handler,
    );
}

async fn async_middleware(arc_lock_controller_data: ArcRwLockControllerData) {
    let controller_data: ControllerData = get_controller_data(&arc_lock_controller_data);
    println!(
        "async middleware request{:?}",
        String::from_utf8_lossy(controller_data.get_request())
    );
}

fn sync_func(arc_lock_controller_data: ArcRwLockControllerData) {
    let controller_data: ControllerData = get_controller_data(&arc_lock_controller_data);
    let stream: ArcTcpStream = controller_data.get_stream().clone().unwrap();
    let res: ResponseResult = controller_data
        .get_response()
        .clone()
        .set_data("hello world".into())
        .send(&stream);
    controller_data.get_log().debug(
        format!("Response => {:?}\n", String::from_utf8_lossy(&res.unwrap())),
        log_debug_format_handler,
    );
}

async fn async_func(arc_lock_controller_data: ArcRwLockControllerData) {
    let controller_data: ControllerData = get_controller_data(&arc_lock_controller_data);
    let stream: ArcTcpStream = controller_data.get_stream().clone().unwrap();
    let res: ResponseResult = controller_data
        .get_response()
        .clone()
        .set_data("Async".into())
        .send(&stream);
    controller_data.get_log().debug(
        format!("Response => {:?}\n", String::from_utf8_lossy(&res.unwrap())),
        log_debug_format_handler,
    );
}

async fn run_server() {
    let mut server: Server = Server::new();
    server.host("0.0.0.0");
    server.port(60000);
    server.log_dir("./logs");
    server.log_size(100_024_000);
    server.buffer(100_024_000);
    server.log_interval_millis(360);
    server.middleware(sync_middleware);
    server.async_middleware(async_middleware).await;
    server.func(sync_func);
    server.async_func(async_func).await;
    let test_string: String = "test".to_owned();
    server
        .async_func(async_func!(test_string, |data| {
            println(&test_string);
            println(&format!("{:?}", data));
        }))
        .await;
    server.listen();
}
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## Contact

For any inquiries, please reach out to the author at [ltpp-universe <root@ltpp.vip>](mailto:root@ltpp.vip).
