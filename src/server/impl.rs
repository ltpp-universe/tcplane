use super::{
    config::r#type::ServerConfig, controller_data::r#type::ControllerData, error::r#type::Error,
    func::r#type::FuncArcLock, middleware::r#type::MiddlewareArcLock, r#type::Server,
    thread_pool::r#type::ThreadPool, tmp::r#type::Tmp,
};
use http_type::*;
use hyperlane_log::*;
use hyperlane_time::*;
use std::io::Read;
use std_macro_extensions::*;

impl Default for Server {
    fn default() -> Self {
        Self {
            cfg: Arc::new(RwLock::new(ServerConfig::default())),
            func: Arc::new(RwLock::new(Box::new(|_| {}))),
            middleware: Arc::new(RwLock::new(vec![])),
            tmp: Arc::new(RwLock::new(Tmp::default())),
        }
    }
}

impl Server {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn host(&mut self, host: &'static str) -> &mut Self {
        let _ = self.get_cfg().write().and_then(|mut cfg| {
            cfg.set_host(host);
            Ok(())
        });
        self
    }

    pub fn port(&mut self, port: usize) -> &mut Self {
        let _ = self.get_cfg().write().and_then(|mut cfg| {
            cfg.set_port(port);
            Ok(())
        });
        self
    }

    pub fn thread_pool_size(&mut self, thread_pool_size: usize) -> &mut Self {
        let _ = self.get_cfg().write().and_then(|mut cfg| {
            cfg.set_thread_pool_size(thread_pool_size);
            Ok(())
        });
        self
    }

    pub fn log_dir(&mut self, log_dir: &'static str) -> &mut Self {
        let _ = self.get_cfg().write().and_then(|mut cfg| {
            cfg.set_log_dir(log_dir);
            Ok(())
        });
        let _ = self.get_tmp().write().and_then(|mut tmp| {
            tmp.log.set_path(log_dir.into());
            Ok(())
        });
        self
    }

    pub fn log_size(&mut self, log_size: usize) -> &mut Self {
        let _ = self.get_cfg().write().and_then(|mut cfg| {
            cfg.set_log_size(log_size);
            Ok(())
        });
        let _ = self.get_tmp().write().and_then(|mut tmp| {
            tmp.log.set_file_size(log_size);
            Ok(())
        });
        self
    }

    pub fn buffer_size(&mut self, buffer_size: usize) -> &mut Self {
        let _ = self.get_cfg().write().and_then(|mut cfg| {
            cfg.set_buffer_size(buffer_size);
            Ok(())
        });
        self
    }

    pub fn func<F>(&mut self, func: F) -> &mut Self
    where
        F: 'static + Send + Sync + Fn(&mut ControllerData),
    {
        if let Ok(mut mut_func) = self.func.write() {
            *mut_func = Box::new(func);
        }
        self
    }

    pub fn middleware<F>(&mut self, func: F) -> &mut Self
    where
        F: 'static + Send + Sync + Fn(&mut ControllerData),
    {
        if let Ok(mut middleware) = self.middleware.write() {
            middleware.push(Box::new(func));
        }
        self
    }

    fn handle_stream(&self, mut stream: &TcpStream) -> Vec<u8> {
        let mut buffer: Vec<u8> = Vec::new();
        let buffer_size: usize = self
            .get_cfg()
            .read()
            .and_then(|data| Ok(data.get_buffer_size().clone()))
            .unwrap_or_default();
        let mut tmp_buf: Vec<u8> = vec![0u8; buffer_size];
        loop {
            match stream.read(&mut tmp_buf) {
                Ok(n) => {
                    if n == 0 {
                        break;
                    }
                    buffer.extend_from_slice(&tmp_buf[..n]);
                }
                _ => {
                    break;
                }
            }
        }
        buffer
    }

    pub fn listen(&mut self) -> &mut Self {
        self.init();
        let mut host: &str = EMPTY_STR;
        let mut port: usize = usize::default();
        let mut thread_pool_size: usize = usize::default();
        let _ = self.get_cfg().read().and_then(|cfg| {
            host = cfg.get_host();
            port = *cfg.get_port();
            thread_pool_size = *cfg.get_thread_pool_size();
            Ok(())
        });
        let addr: String = format!("{}{}{}", host, COLON_SPACE_SYMBOL, port);
        let listener_res: Result<TcpListener, Error> =
            TcpListener::bind(&addr).map_err(|e| Error::TcpBindError(e.to_string()));
        if listener_res.is_err() {
            return self;
        }
        let tcp_listener: TcpListener = listener_res.unwrap();
        let thread_pool: ThreadPool = ThreadPool::new(thread_pool_size);
        for stream_res in tcp_listener.incoming() {
            if stream_res.is_err() {
                continue;
            }
            let stream: TcpStream = stream_res.unwrap();
            let stream_arc: Arc<TcpStream> = Arc::new(stream);
            let middleware_arc: MiddlewareArcLock = Arc::clone(&self.middleware);
            let func: FuncArcLock = Arc::clone(&self.func);
            let tmp_arc: ArcRwLock<Tmp> = Arc::clone(&self.tmp);
            let request: Vec<u8> = self.handle_stream(&stream_arc);
            let thread_pool_func = move || {
                let _ = tmp_arc.write().and_then(|mut tmp| {
                    tmp.add_thread_num();
                    Ok(())
                });
                let log: Log = tmp_arc
                    .read()
                    .and_then(|tmp| Ok(tmp.log.clone()))
                    .unwrap_or_default();
                let thread_result: Result<(), Box<dyn Any + Send>> = catch_unwind(move || {
                    let mut controller_data: ControllerData = ControllerData::new();
                    controller_data
                        .set_stream(Some(stream_arc.clone()))
                        .set_response(Some(vec![]))
                        .set_request(Some(request))
                        .set_log(log);
                    if let Ok(middleware_guard) = middleware_arc.read() {
                        for middleware in middleware_guard.iter() {
                            middleware(&mut controller_data);
                        }
                    }
                    if let Ok(func_guard) = func.read() {
                        func_guard(&mut controller_data);
                    }
                });
                let _ = tmp_arc.write().and_then(|mut tmp| {
                    tmp.sub_thread_num();
                    Ok(())
                });
                if let Err(err) = thread_result {
                    let _ = tmp_arc.read().and_then(|tem| {
                        tem.get_log().log_error(format!("{:?}", err), |data| {
                            format!("{}: {}{}", current_time(), data.to_string(), HTTP_BR)
                        });
                        Ok(())
                    });
                }
            };
            thread_pool.execute(thread_pool_func);
        }
        self
    }

    fn init_log(&self) {
        let _ = self.get_tmp().read().and_then(|tmp| {
            log_run(tmp.get_log());
            Ok(())
        });
    }

    fn init(&self) {
        self.init_log();
    }
}
