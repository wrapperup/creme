use std::{convert::Infallible, path::PathBuf, task::{Poll, Context}};

use bytes::Bytes;
use http::{Request, Response};
use tower::Service;
use tower_http::services::fs::DefaultServeDirFallback;
pub use tower_http::services::fs::ServeDir;

pub struct CremeDevService<S> {
    inner: S,
    pub assets_dir: PathBuf,
    pub public_dir: PathBuf,
}

impl<S> CremeDevService<S> {
    pub fn new(inner: S, assets_dir: PathBuf, public_dir: PathBuf) -> Self {
        Self {
            inner,
            assets_dir,
            public_dir,
        }
    }
}

impl<S, Request> Service<Request> for CremeDevService<S>
where
    S: Service<Request>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Our middleware doesn't care about backpressure so its ready as long
        // as the inner service is ready.
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        self.inner.call(request)
    }
}
