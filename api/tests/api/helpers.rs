use contextswitch_api::configuration::Settings;
use contextswitch_api::contextswitch::taskwarrior;
use contextswitch_api::observability::{get_subscriber, init_subscriber};
use mktemp::Temp;
use rstest::*;
use std::net::TcpListener;
use tracing::info;

fn setup_tracing(settings: &Settings) {
    info!("Setting up tracing");
    let subscriber = get_subscriber(&settings.application.log_directive);
    init_subscriber(subscriber);
}

fn setup_server(settings: &Settings) -> String {
    info!("Setting up server");
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let server = contextswitch_api::run(listener, &settings).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}

fn setup_taskwarrior(mut settings: Settings) -> String {
    info!("Setting up Taskwarrior");
    let tmp_dir = Temp::new_dir().unwrap();
    settings.taskwarrior.data_location = tmp_dir.to_str().map(String::from);
    let task_data_location = taskwarrior::load_config(&settings.taskwarrior);
    tmp_dir.release();

    task_data_location
}

#[fixture]
#[once]
pub fn app_address() -> String {
    let settings = Settings::new_from_file(Some("config/test".to_string()))
        .expect("Cannot load test configuration");
    setup_tracing(&settings);
    let address = setup_server(&settings);
    setup_taskwarrior(settings);
    address
}
