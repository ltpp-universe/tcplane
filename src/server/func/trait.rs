use crate::*;

pub trait AsyncFunc:
    Fn(&mut ControllerData) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
    + Send
    + Sync
    + 'static
{
}

pub trait AsyncFuncWithoutPin<Fut>: Fn(&mut ControllerData) -> Fut + Send + Sync + 'static
where
    Fut: Future<Output = ()> + Send + 'static,
{
}

pub trait Func: Fn(&mut ControllerData) + Send + Sync + 'static {}
