use std::{
    convert::Infallible,
    path::PathBuf,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures_util::{Future, FutureExt};
use http::{Request, Response, StatusCode};
use http_body::{combinators::UnsyncBoxBody, Body, Empty};
use tower::Service;
use tower_http::services::fs::{
    DefaultServeDirFallback, ServeDir, ServeFileSystemResponseBody as ResponseBody,
};

#[derive(Clone)]
pub struct CremeDevService<F = DefaultServeDirFallback> {
    asset_service: ServeDir<F>,
    public_service: ServeDir<F>,
}

impl CremeDevService {
    pub fn new(assets_dir: PathBuf, public_dir: PathBuf) -> Self {
        Self {
            asset_service: ServeDir::new(assets_dir),
            public_service: ServeDir::new(public_dir),
        }
    }

    // TODO: This is a bit of a hack, requiring a clone.
    // We can downcast the fallback service to get around this eventually.
    pub fn fallback<F2>(self, new_fallback: F2) -> CremeDevService<F2>
    where
        F2: Clone,
    {
        CremeDevService {
            asset_service: self.asset_service.fallback(new_fallback.clone()),
            public_service: self.public_service.fallback(new_fallback),
        }
    }
}

impl<ReqBody, F, FResBody> Service<Request<ReqBody>> for CremeDevService<F>
where
    F: Service<Request<ReqBody>, Response = Response<FResBody>, Error = Infallible>
        + Clone
        + Send
        + 'static,
    F::Future: Send + 'static,
    ReqBody: Send + 'static,
    FResBody: http_body::Body<Data = Bytes> + Send + 'static,
    FResBody::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = Response<UnsyncBoxBody<Bytes, std::io::Error>>;
    type Error = Infallible;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let asset_ready = self.asset_service.poll_ready(cx);
        let public_ready = self.public_service.poll_ready(cx);

        if asset_ready.is_ready() && public_ready.is_ready() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        if req.uri().path().starts_with("/assets") {
            let req = Request::builder()
                .uri(
                    req.uri()
                        .path_and_query()
                        .unwrap()
                        .as_str()
                        .strip_prefix("/assets")
                        .unwrap(),
                )
                .body(req.into_body())
                .unwrap();

            self.asset_service.try_call(req)
        } else {
            self.public_service.try_call(req)
        }
        .map(
            |result: Result<Response<ResponseBody>, std::io::Error>| -> Result<Self::Response, Infallible> {
                let response = result
                    .map(|response| Response::new(response.boxed_unsync()))
                    .unwrap_or_else(|_err| {
                        let body = Empty::new().map_err(|err| match err {}).boxed_unsync();
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(body)
                            .unwrap()
                    });
                Ok(response)
            })
        .boxed()
    }
}
