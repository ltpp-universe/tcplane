use hyperlane_log::*;
use lombok_macros::*;
use std::{net::TcpStream, sync::Arc};

pub type ControllerDataStream = Arc<TcpStream>;
pub type ControllerDataStreamOpt = Option<ControllerDataStream>;
pub type ControllerDataRequest = Vec<u8>;
pub type ControllerDataRequestOpt = Option<ControllerDataRequest>;
pub type ControllerDataResponse = Vec<u8>;
pub type ControllerDataResponseOpt = Option<ControllerDataResponse>;

#[derive(Clone, Debug, Lombok)]
pub struct ControllerData {
    pub(super) stream: ControllerDataStreamOpt,
    pub(super) request: ControllerDataRequestOpt,
    pub(super) response: ControllerDataResponseOpt,
    pub(super) log: Log,
}