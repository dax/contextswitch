use chrono::{DateTime, Utc};
use configparser::ini::Ini;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::Command;
use std::str;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub uuid: Uuid,
    pub id: u64,
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

fn write_default_config(data_location: &str) -> String {
    let mut taskrc = Ini::new();
    taskrc.setstr("default", "data.location", Some(data_location));
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

    taskrc_location.into()
}

pub fn load_config(task_data_location: Option<&str>) -> String {
    if let Ok(taskrc_location) = env::var("TASKRC") {
        let mut taskrc = Ini::new();
        taskrc
            .load(&taskrc_location)
            .unwrap_or_else(|_| panic!("Cannot load taskrc file {}", taskrc_location));
        let data_location = taskrc.get("default", "data.location").unwrap_or_else(|| {
            panic!(
                "'data.location' must be set in taskrc file {}",
                taskrc_location
            )
        });
        debug!(
            "Extracted data location `{}` from existing taskrc `{}`",
            data_location, taskrc_location
        );

        data_location
    } else {
        let data_location = task_data_location
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                env::var("TASK_DATA_LOCATION")
                    .expect("Expecting TASKRC or TASK_DATA_LOCATION environment variable value")
            });
        let taskrc_location = write_default_config(&data_location);

        env::set_var("TASKRC", &taskrc_location);
        debug!("Default taskrc written in `{}`", &taskrc_location);

        data_location
    }
}

#[tracing::instrument(level = "debug")]
pub fn export(filters: Vec<&str>) -> Result<Vec<Task>, Error> {
    let args = [filters, vec!["export"]].concat();
    let export_output = Command::new("task").args(args).output()?;

    let tasks: Vec<Task> = serde_json::from_slice(&export_output.stdout)?;

    Ok(tasks)
}

#[tracing::instrument(level = "debug")]
pub fn add(add_args: Vec<&str>) -> Result<u64, Error> {
    let mut args = vec!["add"];
    args.extend(add_args);
    let add_output = Command::new("task").args(args).output()?;
    let output = String::from_utf8(add_output.stdout).unwrap();
    lazy_static! {
        static ref RE: Regex = Regex::new(r"Created task (?P<id>\d+).").unwrap();
    }
    let task_id_capture = RE.captures(&output).ok_or_else(|| {
        Error::new(
            ErrorKind::Other,
            format!("Cannot extract task ID from: {}", &output),
        )
    })?;
    let task_id_str = task_id_capture
        .name("id")
        .ok_or_else(|| {
            Error::new(
                ErrorKind::Other,
                format!("Cannot extract task ID value from: {}", &output),
            )
        })?
        .as_str();

    task_id_str
        .parse::<u64>()
        .map_err(|_| Error::new(ErrorKind::Other, "Cannot parse task ID value"))
}
