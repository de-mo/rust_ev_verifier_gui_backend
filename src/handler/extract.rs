use super::{get_status_response, update_status};
use crate::{
    app_data::{AppData, AppDataLockArc, AppStatus, InputFileLocation},
    response::StatusResponse,
    AppError,
};
use anyhow::{anyhow, Context};
use axum::{extract::State, Json};
use rust_ev_verifier_lib::{
    application_runner::ExtractDataSetResults, verification::VerificationPeriod, Config,
};
use tracing::{error, info, instrument};

#[instrument(skip(state, config))]
async fn extract_fn(
    state: AppDataLockArc,
    period: VerificationPeriod,
    file_location: InputFileLocation,
    password: String,
    config: &'static Config,
) -> Result<(), anyhow::Error> {
    info!("Extraction started");
    let extracted = ExtractDataSetResults::extract_datasets(
        period,
        file_location.context_zip_file.unwrap().as_path(),
        file_location.setup_zip_file.as_deref(),
        file_location.tally_zip_file.as_deref(),
        &password,
        config,
    )
    .context("Problem with extraction")
    .or_else(|e| {
        error!("{:?}", e);
        Err(e)
    })?;
    info!(
        "Extraction successful in {}",
        extracted.location().to_str().unwrap()
    );
    let mut state_mut: tokio::sync::RwLockWriteGuard<'_, AppData> = state.write().await;
    state_mut.extracted_dataset_result = Some(extracted);
    update_status(&mut state_mut, AppStatus::Extracted);
    Ok(())
}

pub async fn extract_handler(
    State(state): State<AppDataLockArc>,
) -> Result<Json<StatusResponse>, AppError> {
    let mut state_mut: tokio::sync::RwLockWriteGuard<'_, AppData> = state.write().await;
    let status_spawn = state.clone();
    let period = state_mut.verfification_period.unwrap().clone();
    let file_location = state_mut.input_file_location.clone();
    let config = state_mut.config;
    let password = dotenvy::var("APP_VERIFIER_DATASET_PASSWORD").map_err(|e| {
        error!(
            "port (APP_VERIFIER_DATASET_PASSWORD) not found in .env {}",
            e
        );
        AppError::from(anyhow!(e))
    })?;
    println!("{}", password);
    tokio::spawn(
        async move { extract_fn(status_spawn, period, file_location, password, config).await },
    );
    update_status(&mut state_mut, AppStatus::Extracting);
    Ok(get_status_response(&state_mut))
}
