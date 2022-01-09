use contextswitch_api::taskwarrior;
use mktemp::Temp;
use std::fs;
use std::net::TcpListener;

pub fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let server = contextswitch_api::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}

pub fn setup_tasks() -> String {
    let tmp_dir = Temp::new_dir().unwrap();
    let task_data_location = taskwarrior::load_config(tmp_dir.to_str());
    tmp_dir.release();

    return task_data_location;
}

pub fn clear_tasks(task_data_location: String) {
    fs::remove_dir_all(task_data_location).unwrap();
}