use std::{collections::HashMap, path::PathBuf};

use crate::app_data::{
    AppData, AppStatus, InputFileLocation, VerificationInformation, VerificationPeriodDef,
    VerificationStatus,
};
use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

pub fn response_error_with_status(status: StatusCode, message: &str) -> (StatusCode, Json<String>) {
    (status, Json(message.to_string()))
}

pub fn response_anyhow_with_status(
    status: StatusCode,
    error: anyhow::Error,
) -> (StatusCode, Json<String>) {
    response_error_with_status(status, format!("{:?}", error).as_str())
}

#[derive(Serialize, Deserialize)]
pub struct StatusResponse {
    pub app_status: AppStatus,
    pub verfification_period: Option<VerificationPeriodDef>,
    pub input_file_location: InputFileLocation,
    pub location: Option<PathBuf>,
    pub verification_information: HashMap<String, VerificationInformation>,
    pub verification_status: HashMap<String, VerificationStatus>,
}

impl From<&AppData> for StatusResponse {
    fn from(value: &AppData) -> Self {
        Self {
            app_status: value.app_status,
            verfification_period: value
                .verfification_period
                .map(|v| VerificationPeriodDef::from(&v)),
            input_file_location: value.input_file_location.clone(),
            location: value
                .extracted_dataset_result
                .as_ref()
                .map(|r| r.location().to_path_buf()),
            verification_information: value.verification_information.clone(),
            verification_status: value.verification_status.clone(),
        }
    }
}
