use std::{convert::Infallible, task::Poll};

use futures::future::{ready, Ready};
use hyper::{Body, Request, Response};
use tower::Service;

pub mod logging;

#[derive(Debug)]
pub struct HelloWorld;

impl Service<Request<Body>> for HelloWorld {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: Request<Body>) -> Self::Future {
        ready(Ok(Response::new(Body::from(
            "Hello, welcome to the rust web server",
        ))))
    }
}
