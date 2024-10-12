use crate::{
    app_data::{AppDataLockArc, AppStatus},
    response::response_error_with_status,
    router::{RoutePath, ALLOWED_ROUTE_PATHES},
};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::str::FromStr;

fn validate_uri_with_status(path: &RoutePath, status: &AppStatus) -> String {
    match ALLOWED_ROUTE_PATHES.iter().find(|(s, _)| s == status) {
        Some((_, ps)) => match ps.contains(path) {
            true => String::default(),
            false => format!(
                "Path {} not allowed for status {}",
                path.as_ref(),
                status.as_ref()
            ),
        },
        None => format!("Status {} not allowed", status.as_ref()),
    }
}

pub async fn check_status_middelware(
    State(state): State<AppDataLockArc>,
    // you can add more extractors here but the last
    // extractor must implement `FromRequest` which
    // `Request` does
    request: Request,
    next: Next,
) -> Response {
    {
        let path = request.uri().path();
        let status = &state.read().await.app_status;
        let path_enum = match RoutePath::from_str(path) {
            Ok(p) => p,
            Err(_) => {
                return response_error_with_status(
                    StatusCode::BAD_REQUEST,
                    &format!("Path {} not found", path),
                )
                .into_response()
            }
        };
        let validation_result = validate_uri_with_status(&path_enum, status);
        if !validation_result.is_empty() {
            return response_error_with_status(StatusCode::BAD_REQUEST, &validation_result)
                .into_response();
        }
    }
    next.run(request).await
}
