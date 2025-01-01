use crate::server::controller_data::r#type::ControllerData;
use http_type::ArcRwLock;

pub type Func = dyn Fn(&mut ControllerData) + Send + Sync + 'static;
pub type FuncBox = Box<Func>;
pub type FuncArcLock = ArcRwLock<Box<dyn Fn(&mut ControllerData) + Send + Sync>>;