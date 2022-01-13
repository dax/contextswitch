use contextswitch_api::observability::{get_subscriber, init_subscriber};
use contextswitch_api::taskwarrior;
use mktemp::Temp;
use once_cell::sync::Lazy;
use std::fs;
use std::net::TcpListener;

static TRACING: Lazy<()> = Lazy::new(|| {
    let subscriber = get_subscriber("info".to_string());
    init_subscriber(subscriber);
});

static SERVER_ADDRESS: Lazy<String> = Lazy::new(|| {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let server = contextswitch_api::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
});

static TASK_DATA_LOCATION: Lazy<String> = Lazy::new(|| {
    let tmp_dir = Temp::new_dir().unwrap();
    let task_data_location = taskwarrior::load_config(tmp_dir.to_str());
    tmp_dir.release();

    task_data_location
});

pub fn spawn_app() -> String {
    Lazy::force(&TRACING);
    setup_tasks();
    Lazy::force(&SERVER_ADDRESS).to_string()
}

pub fn setup_tasks() -> String {
    Lazy::force(&TASK_DATA_LOCATION).to_string()
}

pub fn clear_tasks(task_data_location: String) {
    fs::remove_dir_all(task_data_location).unwrap();
}
