use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use macros::my_ready;

#[tokio::main]
async fn main() {
    // let mut cx = Context::from_waker(futures::task::noop_waker_ref());
    // let ret = poll_fut(&mut cx);

    let fut = MyFut::new(42);
    println!("result: {:?}", fut.await);
}

#[allow(unused)]
fn poll_fut(cx: &mut Context<'_>) -> Poll<usize> {
    // let mut fut = future::ready(42);
    let mut fut = MyFut::new(42);
    let fut = Pin::new(&mut fut);
    my_ready!(fut.poll(cx))
}

struct MyFut {
    polled: bool,
    v: usize,
}

impl MyFut {
    fn new(v: usize) -> Self {
        MyFut { polled: false, v }
    }
}

impl Future for MyFut {
    type Output = usize;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.polled {
            Poll::Ready(self.v)
        } else {
            self.polled = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}