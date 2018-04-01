extern crate futures as new;
#[macro_use(task_local)]
extern crate old_futures as old;
#[macro_use]
extern crate lazy_static;

use std::{
    cell::RefCell,
    sync::Arc,
    thread
};

pub struct OldFuture<F>(pub F);

impl<F, I, E> new::Future for OldFuture<F>
where
    F: old::Future<Item = I, Error = E>
{
    type Item = I;
    type Error = E;

    fn poll(
        &mut self, 
        _cx: &mut new::task::Context
    ) -> Result<new::Async<I>, E> {
        match self.0.poll() {
            Ok(old::Async::Ready(val)) => Ok(new::Async::Ready(val)),
            Ok(old::Async::NotReady) => Ok(new::Async::Pending),
            Err(err) => Err(err)
        }
    }
}

pub struct NewFuture<F>(pub F);

impl<F, I, E> old::Future for NewFuture<F>
where
    F: new::Future<Item = I, Error = E>
{
    type Item = I;
    type Error = E;

    fn poll(&mut self) -> Result<old::Async<I>, E> {
        use new::{
            executor::ThreadPool,
            task::{Waker, LocalMap},
            future::empty,
        };

        task_local! {
            static MAP: RefCell<LocalMap> = RefCell::new(LocalMap::new())
        }

        task_local! {
            static WAKER: Waker = OldWaker::new()
        }

        lazy_static! {
            static ref EXECUTOR: ThreadPool = {
                let pool = ThreadPool::new();

                {
                    thread::spawn(move || {
                        ThreadPool::new().run(empty::<(), ()>()).unwrap();
                    });
                }

                pool
            };
        }

        let res = MAP.with(|map| {
            WAKER.with(|waker| {
                let map = &mut *map.borrow_mut();
                let executor = &mut EXECUTOR.clone();
                let mut cx = new::task::Context::new(
                    map,
                    waker,
                    executor
                );

                self.0.poll(&mut cx)
            })
        });


        match res {
            Ok(new::Async::Ready(val)) => Ok(old::Async::Ready(val)),
            Ok(new::Async::Pending) => Ok(old::Async::NotReady),
            Err(err) => Err(err)
        }
    }
}

#[derive(Clone)]
struct OldWaker(old::task::Task);

impl OldWaker {
    fn new() -> new::task::Waker {
        let task = old::task::current();
        let waker = Arc::new(OldWaker(task));

        new::task::Waker::from(waker)
    }
}

impl new::task::Wake for OldWaker {
    fn wake(arc_self: &Arc<Self>) {
        arc_self.0.notify();
    }
}