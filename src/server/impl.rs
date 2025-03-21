use crate::*;

impl Default for Server {
    #[inline]
    fn default() -> Self {
        Self {
            cfg: Arc::new(RwLock::new(ServerConfig::default())),
            func_list: Arc::new(RwLock::new(vec![])),
            tmp: Arc::new(RwLock::new(Tmp::default())),
        }
    }
}

impl Server {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub async fn host<T>(&mut self, host: T) -> &mut Self
    where
        T: Into<String>,
    {
        {
            let mut cfg: RwLockWriteGuard<'_, ServerConfig> = self.get_cfg().write().await;
            cfg.set_host(host.into());
        }
        self
    }

    #[inline]
    pub async fn port(&mut self, port: usize) -> &mut Self {
        {
            let mut cfg: RwLockWriteGuard<'_, ServerConfig> = self.get_cfg().write().await;
            cfg.set_port(port);
        }
        self
    }

    #[inline]
    pub async fn log_dir<T>(&mut self, log_dir: T) -> &mut Self
    where
        T: Into<String> + Clone,
    {
        {
            let mut cfg: RwLockWriteGuard<'_, ServerConfig> = self.get_cfg().write().await;
            cfg.set_log_dir(log_dir.clone().into());
            let mut tmp: RwLockWriteGuard<'_, Tmp> = self.get_tmp().write().await;
            tmp.log.set_path(log_dir.clone().into());
        }
        self
    }

    #[inline]
    pub async fn log_size(&mut self, log_size: usize) -> &mut Self {
        {
            let mut cfg: RwLockWriteGuard<'_, ServerConfig> = self.get_cfg().write().await;
            cfg.set_log_size(log_size);
            let mut tmp: RwLockWriteGuard<'_, Tmp> = self.get_tmp().write().await;
            tmp.log.set_file_size(log_size);
        }
        self
    }

    #[inline]
    pub async fn log_interval_millis(&mut self, interval_millis: usize) -> &mut Self {
        {
            let mut cfg: RwLockWriteGuard<'_, ServerConfig> = self.get_cfg().write().await;
            cfg.set_interval_millis(interval_millis);
            let mut tmp: RwLockWriteGuard<'_, Tmp> = self.get_tmp().write().await;
            tmp.log.set_interval_millis(interval_millis);
        }
        self
    }

    #[inline]
    pub async fn print(&mut self, print: bool) -> &mut Self {
        {
            let mut cfg: RwLockWriteGuard<'_, ServerConfig> = self.get_cfg().write().await;
            cfg.set_print(print);
        }
        self
    }

    #[inline]
    pub async fn enable_print(&mut self) -> &mut Self {
        self.print(true).await;
        self
    }

    #[inline]
    pub async fn disable_print(&mut self) -> &mut Self {
        self.print(false).await;
        self
    }

    #[inline]
    pub async fn open_print(&mut self, print: bool) -> &mut Self {
        {
            let mut cfg: RwLockWriteGuard<'_, ServerConfig> = self.get_cfg().write().await;
            cfg.set_print(print);
        }
        self
    }

    #[inline]
    pub async fn buffer(&mut self, buffer_size: usize) -> &mut Self {
        {
            let mut cfg: RwLockWriteGuard<'_, ServerConfig> = self.get_cfg().write().await;
            cfg.set_buffer_size(buffer_size);
        }
        self
    }

    #[inline]
    pub async fn func<F, Fut>(&mut self, func: F) -> &mut Self
    where
        F: AsyncFuncWithoutPin<Fut>,
        Fut: Future<Output = ()> + Send + Sync + 'static,
    {
        {
            let mut mut_func_list: RwLockWriteGuard<'_, Vec<BoxFunc>> =
                self.func_list.write().await;
            mut_func_list.push(Box::new(move |controller_data| {
                Box::pin(func(controller_data))
            }));
        }
        self
    }

    #[inline]
    pub(super) async fn handle_stream(cfg: &ServerConfig, stream_lock: ArcRwLockStream) -> Vec<u8> {
        let buffer_size: usize = cfg.get_buffer_size().clone().max(SPLIT_REQUEST_BYTES.len());
        let mut buffer: Vec<u8> = Vec::new();
        let mut tmp_buf: Vec<u8> = vec![0u8; buffer_size];
        let mut stream: RwLockWriteGuard<'_, TcpStream> = stream_lock.get_write_lock().await;
        loop {
            match stream.read(&mut tmp_buf).await {
                Ok(n) => {
                    let old_len: usize = tmp_buf.len();
                    tmp_buf = remove_trailing_zeros(&mut tmp_buf);
                    let new_len: usize = tmp_buf.len();
                    if n == 0 {
                        break;
                    }
                    if old_len != new_len || tmp_buf.ends_with(SPLIT_REQUEST_BYTES) {
                        buffer.extend_from_slice(&tmp_buf[..n - SPLIT_REQUEST_BYTES.len()]);
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

    #[inline]
    pub async fn listen(&mut self) -> &mut Self {
        self.init().await;
        let cfg: ServerConfig = self.get_cfg().read().await.clone();
        let host: String = cfg.get_host().to_owned();
        let port: usize = *cfg.get_port();
        let addr: String = format!("{}{}{}", host, COLON_SPACE_SYMBOL, port);
        let listener_res: Result<TcpListener, ServerError> = TcpListener::bind(&addr)
            .await
            .map_err(|e| ServerError::TcpBindError(e.to_string()));
        if let Err(err) = listener_res {
            {
                let tmp: RwLockReadGuard<'_, Tmp> = self.get_tmp().read().await;
                tmp.get_log().error(err.to_string(), common_log);
            }
            return self;
        }
        let tcp_listener: TcpListener = listener_res.unwrap();
        while let Ok((stream, _)) = tcp_listener.accept().await {
            let tmp_arc_lock: ArcRwLockTmp = Arc::clone(&self.tmp);
            let stream_lock: ArcRwLockStream = ArcRwLockStream::from_stream(stream);
            let func_list_arc_lock: ArcRwlockVecBoxFunc = Arc::clone(&self.get_func_list());
            let cfg_arc_lock: ArcRwLockServerConfig = Arc::clone(&self.get_cfg());
            let handle_request = move || async move {
                let cfg: ServerConfig = cfg_arc_lock.read().await.clone();
                let request: Vec<u8> = Self::handle_stream(&cfg, stream_lock.clone()).await;
                let log: Log = tmp_arc_lock.read().await.get_log().clone();
                let mut controller_data: InnerControllerData = InnerControllerData::new();
                controller_data
                    .set_stream(Some(stream_lock.clone()))
                    .set_request(request)
                    .set_log(log);
                let controller_data: ControllerData =
                    ControllerData::from_controller_data(controller_data);
                for func in func_list_arc_lock.read().await.iter() {
                    func(controller_data.clone()).await;
                }
            };
            tokio::spawn(handle_request());
        }
        self
    }

    #[inline]
    async fn init_panic_hook(&self) {
        let tmp: Tmp = self.tmp.read().await.clone();
        let print: bool = self.get_cfg().read().await.get_print().clone();
        set_hook(Box::new(move |err| {
            let err_msg: String = format!("{}", err);
            if print {
                println_error!(err_msg);
            }
            handle_error(&tmp, err_msg.clone());
        }));
    }

    #[inline]
    async fn init_log(&self) {
        let tmp: RwLockReadGuard<'_, Tmp> = self.get_tmp().read().await;
        log_run(tmp.get_log());
    }

    #[inline]
    async fn init(&self) {
        self.init_panic_hook().await;
        self.init_log().await;
    }
}
