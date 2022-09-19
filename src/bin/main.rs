use rust_web_server::services::{logging::Logging, timeout::Timeout};

use std::{
    convert::Infallible,
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use hyper::{server::conn::Http, service::service_fn, Body, Request, Response};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

const DEFAULT_TIMEOUT: u64 = 5;
const PROGRAM_TRACING_FILTER: &str = "main=trace";
const SYSTEM_TRACING_FILTER: &str = "rust_web_server=trace";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let env_filter = EnvFilter::from_default_env()
        .add_directive(PROGRAM_TRACING_FILTER.parse().unwrap_or_default())
        .add_directive(SYSTEM_TRACING_FILTER.parse().unwrap_or_default());

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .compact()
        .init();

    let connection_counter = Arc::new(AtomicUsize::new(1));

    let addr: SocketAddr = ([127, 0, 0, 1], 5000).into();
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let connection_counter = connection_counter.clone();

        let process = async move {
            let connection_count = connection_counter.fetch_add(1, Ordering::AcqRel);

            let svc = service_fn(hello_world);
            let svc = Timeout::new(svc, Duration::from_secs(DEFAULT_TIMEOUT));
            let svc = Logging::new(svc, connection_count);

            if let Err(err) = Http::new().serve_connection(stream, svc).await {
                tracing::error!("[ request {} ] {}", connection_count, err);
            }
        };

        tokio::task::spawn(process);
    }
}
async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    tokio::time::sleep(Duration::from_secs(1)).await;
    Ok(Response::new("Welcome to the Rust Web Server".into()))
}
