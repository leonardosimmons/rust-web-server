use std::{error::Error, fmt::Display};

use hyper::{
    header::{Entry, HeaderName, HOST},
    http::HeaderValue,
    HeaderMap,
};

#[derive(Debug)]
enum FilterErrorKind {
    NotFound,
}

#[derive(Debug)]
pub struct FilterError {
    _kind: FilterErrorKind,
}

#[derive(Clone)]
pub struct Filter<T> {
    _filter: T,
}

impl Filter<HeaderMap> {
    pub fn header(headers: &mut HeaderMap, header: HeaderName) -> Result<HeaderValue, FilterError> {
        let name = header.clone();
        match headers.entry(header) {
            Entry::Occupied(entry) => Ok(entry.get().to_owned()),
            Entry::Vacant(_) => {
                tracing::error!("'{}' header not found", name);
                Err(FilterError {
                    _kind: FilterErrorKind::NotFound,
                })
            }
        }
    }

    pub fn host(headers: &mut HeaderMap) -> HeaderValue {
        Filter::<HeaderMap>::header(headers, HOST).unwrap_or_else(|_err| {
            tracing::error!("{} header not found", HOST);
            HeaderValue::from_static("unknown")
        })
    }
}

impl Display for FilterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self._kind {
            FilterErrorKind::NotFound => write!(f, "filter not found"),
        }
    }
}

impl Error for FilterError {}
