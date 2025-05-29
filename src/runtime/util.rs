use std::{cell::RefCell, task::Poll};

use crossbeam::channel::{self, Receiver, Sender};

pub struct UnblockFuture<F: FnOnce() -> T + Send + 'static, T: Send + 'static> {
    tx: Sender<T>,
    rx: Receiver<T>,
    func: RefCell<Option<F>>,
}

impl<F: FnOnce() -> T + Send + 'static, T: Send + 'static> Future for UnblockFuture<F, T> {
    type Output = T;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if self.rx.is_empty() {
            let tx = self.tx.clone();
            let func = self.func.take().unwrap();
            let waker = cx.waker().clone();
            rayon::spawn(move || {
                let result = func();
                tx.send(result).unwrap();

                waker.wake_by_ref();
            });

            Poll::Pending
        } else {
            let result = self.rx.recv().unwrap();
            Poll::Ready(result)
        }
    }
}

pub fn unblock<F, T>(func: F) -> UnblockFuture<F, T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = channel::bounded(1);
    let func = RefCell::new(Some(func));

    UnblockFuture { tx, rx, func }
}
