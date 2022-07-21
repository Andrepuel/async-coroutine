use corosensei::{CoroutineResult, ScopedCoroutine, Yielder};
use std::{cell::Cell, future::Future, pin::Pin, rc::Rc};

pub async fn coroutine_start<U, F>(f: F) -> U
where
    F: for<'a> FnOnce(Awaiter<'a>) -> U,
{
    let mut coroutine = ScopedCoroutine::new(move |yielder, _| f(Awaiter(yielder)));

    loop {
        match coroutine.resume(()) {
            CoroutineResult::Yield(exec) => {
                // SAFETY: The execution may reference values from the coroutine stack,
                // at this point the coroutine is in a yielded state, thus everything
                // in its stack is alive.
                unsafe { exec.exec().await }
            }
            CoroutineResult::Return(ret) => break ret,
        }
    }
}

pub struct Awaiter<'a>(&'a Yielder<(), Execution>);
impl<'a> Awaiter<'a> {
    pub fn await_on<'b, F: Future + 'b>(&self, f: F) -> F::Output {
        let recv = Rc::new(Cell::new(None));
        let send = recv.clone();

        let executor: Pin<Box<dyn Future<Output = ()> + 'b>> = Box::pin(async move {
            send.set(Some(f.await));
        });
        // SAFETY: We are getting rid of the 'b lifetime, but we know that the future
        // will not outlive the current stack due to the suspend() call plus the implementation
        // of coroutine_start.
        let executor: Pin<Box<dyn Future<Output = ()>>> = unsafe { std::mem::transmute(executor) };
        self.0.suspend(Execution(executor));
        recv.take().unwrap()
    }
}
struct Execution(Pin<Box<dyn Future<Output = ()> + 'static>>);
impl Execution {
    async unsafe fn exec(self) {
        self.0.await
    }
}
