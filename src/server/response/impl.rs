use super::error::Error;
use crate::*;
use std::{io::Write, net::TcpStream};

impl Default for Response {
    #[inline]
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl Response {
    #[inline]
    pub fn send(&mut self, mut stream: &TcpStream) -> ResponseResult {
        let send_res: ResponseResult = stream
            .write_all(&self.get_data())
            .and_then(|_| stream.flush())
            .map_err(|err| Error::ResponseError(err.to_string()))
            .and_then(|_| Ok(self.get_data()))
            .cloned();
        send_res
    }

    #[inline]
    pub fn set_data<T: Into<ResponseData>>(&mut self, data: T) -> &mut Self {
        self.data = data.into();
        self
    }
}
