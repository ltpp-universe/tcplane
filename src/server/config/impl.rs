use super::r#type::ServerConfig;
use crate::*;
use http_type::*;

impl Default for ServerConfig {
    #[inline]
    fn default() -> Self {
        Self {
            host: DEFAULT_HOST.to_owned(),
            port: DEFAULT_WEB_PORT,
            log_dir: DEFAULT_LOG_DIR.to_owned(),
            log_size: DEFAULT_LOG_FILE_SIZE,
            buffer_size: DEFAULT_BUFFER_SIZE,
            interval_millis: DEFAULT_LOG_INTERVAL_MILLIS,
        }
    }
}
