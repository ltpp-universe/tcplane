use crate::*;
use http_type::*;

pub type ArcTcpStream = Arc<TcpStream>;
pub type OptionArcTcpStream = Option<ArcTcpStream>;
pub type ArcRwLockControllerData = ArcRwLock<ControllerData>;

#[derive(Clone, Debug, Lombok)]
pub struct ControllerData {
    pub(super) stream: OptionArcTcpStream,
    pub(super) request: crate::Request,
    pub(super) response: crate::Response,
    pub(super) log: Log,
}
