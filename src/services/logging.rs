use crate::filter::Filter;

use std::task::Poll;

use futures::Future;
use hyper::{header::HOST, http::HeaderValue, HeaderMap, Request, Response};
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
    S: Service<Request<B>, Response = Response<B>> + Clone + Send + 'static,
    B: 'static + Send,
    S::Future: 'static + Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = LoggingFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<B>) -> Self::Future {
        let conn = self.connection_number.clone();
        let mut headers = req.headers().clone();
        let method = req.method().clone();
        let route = req.uri().path().to_string();
        let host = Filter::<HeaderMap>::header(&mut headers, HOST).unwrap_or_else(|err| {
            tracing::error!("[ request {} ] {} -> header={}", conn, err, HOST);
            HeaderValue::from_static("unknown")
        });

        tracing::debug!(
            "[ request {} ] processing... | host={:?} method={}, route={}",
            conn,
            host,
            method,
            route
        );

        let start = Instant::now();
        LoggingFuture {
            future: self.inner.call(req),
            connection_number: conn,
            host,
            method,
            route,
            start,
        }
    }
}

#[pin_project]
pub struct LoggingFuture<F> {
    #[pin]
    future: F,
    connection_number: usize,
    host: HeaderValue,
    method: hyper::Method,
    route: String,
    start: Instant,
}

impl<F, B, E> Future for LoggingFuture<F>
where
    F: Future<Output = Result<Response<B>, E>>,
{
    type Output = Result<Response<B>, E>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();

        let res = match this.future.poll(cx) {
            Poll::Ready(res) => res,
            Poll::Pending => return Poll::Pending,
        };
        let duration = this.start.elapsed();

        let status = if let Ok(res) = &res {
            res.status().as_u16()
        } else {
            500
        };

        tracing::debug!(
            "[ request {} ] completed | status={} time={:?} host={:?}, method={}, route={}",
            this.connection_number,
            status,
            duration,
            this.host,
            this.method,
            this.route
        );
        Poll::Ready(res)
    }
}
