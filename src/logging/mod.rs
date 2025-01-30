use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize logging for the build system
pub fn init_logging() {
    // Create a subscriber with a formatting layer
    let subscriber = tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
        )
        // Optional: Add a filter layer to control log levels
        .with(
            tracing_subscriber::filter::EnvFilter::from_default_env()
        );

    // Set the global default subscriber
    subscriber.init();
}

/// Log a debug message
pub fn debug(message: &str) {
    tracing::debug!(message);
}

/// Log an info message
pub fn info(message: &str) {
    tracing::info!(message);
}

/// Log a warning message
pub fn warn(message: &str) {
    tracing::warn!(message);
}

/// Log an error message
pub fn error(message: &str) {
    tracing::error!(message);
}
