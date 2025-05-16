use std::{
    collections::BTreeMap,
    sync::OnceLock,
    task::Waker,
    time::{Duration, Instant},
};

use crossbeam::{
    queue::SegQueue,
    sync::{Parker, Unparker},
};

static UNPARKER: OnceLock<Unparker> = OnceLock::new();

static TIMER: SegQueue<(Instant, Waker)> = SegQueue::new();

pub fn reactor() {
    let parker = Parker::new();

    UNPARKER
        .set(parker.unparker().clone())
        .expect("unparker already set");

    let mut timers = BTreeMap::new();

    loop {
        let now = Instant::now();

        while let Some((when, waker)) = TIMER.pop() {
            assert!(timers.insert(when, waker).is_none(), "duplicate timer");
        }

        let pending = timers.split_off(&(now + Duration::from_nanos(1)));
        let ready = std::mem::replace(&mut timers, pending);

        for (_, waker) in ready {
            waker.wake();
        }

        let dur = timers
            .keys()
            .next()
            .map(|&when| when.saturating_duration_since(now));

        if let Some(dur) = dur {
            parker.park_timeout(dur);
        } else {
            parker.park();
        }
    }
}

pub fn sleep(dur: Duration, waker: Waker) {
    TIMER.push((Instant::now() + dur, waker));

    if let Some(unparker) = UNPARKER.get() {
        unparker.unpark();
    }
}
