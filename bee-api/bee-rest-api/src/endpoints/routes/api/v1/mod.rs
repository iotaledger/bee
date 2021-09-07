pub mod test;

use axum::{
    handler::get,
    Router,
    response::Html,
    routing::BoxRoute,
    body::Body,
    http::Request,
};
use http::Response;
use tower::service_fn;

pub fn api_routes() -> Router<BoxRoute> {
    Router::new()
        .route("/", get(handler))
        //.nest("/test", test::api_routes())
        .route("/test/", service_fn(|req: Request<Body>| async move {
            let body = Body::from(format!("Hi from `/{}`", req.uri()));
            let body = axum::body::box_body(body);
            let res = Response::new(body);
            Ok(res)
            }))
        .boxed()
}

async fn handler() -> Html<&'static str> {
    Html("<h1>You are in /v1</h1>")
}