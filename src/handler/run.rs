use std::path::PathBuf;

use super::{get_status_response, update_status, update_with_error};
use crate::{
    app_data::{AppData, AppDataLockArc, AppStatus, VerificationStatusEnum},
    response::StatusResponse,
    AppError,
};
use axum::{extract::State, Json};
use rust_ev_verifier_lib::{
    application_runner::{RunParallel, Runner},
    verification::{VerificationMetaDataList, VerificationPeriod},
    Config,
};
use tracing::{debug, info, instrument, trace};

#[instrument(skip(state, config, metada_list))]
async fn run_fn(
    state: AppDataLockArc,
    period: VerificationPeriod,
    extracted_location: PathBuf,
    metada_list: &VerificationMetaDataList,
    config: &'static Config,
) {
    let state_before = state.clone();
    let state_after = state.clone();
    let mut runner = match Runner::new(
        extracted_location.as_path(),
        &period,
        &metada_list,
        &[],
        RunParallel,
        config,
        move |id| {
            trace!("before for {}", id);
            let mut data_mut = futures::executor::block_on(state_before.write());
            data_mut.verification_status.get_mut(id).unwrap().status =
                VerificationStatusEnum::Running;
            trace!("end of before for {}", id);
        },
        move |id, errors, failures| {
            trace!("after for {}", id);
            let mut data_mut = futures::executor::block_on(state_after.write());
            data_mut.set_verification_status(id, errors, failures);
            if !data_mut.not_finished() {
                update_status(&mut data_mut, AppStatus::Finished);
            }
            trace!("end of after for {}", id);
        },
    ) {
        Ok(res) => res,
        Err(e) => {
            let mut state_mut: tokio::sync::RwLockWriteGuard<'_, AppData> = state.write().await;
            update_with_error(
                &mut state_mut,
                AppStatus::RunError,
                format!("Error creating the runner: {:?}", e).as_str(),
            );
            return;
        }
    };
    debug!("Runner created");
    if let Err(e) = runner.run_all(&metada_list) {
        let mut state_mut: tokio::sync::RwLockWriteGuard<'_, AppData> = state.write().await;
        update_with_error(
            &mut state_mut,
            AppStatus::RunError,
            format!("error running the tests: {:?}", e).as_str(),
        );
    }
}

pub async fn run_handler(
    State(state): State<AppDataLockArc>,
) -> Result<Json<StatusResponse>, AppError> {
    let mut state_mut: tokio::sync::RwLockWriteGuard<'_, AppData> = state.write().await;

    let metadata =
        VerificationMetaDataList::load(state_mut.config.get_verification_list_str()).unwrap();

    state_mut.set_with_medata(&metadata);

    info!(
        "Start the verification for period {}",
        state_mut.verfification_period.as_ref().unwrap().as_ref()
    );
    let status_spawn = state.clone();
    let period = state_mut.verfification_period.unwrap().clone();
    let extracted_location = state_mut
        .extracted_dataset_result
        .as_ref()
        .unwrap()
        .location()
        .to_path_buf();
    let config = state_mut.config;
    tokio::spawn(async move {
        run_fn(status_spawn, period, extracted_location, &metadata, config).await
    });
    update_status(&mut state_mut, AppStatus::Running);
    Ok(get_status_response(&state_mut))
}
