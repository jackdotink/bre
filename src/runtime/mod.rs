use std::{
    cell::{Cell, RefCell},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};

use crossbeam::channel::{self, Receiver, Sender};

mod reactor;

pub mod fs;
pub mod time;
pub mod util;

pub struct Executor {
    queue: Receiver<Arc<Task>>,
    sender: Sender<Arc<Task>>,
    pending: Cell<usize>,
}

impl Default for Executor {
    fn default() -> Self {
        std::thread::spawn(reactor::reactor);

        let (tx, rx) = channel::unbounded();

        Self {
            queue: rx,
            sender: tx,
            pending: Cell::new(0),
        }
    }
}

impl Executor {
    pub fn spawner(&self) -> Spawner {
        Spawner {
            sender: self.sender.clone(),
            pending: &self.pending,
        }
    }

    pub fn run(&self) {
        if self.pending.get() == 0 {
            return;
        }

        while let Ok(task) = self.queue.recv() {
            if unsafe { task.poll() }.is_ready() {
                self.pending.set(self.pending.get() - 1);

                if self.pending.get() == 0 {
                    return;
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct Spawner<'executor> {
    sender: Sender<Arc<Task>>,
    pending: &'executor Cell<usize>,
}

impl Spawner<'_> {
    pub fn spawn(&self, future: impl Future<Output = ()> + 'static) {
        let task = Arc::new(Task::new(future, self.sender.clone()));

        if unsafe { task.poll() }.is_pending() {
            self.pending.set(self.pending.get() + 1);
        }
    }

    pub fn defer(&self, future: impl Future<Output = ()> + 'static) {
        let task = Arc::new(Task::new(future, self.sender.clone()));

        self.pending.set(self.pending.get() + 1);
        self.sender.send(task).unwrap();
    }
}

struct Task {
    future: RefCell<Pin<Box<dyn Future<Output = ()> + 'static>>>,
    sender: Sender<Arc<Self>>,
}

unsafe impl Send for Task {}
unsafe impl Sync for Task {}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static, sender: Sender<Arc<Self>>) -> Self {
        Self {
            future: RefCell::new(Box::pin(future)),
            sender,
        }
    }

    pub unsafe fn poll(self: Arc<Self>) -> Poll<()> {
        let waker = Waker::from(Arc::clone(&self));
        let cx = &mut Context::from_waker(&waker);

        self.future.borrow_mut().as_mut().poll(cx)
    }
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.sender.send(self.clone()).unwrap();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.sender.send(self.clone()).unwrap();
    }
}
