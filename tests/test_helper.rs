use contextswitch_api::contextswitch::taskwarrior;
use contextswitch_api::observability::{get_subscriber, init_subscriber};
use mktemp::Temp;
use rstest::*;
use std::fs;
use std::net::TcpListener;
use tracing::info;

fn setup_tracing() {
    info!("Setting up tracing");
    let subscriber = get_subscriber("debug".to_string());
    init_subscriber(subscriber);
}

fn setup_server() -> String {
    info!("Setting up server");
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let server = contextswitch_api::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}

fn setup_taskwarrior() -> String {
    info!("Setting up TW");
    let tmp_dir = Temp::new_dir().unwrap();
    let task_data_location = taskwarrior::load_config(tmp_dir.to_str());
    tmp_dir.release();

    task_data_location
}

pub fn clear_tasks(task_data_location: String) {
    fs::remove_dir_all(task_data_location).unwrap();
}

#[fixture]
#[once]
pub fn app_address() -> String {
    setup_tracing();
    setup_taskwarrior();
    setup_server()
}
