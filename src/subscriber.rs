use rust_ev_verifier_lib::Config as VerifierConfig;
use std::fs::File;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, EnvFilter, Layer};

/// Init the subscriber with or without stdout
pub fn init_subscriber(config: &'static VerifierConfig) -> Vec<WorkerGuard> {
    // Get the logile
    let log_file = File::options()
        .create(true)
        .append(true)
        .open(config.log_file_path())
        .unwrap();

    // Define which span evens will be logged (new and clode)
    let span_events = FmtSpan::NEW | FmtSpan::CLOSE;

    // Define the writers for output and file, using non_blocking
    let (mk_writer_output, guard_output) = tracing_appender::non_blocking(std::io::stdout());
    let (mk_writer_file, guard_file) = tracing_appender::non_blocking(log_file);

    // Define the layer for output
    let layer_output = tracing_subscriber::fmt::layer()
        .with_span_events(span_events.clone())
        .with_writer(mk_writer_output)
        .with_filter(EnvFilter::from_default_env());

    // Define the layer for file
    // USe the EnvFilter to read the value "RUST_LOG" in .env
    let layer_file = tracing_subscriber::fmt::layer()
        .with_span_events(span_events)
        .with_writer(mk_writer_file)
        .with_filter(EnvFilter::from_default_env());

    // Combine the layers in a subcriber
    // USe the EnvFilter to read the value "RUST_LOG" in .env
    let subscriber = tracing_subscriber::registry()
        .with(layer_output)
        .with(layer_file);

    // Set the subscriber as global
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Return the guards, in order to ensure that the logs will be written
    // See https://docs.rs/tracing-appender/latest/tracing_appender/non_blocking/struct.WorkerGuard.html
    vec![guard_output, guard_file]
}
