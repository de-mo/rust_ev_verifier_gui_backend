use super::{get_status_response, update_status};
use crate::{
    app_data::{AppData, AppDataLockArc, AppStatus},
    request::FilePathRequest,
    response::StatusResponse,
    AppError,
};
use anyhow::anyhow;
use axum::{extract::State, Json};
use rust_ev_verifier_lib::verification::VerificationPeriod;
use tracing::{error, info};

pub async fn context_dataset_handler(
    State(state): State<AppDataLockArc>,
    Json(payload): Json<FilePathRequest>,
) -> Result<Json<StatusResponse>, AppError> {
    if !payload.path.exists() {
        let msg = format!("Context file {} not exist", payload.path.to_str().unwrap());
        error!(msg);
        return Err(AppError::from(anyhow!(msg)));
    }
    let mut state_mut = state.write().await;
    state_mut.input_file_location.context_zip_file = Some(payload.path.clone());
    info!(
        "Context input dataset set to {}",
        payload.path.to_str().unwrap()
    );
    update_status(&mut state_mut, AppStatus::ContextDataSetLoaded);
    Ok(get_status_response(&state_mut))
}

pub async fn period_dataset_handler(
    State(state): State<AppDataLockArc>,
    Json(payload): Json<FilePathRequest>,
) -> Result<Json<StatusResponse>, AppError> {
    if !payload.path.exists() {
        let msg = format!(
            "Period dataset file {} not exist",
            payload.path.to_str().unwrap()
        );
        error!(msg);
        return Err(AppError::from(anyhow!(msg)));
    }
    let mut state_mut: tokio::sync::RwLockWriteGuard<'_, AppData> = state.write().await;
    match state_mut.verfification_period.unwrap() {
        VerificationPeriod::Setup => {
            state_mut.input_file_location.setup_zip_file = Some(payload.path.clone())
        }
        VerificationPeriod::Tally => {
            state_mut.input_file_location.tally_zip_file = Some(payload.path.clone())
        }
    }
    info!(
        "input dataset for {} set to {}",
        state_mut.verfification_period.unwrap().as_ref(),
        payload.path.to_str().unwrap()
    );
    update_status(&mut state_mut, AppStatus::PeriodDataSetLoaded);
    Ok(get_status_response(&state_mut))
}
