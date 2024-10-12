use crate::CONFIG;
use rust_ev_verifier_lib::{
    application_runner::ExtractDataSetResults,
    verification::{VerificationMetaDataList, VerificationPeriod},
    Config,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use strum::AsRefStr;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr, Serialize, Deserialize)]
pub enum AppStatus {
    NotInitialized,
    Initialized,
    ContextDataSetLoaded,
    PeriodDataSetLoaded,
    Extracting,
    Extracted,
    Running,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputFileLocation {
    pub context_zip_file: Option<PathBuf>,
    pub setup_zip_file: Option<PathBuf>,
    pub tally_zip_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatusEnum {
    NotStarted,
    Running,
    FinishedSuccessfully,
    FinishedWithFailures,
    FinishedWithErrors,
    FinishedWithFailureAndErrors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationInformation {
    pub id: String,
    pub name: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationStatus {
    pub id: String,
    pub status: VerificationStatusEnum,
    pub failures: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, AsRefStr, Serialize, Deserialize)]
#[strum(serialize_all = "lowercase")]
pub enum VerificationPeriodDef {
    #[serde(rename = "setup")]
    Setup,
    #[serde(rename = "tally")]
    Tally,
}

pub struct AppData {
    pub app_status: AppStatus,
    pub config: &'static Config,
    pub verfification_period: Option<VerificationPeriod>,
    pub input_file_location: InputFileLocation,
    pub extracted_dataset_result: Option<ExtractDataSetResults>,
    pub verification_information: HashMap<String, VerificationInformation>,
    pub verification_status: HashMap<String, VerificationStatus>,
}

pub type AppDataLockArc = Arc<RwLock<AppData>>;

impl Default for AppData {
    fn default() -> Self {
        Self {
            app_status: AppStatus::NotInitialized,
            config: &CONFIG,
            verfification_period: None,
            input_file_location: InputFileLocation::default(),
            extracted_dataset_result: None,
            verification_information: HashMap::new(),
            verification_status: HashMap::new(),
        }
    }
}

impl Default for InputFileLocation {
    fn default() -> Self {
        Self {
            context_zip_file: Default::default(),
            setup_zip_file: Default::default(),
            tally_zip_file: Default::default(),
        }
    }
}

impl Default for VerificationStatusEnum {
    fn default() -> Self {
        VerificationStatusEnum::NotStarted
    }
}

impl VerificationStatusEnum {
    pub fn from_has_errors_has_failures(has_errors: bool, has_failures: bool) -> Self {
        match has_errors {
            true => match has_failures {
                true => Self::FinishedWithFailureAndErrors,
                false => Self::FinishedWithErrors,
            },
            false => match has_failures {
                true => Self::FinishedWithFailures,
                false => Self::FinishedSuccessfully,
            },
        }
    }
}

impl AppData {
    pub fn new() -> AppDataLockArc {
        Arc::new(RwLock::new(AppData::default()))
    }

    pub fn set_with_medata(&mut self, metadata_list: &VerificationMetaDataList) {
        for &id in metadata_list
            .id_list_for_period(self.verfification_period.as_ref().unwrap())
            .iter()
        {
            let md = metadata_list.meta_data_from_id(id).unwrap();
            self.verification_information.insert(
                id.to_string(),
                VerificationInformation {
                    id: id.to_string(),
                    name: md.name().to_string(),
                    category: md.category().as_ref().to_string(),
                },
            );
            self.verification_status.insert(
                id.to_string(),
                VerificationStatus {
                    id: id.to_string(),
                    status: VerificationStatusEnum::default(),
                    failures: vec![],
                    errors: vec![],
                },
            );
        }
    }

    pub fn not_finished(&self) -> bool {
        self.verification_status.values().any(|v| {
            v.status == VerificationStatusEnum::NotStarted
                || v.status == VerificationStatusEnum::Running
        })
    }

    pub fn set_verification_status(
        &mut self,
        id: &str,
        errors: Vec<String>,
        failures: Vec<String>,
    ) {
        let has_errors = !errors.is_empty();
        let has_failures = !failures.is_empty();
        if let Some(vs) = self.verification_status.get_mut(id) {
            vs.status =
                VerificationStatusEnum::from_has_errors_has_failures(has_errors, has_failures);
            vs.errors = errors;
            vs.failures = failures
        }
    }
}

impl From<&VerificationPeriod> for VerificationPeriodDef {
    fn from(value: &VerificationPeriod) -> Self {
        match value {
            VerificationPeriod::Setup => VerificationPeriodDef::Setup,
            VerificationPeriod::Tally => VerificationPeriodDef::Tally,
        }
    }
}

impl From<&VerificationPeriodDef> for VerificationPeriod {
    fn from(value: &VerificationPeriodDef) -> Self {
        match value {
            VerificationPeriodDef::Setup => VerificationPeriod::Setup,
            VerificationPeriodDef::Tally => VerificationPeriod::Tally,
        }
    }
}
