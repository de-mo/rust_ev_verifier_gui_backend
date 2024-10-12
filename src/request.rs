use std::path::PathBuf;

use crate::app_data::VerificationPeriodDef;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct InitRequest {
    pub period: VerificationPeriodDef,
}

#[derive(Deserialize)]
pub struct FilePathRequest {
    pub path: PathBuf,
}
