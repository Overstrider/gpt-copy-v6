pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod provider;
mod routes;
pub mod state;

use axum::Router;
use axum::http::{HeaderValue, Method, header::CONTENT_TYPE};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use state::AppState;

pub fn create_router(state: AppState) -> Router {
    routes::router(state)
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
