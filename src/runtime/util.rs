use std::{
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};

use crossbeam::channel::{self, Receiver};

pub struct UnblockFuture<T: Send> {
    rx: Receiver<T>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl<T: Send> Future for UnblockFuture<T> {
    type Output = T;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if self.rx.is_empty() {
            let mut waker_slot = self.waker.lock().unwrap();
            *waker_slot = Some(cx.waker().clone());

            Poll::Pending
        } else {
            let result = self.rx.recv().unwrap();
            Poll::Ready(result)
        }
    }
}

pub fn unblock<F, T>(func: F) -> UnblockFuture<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = channel::bounded(1);
    let waker = Arc::new(Mutex::new(None::<Waker>));

    let thread_waker = waker.clone();
    rayon::spawn(move || {
        let result = func();
        tx.send(result).unwrap();

        let waker = thread_waker.lock().unwrap();
        if let Some(waker) = &*waker {
            waker.wake_by_ref();
        }
    });

    UnblockFuture { rx, waker }
}
