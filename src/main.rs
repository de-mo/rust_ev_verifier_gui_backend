mod app_data;
mod handler;
mod middlewares;
pub mod request;
pub mod response;
mod router;
mod subscriber;

#[cfg(test)]
mod test_request;

use anyhow::anyhow;
use app_data::{AppData, AppDataLockArc};
use axum::{
    http::StatusCode,
    middleware::{self},
    response::{IntoResponse, Response},
    Router,
};
use lazy_static::lazy_static;
use middlewares::check_status_middelware;
use router::routes;
use rust_ev_verifier_lib::Config as VerifierConfig;
use subscriber::init_subscriber;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};

lazy_static! {
    static ref CONFIG: VerifierConfig = VerifierConfig::new(".");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv().map_err(|e| {
        let error = anyhow!(format!("Error reading .env file: {e}"));
        error
    })?;

    let _guards = init_subscriber(&CONFIG);

    info!(
        "Starting the backend of the Verifier GUI (Version: {})",
        env!("CARGO_PKG_VERSION")
    );

    let shared_app_data: AppDataLockArc = AppData::new();

    let port = dotenvy::var("APP_PORT").map_err(|e| {
        error!("port (APP_PORT) not found in .env {}", e);
        anyhow!(e)
    })?;

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();
    debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app(shared_app_data))
        .await
        .map_err(|e| {
            let error = anyhow!(format!("Error in serve: {}", e));
            error
        })
}

pub fn app(shared_app_data: AppDataLockArc) -> Router {
    routes()
        .route_layer(middleware::from_fn_with_state(
            shared_app_data.clone(),
            check_status_middelware,
        ))
        .layer(TraceLayer::new_for_http())
        .with_state(shared_app_data)
}

// Make our own error that wraps `anyhow::Error`.
pub struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[cfg(test)]
mod test_helpers {
    use super::*;
    use app_data::VerificationPeriodDef;
    use axum::{
        body::Body,
        http::{self, Request, Response, StatusCode},
    };
    use std::path::Path;
    use tower::ServiceExt;

    pub fn get_data_app() -> (AppDataLockArc, Router) {
        let shared_app_data: AppDataLockArc = AppData::new();
        (shared_app_data.clone(), app(shared_app_data.clone()))
    }

    pub fn is_response_ok<T>(response: &Response<T>) {
        assert_eq!(response.status(), StatusCode::OK);
    }

    pub fn is_response_json<T>(response: &Response<T>) {
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );
    }

    pub async fn call_status(app: &Router) -> Response<Body> {
        app.clone()
            .oneshot(
                Request::builder()
                    .uri("/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    pub async fn call_init(app: &Router, period: VerificationPeriodDef) -> Response<Body> {
        let body = format!("{{\"period\": \"{}\"}}", period.as_ref());
        app.clone()
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/init")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    pub async fn call_reset(app: &Router) -> Response<Body> {
        app.clone()
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/reset")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    pub async fn call_extract(app: &Router) -> Response<Body> {
        app.clone()
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/extract")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    pub async fn call_input_file(app: &Router, path: &Path, uri_path: &str) -> Response<Body> {
        let body = format!("{{\"path\": \"{}\"}}", path.to_str().unwrap());
        app.clone()
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri(uri_path)
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    pub async fn call_input_context(app: &Router, path: &Path) -> Response<Body> {
        call_input_file(app, path, "/context-dataset").await
    }

    pub async fn call_input_period_dataset(app: &Router, path: &Path) -> Response<Body> {
        call_input_file(app, path, "/period-dataset").await
    }
}
