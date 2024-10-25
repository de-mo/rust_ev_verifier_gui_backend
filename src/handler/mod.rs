mod extract;
mod run;
mod send_file;

pub use extract::extract_handler;
pub use run::run_handler;
pub use send_file::{context_dataset_handler, period_dataset_handler};

use crate::{
    app_data::{AppData, AppDataLockArc, AppStatus},
    request::InitRequest,
    response::StatusResponse,
};
use axum::{extract::State, Json};
use rust_ev_verifier_lib::verification::VerificationPeriod;
use tracing::{error, info};

pub async fn health_check_handler() -> Json<String> {
    Json("Server is living".to_string())
}

fn get_status_response(app_data: &AppData) -> Json<StatusResponse> {
    Json(StatusResponse::from(app_data))
}

fn update_status(data_mut: &mut AppData, status: AppStatus) {
    data_mut.app_status = status;
    info!("Status set to {}", status.as_ref());
}

fn update_with_error(data_mut: &mut AppData, status: AppStatus, error: &str) {
    error!("{}", error);
    data_mut.error = Some(error.to_string());
    update_status(data_mut, status);
}

pub async fn status_handler(State(state): State<AppDataLockArc>) -> Json<StatusResponse> {
    let state_read = state.read().await;
    get_status_response(&state_read)
}

pub async fn init_handler(
    State(state): State<AppDataLockArc>,
    Json(payload): Json<InitRequest>,
) -> Json<StatusResponse> {
    let mut state_mut = state.write().await;
    state_mut.verfification_period = Some(VerificationPeriod::from(&payload.period));
    info!("Verification period set to {}", payload.period.as_ref());
    update_status(&mut state_mut, AppStatus::Initialized);
    get_status_response(&state_mut)
}

pub async fn reset_handler(State(state): State<AppDataLockArc>) -> Json<StatusResponse> {
    let mut state_mut = state.write().await;
    *state_mut = AppData::default();
    info!("Application reseted");
    get_status_response(&state_mut)
}

pub async fn manual_checks_handler() {
    todo!()
}
