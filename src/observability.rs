use tracing::Level;
use tracing::{subscriber::set_global_default, Subscriber};
use tracing_log::LogTracer;
use tracing_subscriber::{filter, fmt, fmt::format, layer::SubscriberExt, EnvFilter, Layer};

pub fn get_subscriber(env_filter_str: String) -> impl Subscriber + Send + Sync {
    let fmt_layer = fmt::layer()
        .event_format(format::Format::default().json())
        .fmt_fields(format::JsonFields::new())
        .flatten_event(true)
        .with_test_writer();

    let access_log_layer = fmt::layer()
        .event_format(format::Format::default().json())
        .fmt_fields(format::JsonFields::new())
        .flatten_event(true)
        .with_test_writer()
        .with_span_events(format::FmtSpan::CLOSE)
        .with_span_list(false)
        .with_filter(
            filter::Targets::new().with_target("tracing_actix_web::root_span_builder", Level::INFO),
        );

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter_str));

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(access_log_layer)
        .with(env_filter)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}
