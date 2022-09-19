use std::{fmt::Display, task::Poll, time::Duration};

use futures::Future;
use pin_project::pin_project;
use tokio::time::Sleep;
use tower::{BoxError, Service};

#[derive(Clone, Debug)]
pub struct Timeout<S> {
    inner: S,
    timeout: Duration,
}

impl<S> Timeout<S> {
    pub fn new(inner: S, timeout: Duration) -> Self {
        Self { inner, timeout }
    }
}

impl<S, Req> Service<Req> for Timeout<S>
where
    S: Service<Req>,
    S::Error: Into<BoxError>,
{
    type Response = S::Response;
    type Error = BoxError;
    type Future = TimeoutFuture<S::Future>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        let sleep = tokio::time::sleep(self.timeout);

        TimeoutFuture {
            future: self.inner.call(req),
            sleep,
        }
    }
}

#[pin_project]
pub struct TimeoutFuture<F> {
    #[pin]
    future: F,
    #[pin]
    sleep: Sleep,
}

impl<F, Res, Error> Future for TimeoutFuture<F>
where
    F: Future<Output = Result<Res, Error>>,
    Error: Into<BoxError>,
{
    type Output = Result<Res, BoxError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        match this.future.poll(cx) {
            Poll::Pending => { /* time still remaining */ }
            Poll::Ready(res) => {
                let res = res.map_err(Into::into);
                return Poll::Ready(res);
            }
        };

        match this.sleep.poll(cx) {
            Poll::Pending => { /* time still remaining */ }
            Poll::Ready(()) => {
                let error = Box::new(TimeoutError(()));
                return Poll::Ready(Err(error));
            }
        };

        Poll::Pending
    }
}

#[derive(Debug)]
pub struct TimeoutError(());

impl Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad("timed out")
    }
}

impl std::error::Error for TimeoutError {}
