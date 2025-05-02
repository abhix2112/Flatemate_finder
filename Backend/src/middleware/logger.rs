use std::task::{Context, Poll};
use tower::{Layer, Service};
use hyper::{Request, Response};
use axum::body::Body;
use std::pin::Pin;
use std::future::Future;
use tracing::info;

#[derive(Clone, Copy)]
pub struct LoggerLayer;

impl<S> Layer<S> for LoggerLayer {
    type Service = LoggerMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        LoggerMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct LoggerMiddleware<S> {
    inner: S,
}

impl<S, ReqBody> Service<Request<ReqBody>> for LoggerMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,

    S::Error: std::fmt::Debug,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let method = req.method().clone();
        let uri = req.uri().clone();
        let fut = self.inner.call(req);

        Box::pin(async move {
            let response = fut.await?;
            info!("{} {} => {}", method, uri, response.status());
            Ok(response)
        })
    }
}