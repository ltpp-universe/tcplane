#[test]
fn test_server_basic_usage() {
    use crate::*;

    fn println(data: &str) {
        let binding: String = current_time();
        let mut time_output_builder: OutputBuilder<'_> = OutputBuilder::new();
        let mut text_output_builder: OutputBuilder<'_> = OutputBuilder::new();
        let time_output: Output<'_> = time_output_builder
            .text(&binding)
            .blod(true)
            .bg_color(ColorType::Use(Color::Yellow))
            .color(ColorType::Rgb(255, 255, 255))
            .build();
        let text_output: Output<'_> = text_output_builder
            .text(data)
            .blod(true)
            .bg_color(ColorType::Use(Color::Green))
            .color(ColorType::Rgb(255, 255, 255))
            .endl(true)
            .build();
        OutputListBuilder::new()
            .add(time_output)
            .add(text_output)
            .run();
    }

    fn common_log(log_data: &String) -> String {
        println(&log_data);
        let write_data: String = format!("{}: {}\n", current_time(), log_data);
        write_data.clone()
    }

    fn run_server() {
        let mut server: Server = Server::new();
        server.host("0.0.0.0");
        server.port(80);
        server.thread_pool_size(10);
        server.log_dir("./logs");
        server.log_size(1_024_000);
        server.buffer_size(1_024_000);
        server.middleware(|controller_data| {
            {
                let request: &mut Vec<u8> = controller_data.get_mut_request();
                let mut new_request: Vec<u8> = request.clone();
                let ext: Vec<u8> = "test".as_bytes().to_vec();
                new_request.extend(ext);
                *request = new_request;
            }
            let request: Request = controller_data.get_request().clone();
            let stream: ControllerDataStream = controller_data.get_stream().clone().unwrap();
            let host: String = stream
                .peer_addr()
                .and_then(|host| Ok(host.to_string()))
                .unwrap_or("Unknown".to_owned());
            let res: ResponseResult = controller_data
                .get_response()
                .clone()
                .data("hello world")
                .send(&stream);
            controller_data.get_log().log_debug(
                format!(
                    "Request host => {}\n{:#?}\n{:?}",
                    host,
                    String::from_utf8_lossy(&request),
                    res.unwrap()
                ),
                common_log,
            );
        });

        server.func(|controller_data| {
            let response: &Response = controller_data.get_response();
            controller_data.get_log().log_debug(
                format!(
                    "Response => {:?}\n",
                    String::from_utf8_lossy(response.get_data())
                ),
                common_log,
            );
        });
        server.listen();
    }

    fn main() {
        run_server();
    }

    main();
}
