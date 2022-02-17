use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use configparser::ini::Ini;
use contextswitch_types::{ContextswitchData, Task, TaskId};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::fmt;
use std::path::Path;
use std::process::Command;
use std::str;
use tokio::sync::Mutex;
use tracing::{debug, warn};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct TaskwarriorTaskLocalId(pub u64);

impl fmt::Display for TaskwarriorTaskLocalId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq)]
pub struct TaskwarriorTaskId(pub Uuid);

impl fmt::Display for TaskwarriorTaskId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&TaskwarriorTaskId> for TaskId {
    fn from(task: &TaskwarriorTaskId) -> Self {
        TaskId(task.0)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TaskwarriorTask {
    pub uuid: TaskwarriorTaskId,
    pub id: TaskwarriorTaskLocalId,
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
    pub start: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "contextswitch_types::opt_tw_date_format"
    )]
    pub end: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "contextswitch_types::opt_tw_date_format"
    )]
    pub wait: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<contextswitch_types::Priority>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recur: Option<contextswitch_types::Recurrence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contextswitch: Option<String>,
}

impl From<&TaskwarriorTask> for Task {
    fn from(task: &TaskwarriorTask) -> Self {
        let cs_data =
            task.contextswitch
                .as_ref()
                .and_then(|cs_string| -> Option<ContextswitchData> {
                    let contextswitch_data_result = serde_json::from_str(cs_string);
                    if contextswitch_data_result.is_err() {
                        warn!(
                            "Invalid Contextswitch data found in {}: {}",
                            &task.uuid, cs_string
                        );
                    }
                    contextswitch_data_result.ok()
                });

        Task {
            id: (&task.uuid).into(),
            entry: task.entry,
            modified: task.modified,
            status: task.status,
            description: task.description.clone(),
            urgency: task.urgency,
            due: task.due,
            start: task.start,
            end: task.end,
            wait: task.wait,
            parent: task.parent,
            project: task.project.clone(),
            priority: task.priority,
            recur: task.recur,
            tags: task.tags.clone(),
            contextswitch: cs_data,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TaskwarriorError {
    #[error("Error while executing Taskwarrior")]
    ExecutionError(#[from] std::io::Error),
    #[error("Error while parsing Taskwarrior output")]
    OutputParsingError {
        #[source]
        source: serde_json::Error,
        output: String,
    },
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

fn write_default_config(data_location: &str) -> String {
    let mut taskrc = Ini::new();
    taskrc.setstr("default", "data.location", Some(data_location));
    taskrc.setstr("default", "uda.contextswitch.type", Some("string"));
    taskrc.setstr(
        "default",
        "uda.contextswitch.label",
        Some("Contextswitch data"),
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
pub fn list_tasks(filters: Vec<&str>) -> Result<Vec<TaskwarriorTask>, TaskwarriorError> {
    let args = [filters, vec!["export"]].concat();
    let export_output = Command::new("task")
        .args(args)
        .output()
        .map_err(TaskwarriorError::ExecutionError)?;

    let output =
        String::from_utf8(export_output.stdout).context("Failed to read Taskwarrior output")?;

    let tasks: Vec<TaskwarriorTask> = serde_json::from_str(&output)
        .map_err(|e| TaskwarriorError::OutputParsingError { source: e, output })?;

    Ok(tasks)
}

#[tracing::instrument(level = "debug")]
pub fn get_task_by_local_id(
    id: &TaskwarriorTaskLocalId,
) -> Result<Option<TaskwarriorTask>, TaskwarriorError> {
    let mut tasks: Vec<TaskwarriorTask> = list_tasks(vec![&id.to_string()])?;
    if tasks.len() > 1 {
        return Err(TaskwarriorError::UnexpectedError(anyhow!(
            "Found more than 1 task when searching for task with local ID {}",
            id
        )));
    }

    Ok(tasks.pop())
}

#[tracing::instrument(level = "debug")]
pub async fn add_task(add_args: Vec<&str>) -> Result<TaskwarriorTask, TaskwarriorError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"Created task (?P<id>\d+).").unwrap();
        static ref LOCK: Mutex<u32> = Mutex::new(0);
    }
    let _lock = LOCK.lock().await;

    let mut args = vec!["add"];
    args.extend(add_args);
    let add_output = Command::new("task")
        .args(args)
        .output()
        .map_err(TaskwarriorError::ExecutionError)?;
    let output =
        String::from_utf8(add_output.stdout).context("Failed to read Taskwarrior output")?;
    let task_id_capture = RE
        .captures(&output)
        .ok_or_else(|| anyhow!("Cannot extract task ID from: {}", &output))?;
    let task_id_str = task_id_capture
        .name("id")
        .ok_or_else(|| anyhow!("Cannot extract task ID value from: {}", &output))?
        .as_str();

    let task_id = TaskwarriorTaskLocalId(
        task_id_str
            .parse::<u64>()
            .context("Cannot parse task ID value")?,
    );

    let task = get_task_by_local_id(&task_id)?;
    task.ok_or_else(|| {
        TaskwarriorError::UnexpectedError(anyhow!(
            "Newly created task with ID {} was not found",
            task_id
        ))
    })
}
