use contextswitch_api::configuration::Settings;
use contextswitch_api::observability::{get_subscriber, init_subscriber};
use contextswitch_api::{contextswitch::taskwarrior, run};
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let settings = Settings::new().expect("Cannot load Contextswitch configuration");
    let subscriber = get_subscriber(&settings.application.log_directive);
    init_subscriber(subscriber);

    taskwarrior::load_config(&settings.taskwarrior);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", settings.application.port))
        .expect("Failed to bind port");
    run(listener)?.await
}
