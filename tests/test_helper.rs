use configparser::ini::Ini;
use mktemp::Temp;
use std::env;
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;

pub fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let server = contextswitch::run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}

pub fn setup_tasks() -> PathBuf {
    let mut config = Ini::new();
    config.setstr("default", "uda.contextswitch.type", Some("string"));
    config.setstr(
        "default",
        "uda.contextswitch.label",
        Some("Context Switch metadata"),
    );
    config.setstr("default", "uda.contextswitch.default", Some("{}"));

    let tmp_dir = Temp::new_dir().unwrap();
    let task_data_path = tmp_dir.to_path_buf();
    config.setstr(
        "default",
        "data.location",
        Some(task_data_path.to_str().unwrap()),
    );

    let taskrc_path = task_data_path.join(".taskrc");
    config.write(taskrc_path.as_path()).unwrap();
    env::set_var("TASKRC", taskrc_path.to_str().unwrap());

    tmp_dir.release();
    return task_data_path;
}

pub fn clear_tasks(task_data_path: PathBuf) {
    fs::remove_dir_all(task_data_path).unwrap();
}
