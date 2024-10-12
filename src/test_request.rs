use super::test_helpers::*;
use crate::{
    app_data::{AppStatus, VerificationPeriodDef},
    response::StatusResponse,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use rust_ev_verifier_lib::verification::VerificationPeriod;
use std::path::Path;
use tower::ServiceExt;

const CONTEXT_FILE_ZIP: &str = "./datasets/Dataset-context-NE_20231124_TT05-20240802_1158.zip";
const SETUP_FILE_ZIP: &str = "./datasets/Dataset-setup-NE_20231124_TT05-20240802_1158.zip";
const TALLY_FILE_ZIP: &str = "./datasets/Dataset-tally-NE_20231124_TT05-20240802_1207.zip";

#[tokio::test]
async fn test_health_check() {
    let (data, app) = get_data_app();

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    is_response_ok(&response);
    is_response_json(&response);
    let read_data = data.read().await;
    assert_eq!(read_data.app_status, AppStatus::NotInitialized);
    assert_eq!(read_data.verfification_period, None);
}

#[tokio::test]
async fn test_status() {
    let (_, app) = get_data_app();

    let response = call_status(&app).await;

    is_response_ok(&response);
    is_response_json(&response);

    let json: StatusResponse =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(json.app_status, AppStatus::NotInitialized)
}

#[tokio::test]
async fn test_init() {
    let (data, app) = get_data_app();

    let response = call_init(&app, VerificationPeriodDef::Tally).await;

    is_response_ok(&response);
    is_response_json(&response);

    let json: StatusResponse =
        serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(json.app_status, AppStatus::Initialized);
    assert_eq!(
        json.verfification_period,
        Some(VerificationPeriodDef::Tally)
    );
    let read_data = data.read().await;
    assert_eq!(read_data.app_status, AppStatus::Initialized);
    assert_eq!(
        read_data.verfification_period,
        Some(VerificationPeriod::Tally)
    );
}

#[tokio::test]
async fn test_reset() {
    let (data, app) = get_data_app();

    let _ = call_init(&app, VerificationPeriodDef::Tally).await;

    let response = call_reset(&app).await;

    is_response_ok(&response);
    is_response_json(&response);
    let read_data = data.read().await;
    assert_eq!(read_data.app_status, AppStatus::NotInitialized);
    assert_eq!(read_data.verfification_period, None);
}

#[tokio::test]
async fn test_files_setup() {
    let (data, app) = get_data_app();

    let _ = call_init(&app, VerificationPeriodDef::Setup).await;
    let _ = call_input_context(&app, Path::new(CONTEXT_FILE_ZIP)).await;
    {
        let read_data = data.read().await;
        assert_eq!(read_data.app_status, AppStatus::ContextDataSetLoaded);
    }
    let _ = call_input_period_dataset(&app, Path::new(SETUP_FILE_ZIP)).await;

    let read_data = data.read().await;
    assert_eq!(read_data.app_status, AppStatus::PeriodDataSetLoaded);
    assert_eq!(
        read_data
            .input_file_location
            .context_zip_file
            .as_ref()
            .unwrap()
            .to_str()
            .unwrap(),
        CONTEXT_FILE_ZIP
    );
    assert_eq!(
        read_data
            .input_file_location
            .setup_zip_file
            .as_ref()
            .unwrap()
            .to_str()
            .unwrap(),
        SETUP_FILE_ZIP
    );
    assert!(read_data.input_file_location.tally_zip_file.is_none())
}

#[tokio::test]
async fn test_files_tally() {
    let (data, app) = get_data_app();

    let _ = call_init(&app, VerificationPeriodDef::Tally).await;
    let _ = call_input_context(&app, Path::new(CONTEXT_FILE_ZIP)).await;
    {
        let read_data = data.read().await;
        assert_eq!(read_data.app_status, AppStatus::ContextDataSetLoaded);
    }
    let _ = call_input_period_dataset(&app, Path::new(TALLY_FILE_ZIP)).await;

    let read_data = data.read().await;
    assert_eq!(read_data.app_status, AppStatus::PeriodDataSetLoaded);
    assert_eq!(
        read_data
            .input_file_location
            .context_zip_file
            .as_ref()
            .unwrap()
            .to_str()
            .unwrap(),
        CONTEXT_FILE_ZIP
    );
    assert_eq!(
        read_data
            .input_file_location
            .tally_zip_file
            .as_ref()
            .unwrap()
            .to_str()
            .unwrap(),
        TALLY_FILE_ZIP
    );
    assert!(read_data.input_file_location.setup_zip_file.is_none())
}

#[tokio::test]
async fn test_files_error() {
    let (_, app) = get_data_app();

    let _ = call_init(&app, VerificationPeriodDef::Tally).await;
    let response = call_input_context(&app, Path::new("./toto")).await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let _ = call_reset(&app);
    let _ = call_init(&app, VerificationPeriodDef::Tally).await;
    let _ = call_input_context(&app, Path::new(CONTEXT_FILE_ZIP)).await;
    let response = call_input_period_dataset(&app, Path::new("./toto")).await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_extract() {
    let (data, app) = get_data_app();

    let _ = call_init(&app, VerificationPeriodDef::Tally).await;
    let _ = call_input_context(&app, Path::new(CONTEXT_FILE_ZIP)).await;
    let _ = call_input_period_dataset(&app, Path::new(TALLY_FILE_ZIP)).await;

    let response = call_extract(&app).await;

    {
        assert_eq!(response.status(), StatusCode::OK);
        let read_data = data.read().await;
        assert_eq!(read_data.app_status, AppStatus::Extracting);
    }

    loop {
        let _ = tokio::time::sleep(tokio::time::Duration::from_millis(1000));
        let _ = call_status(&app).await;
        {
            let read_data = data.read().await;
            if read_data.app_status == AppStatus::Extracted {
                break;
            }
        }
    }

    let read_data = data.read().await;
    assert!(read_data.extracted_dataset_result.is_some());
}
