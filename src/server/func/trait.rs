use crate::*;

pub trait AsyncFunc:
    Fn(ArcRwLockControllerData) -> Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>
    + Send
    + Sync
    + 'static
{
}

pub trait AsyncFuncWithoutPin<Fut>:
    Fn(ArcRwLockControllerData) -> Fut + Send + Sync + 'static
where
    Fut: Future<Output = ()> + Send + 'static,
{
}

pub trait Func: Fn(ArcRwLockControllerData) + Send + Sync + 'static {}
