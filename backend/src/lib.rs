pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod provider;
mod routes;
pub mod state;

use axum::Router;
use axum::body::Body;
use axum::http::{
    HeaderValue, Method, Request, header::CONTENT_TYPE, header::HOST, header::ORIGIN,
};
use axum::middleware::{self, Next};
use axum::response::Response;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use error::AppError;
use state::AppState;

pub fn create_router(state: AppState) -> Router {
    routes::router(state)
        .layer(middleware::from_fn(validate_local_request))
        .layer(cors_layer())
        .layer(TraceLayer::new_for_http())
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin([
            HeaderValue::from_static("http://localhost:3000"),
            HeaderValue::from_static("http://127.0.0.1:3000"),
        ])
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE])
}

async fn validate_local_request(req: Request<Body>, next: Next) -> Result<Response, AppError> {
    let host = req
        .headers()
        .get(HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    if !host.is_empty() && !is_loopback_host(host) {
        return Err(AppError::bad_request("invalid host header"));
    }

    if method_has_side_effect(req.method()) {
        let origin = req.headers().get(ORIGIN);
        let trusted = origin
            .and_then(|value| value.to_str().ok())
            .map(is_trusted_frontend_origin)
            .unwrap_or(origin.is_none());

        if !trusted {
            return Err(AppError::bad_request("invalid origin header"));
        }
    }

    Ok(next.run(req).await)
}

fn is_loopback_host(host: &str) -> bool {
    let host = host
        .strip_prefix('[')
        .and_then(|value| value.split_once(']').map(|(address, _)| address))
        .unwrap_or_else(|| host.split(':').next().unwrap_or(host));

    matches!(host, "localhost" | "127.0.0.1" | "::1")
}

fn method_has_side_effect(method: &Method) -> bool {
    matches!(
        method,
        &Method::POST | &Method::PUT | &Method::PATCH | &Method::DELETE
    )
}

fn is_trusted_frontend_origin(origin: &str) -> bool {
    matches!(origin, "http://localhost:3000" | "http://127.0.0.1:3000")
}
