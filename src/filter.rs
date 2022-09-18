use hyper::{
    header::{Entry, HeaderName, AUTHORIZATION, HOST},
    http::HeaderValue,
    HeaderMap,
};

#[derive(Clone)]
pub struct Filter<T> {
    _filter: T,
}

impl Filter<HeaderMap> {
    pub fn authorization(headers: &mut HeaderMap) -> HeaderValue {
        Filter::header(headers, AUTHORIZATION).unwrap_or_else(|| HeaderValue::from_static(""))
    }

    pub fn host(headers: &mut HeaderMap) -> HeaderValue {
        Filter::header(headers, HOST).unwrap_or_else(|| HeaderValue::from_static(""))
    }

    pub fn header(headers: &mut HeaderMap, header: HeaderName) -> Option<HeaderValue> {
        match headers.entry(header) {
            Entry::Occupied(entry) => Some(entry.get().to_owned()),
            Entry::Vacant(_) => None,
        }
    }
}
