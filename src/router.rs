use crate::{
    app_data::{AppDataLockArc, AppStatus},
    handler::{
        context_dataset_handler, extract_handler, health_check_handler, init_handler,
        manual_checks_handler, period_dataset_handler, reset_handler, run_handler, status_handler,
    },
};
use axum::{
    routing::{get, post},
    Router,
};
use strum::{AsRefStr, EnumString};

pub const ALLOWED_ROUTE_PATHES: &[(AppStatus, &[RoutePath])] = &[
    (
        AppStatus::NotInitialized,
        &[RoutePath::Init, RoutePath::Status, RoutePath::Root],
    ),
    (
        AppStatus::Initialized,
        &[
            RoutePath::ContextDataset,
            RoutePath::Status,
            RoutePath::Root,
            RoutePath::Reset,
        ],
    ),
    (
        AppStatus::ContextDataSetLoaded,
        &[RoutePath::PeriodDataset, RoutePath::Status, RoutePath::Root],
    ),
    (
        AppStatus::PeriodDataSetLoaded,
        &[
            RoutePath::Extract,
            RoutePath::Status,
            RoutePath::Root,
            RoutePath::Reset,
        ],
    ),
    (
        AppStatus::Extracting,
        &[RoutePath::Status, RoutePath::ManualChecks, RoutePath::Root],
    ),
    (
        AppStatus::ExtractError,
        &[RoutePath::Status, RoutePath::ManualChecks, RoutePath::Root],
    ),
    (
        AppStatus::Extracted,
        &[
            RoutePath::Run,
            RoutePath::Status,
            RoutePath::ManualChecks,
            RoutePath::Root,
            RoutePath::Reset,
        ],
    ),
    (
        AppStatus::Running,
        &[RoutePath::Status, RoutePath::ManualChecks, RoutePath::Root],
    ),
    (
        AppStatus::RunError,
        &[RoutePath::Status, RoutePath::ManualChecks, RoutePath::Root],
    ),
    (
        AppStatus::Finished,
        &[
            RoutePath::Reset,
            RoutePath::Status,
            RoutePath::ManualChecks,
            RoutePath::Root,
        ],
    ),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, AsRefStr, EnumString)]
pub enum RoutePath {
    #[strum(serialize = "/")]
    Root,
    #[strum(serialize = "/status")]
    Status,
    #[strum(serialize = "/manual-checks")]
    ManualChecks,
    #[strum(serialize = "/init")]
    Init,
    #[strum(serialize = "/context-dataset")]
    ContextDataset,
    #[strum(serialize = "/period-dataset")]
    PeriodDataset,
    #[strum(serialize = "/extract")]
    Extract,
    #[strum(serialize = "/run")]
    Run,
    #[strum(serialize = "/reset")]
    Reset,
}

pub fn routes() -> Router<AppDataLockArc> {
    Router::new()
        .route(RoutePath::Root.as_ref(), get(health_check_handler))
        .route(RoutePath::Status.as_ref(), get(status_handler))
        .route(RoutePath::ManualChecks.as_ref(), get(manual_checks_handler))
        .route(RoutePath::Init.as_ref(), post(init_handler))
        .route(
            RoutePath::ContextDataset.as_ref(),
            post(context_dataset_handler),
        )
        .route(
            RoutePath::PeriodDataset.as_ref(),
            post(period_dataset_handler),
        )
        .route(RoutePath::Extract.as_ref(), post(extract_handler))
        .route(RoutePath::Run.as_ref(), post(run_handler))
        .route(RoutePath::Reset.as_ref(), post(reset_handler))
        .fallback(dummy_handler)
}
pub async fn dummy_handler() {}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_routes() {
        assert_eq!(RoutePath::Root.as_ref(), "/");
        assert_eq!(RoutePath::from_str("/").unwrap(), RoutePath::Root);
    }
}
