use crate::*;

impl<F> Func for F where F: Fn(ArcRwLockControllerData) + Send + Sync + 'static {}

impl<F, Fut> AsyncFuncWithoutPin<Fut> for F
where
    F: Fn(ArcRwLockControllerData) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + Sync + 'static,
{
}
