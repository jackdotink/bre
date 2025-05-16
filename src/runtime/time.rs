use std::{task::Poll, time::Duration};

use super::reactor;

struct SleepFuture {
    started: bool,
    duration: Duration,
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        if !self.started {
            let waker = cx.waker().clone();
            let duration = self.duration;

            reactor::sleep(duration, waker);
            self.started = true;

            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

pub fn sleep(duration: Duration) -> impl Future<Output = ()> {
    SleepFuture {
        started: false,
        duration,
    }
}
