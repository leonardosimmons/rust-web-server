use std::{fmt::Debug, task::Poll};

use futures::Future;
use hyper::{header::Entry, http::HeaderValue, HeaderMap, Request};
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

impl Logging<HeaderMap> {
    /// Returns the specified header entry from the request headers
    fn get_header(headers: &mut HeaderMap, entry: &'static str) -> HeaderValue {
        return match headers.entry(entry) {
            Entry::Occupied(entry) => entry.get().to_owned(),
            Entry::Vacant(_) => HeaderValue::from(0),
        };
    }

    /// Returns the current host/origin of the request
    fn host(headers: &mut HeaderMap) -> HeaderValue {
        Logging::get_header(headers, "host")
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
        let host = Logging::<HeaderMap>::host(&mut headers);
        let method = req.method().clone();
        let route = req.uri().path().to_string();

        tracing::debug!(
            "processing request #{} | origin: {:?} method: {}, route: {}",
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
        let start = Instant::now();
        let res = match this.future.poll(cx) {
            Poll::Ready(res) => res,
            Poll::Pending => return Poll::Pending,
        };
        let duration = start.elapsed();
        let host = Logging::<HeaderMap>::host(&mut this.headers);
        tracing::debug!(
            "request #{} completed in {:?}. | origin: {:?}, method: {}, route: {}",
            this.connection_number,
            duration,
            host,
            this.method,
            this.route
        );
        Poll::Ready(res)
    }
}
