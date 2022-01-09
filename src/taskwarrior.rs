use chrono::{DateTime, Utc};
use configparser::ini::Ini;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::io::Error;
use std::path::Path;
use std::process::Command;
use std::str;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub uuid: Uuid,
    pub id: u32,
    #[serde(with = "contextswitch_types::tw_date_format")]
    pub entry: DateTime<Utc>,
    #[serde(with = "contextswitch_types::tw_date_format")]
    pub modified: DateTime<Utc>,
    pub status: contextswitch_types::Status,
    pub description: String,
    pub urgency: f64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "contextswitch_types::opt_tw_date_format"
    )]
    pub due: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "contextswitch_types::opt_tw_date_format"
    )]
    pub end: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recur: Option<contextswitch_types::Recurrence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contextswitch: Option<String>,
}

pub fn load_config(task_data_location: Option<&str>) -> String {
    if let Ok(taskrc_location) = env::var("TASKRC") {
        let mut taskrc = Ini::new();
        taskrc
            .load(&taskrc_location)
            .expect(&format!("Cannot load taskrc file {}", taskrc_location));
        return taskrc.get("default", "data.location").expect(&format!(
            "'data.location' must be set in taskrc file {}",
            taskrc_location
        ));
    }

    let data_location = task_data_location
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            env::var("TASK_DATA_LOCATION")
                .expect("Expecting TASKRC or TASK_DATA_LOCATION environment variable value")
        });

    let mut taskrc = Ini::new();
    taskrc.setstr("default", "data.location", Some(&data_location));
    taskrc.setstr("default", "uda.contextswitch.type", Some("string"));
    taskrc.setstr(
        "default",
        "uda.contextswitch.label",
        Some("Context Switch metadata"),
    );
    taskrc.setstr("default", "uda.contextswitch.default", Some("{}"));

    let taskrc_path = Path::new(&data_location).join(".taskrc");
    let taskrc_location = taskrc_path.to_str().unwrap();
    taskrc.write(taskrc_location).unwrap();

    env::set_var("TASKRC", taskrc_location);

    data_location
}

pub fn export(filters: Vec<&str>) -> Result<Vec<Task>, Error> {
    let mut args = vec!["export"];
    args.extend(filters);
    let export_output = Command::new("task").args(args).output()?;

    let tasks: Vec<Task> = serde_json::from_slice(&export_output.stdout)?;

    Ok(tasks)
}

pub fn add(add_args: Vec<&str>) -> Result<(), Error> {
    let mut args = vec!["add"];
    args.extend(add_args);
    Command::new("task").args(args).output()?;

    Ok(())
}
