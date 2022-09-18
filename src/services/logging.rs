use crate::filter::Filter;

use std::{fmt::Debug, task::Poll};

use futures::Future;
use hyper::{HeaderMap, Request};
use pin_project::pin_project;
use tokio::time::Instant;
use tower::Service;

#[derive(Clone)]
pub struct Logging<S> {
    connection_number: usize,
    inner: S,
}

impl<S> Logging<S> {
    pub fn new(inner: S, num: usize) -> Self {
        Self {
            connection_number: num,
            inner,
        }
    }
}

impl<S, B> Service<Request<B>> for Logging<S>
where
    S: Service<Request<B>> + Debug,
    B: Debug,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = LoggingFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let conn = self.connection_number.clone();
        let mut headers = req.headers().clone();
        let method = req.method().clone();
        let route = req.uri().path().to_string();
        let host = Filter::<HeaderMap>::host(&mut headers);

        tracing::debug!(
            "processing request #{} | host: {:?} method: {}, route: {}",
            conn,
            host,
            method,
            route
        );
        LoggingFuture {
            future: self.inner.call(req),
            connection_number: conn,
            headers,
            method,
            route,
        }
    }
}

#[pin_project]
pub struct LoggingFuture<F> {
    #[pin]
    future: F,
    connection_number: usize,
    headers: HeaderMap,
    method: hyper::Method,
    route: String,
}

impl<F> Future for LoggingFuture<F>
where
    F: Future,
{
    type Output = F::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let mut this = self.project();
        let host = Filter::<HeaderMap>::host(&mut this.headers);

        let start = Instant::now();
        let res = match this.future.poll(cx) {
            Poll::Ready(res) => res,
            Poll::Pending => return Poll::Pending,
        };
        let duration = start.elapsed();

        tracing::debug!(
            "request #{} completed in {:?}. | host: {:?}, method: {}, route: {}",
            this.connection_number,
            duration,
            host,
            this.method,
            this.route
        );
        Poll::Ready(res)
    }
}
