use crate::configuration::TaskwarriorSettings;
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

use super::ContextswitchError;

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
pub fn get_task_by_id(
    uuid: &TaskwarriorTaskId,
) -> Result<Option<TaskwarriorTask>, TaskwarriorError> {
    let mut tasks: Vec<TaskwarriorTask> = list_tasks(vec![&uuid.to_string()])?;
    if tasks.len() > 1 {
        return Err(TaskwarriorError::UnexpectedError(anyhow!(
            "Found more than 1 task when searching for task with UUID {}",
            uuid
        )));
    }

    Ok(tasks.pop())
}

lazy_static! {
    static ref RE: Regex = Regex::new(r"Modified 1 task.").unwrap();
    static ref TW_WRITE_LOCK: Mutex<u32> = Mutex::new(0);
}

#[tracing::instrument(level = "debug")]
pub async fn add_task(add_args: Vec<&str>) -> Result<TaskwarriorTask, TaskwarriorError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"Created task (?P<id>\d+).").unwrap();
    }
    let _lock = TW_WRITE_LOCK.lock().await;

    let args = [vec!["add"], add_args].concat();
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

#[tracing::instrument(level = "debug")]
pub async fn update_task(action: TaskwarriorAction) -> Result<TaskwarriorTask, TaskwarriorError> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"Modified 1 task.").unwrap();
    }

    let _lock = TW_WRITE_LOCK.lock().await;
    let args = [
        vec![action.uuid.to_string(), "mod".to_string()],
        action.args,
    ]
    .concat();
    Command::new("task")
        .args(args)
        .output()
        .map_err(TaskwarriorError::ExecutionError)?;

    let updated_task = get_task_by_id(&action.uuid)?;
    updated_task.ok_or_else(|| {
        TaskwarriorError::UnexpectedError(anyhow!(
            "Updated task with UUID {} was not found",
            action.uuid
        ))
    })
}

// Types
// TaskwarriorTask
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

impl From<TaskwarriorTaskId> for TaskId {
    fn from(task: TaskwarriorTaskId) -> Self {
        TaskId(task.0)
    }
}

impl From<TaskId> for TaskwarriorTaskId {
    fn from(task: TaskId) -> Self {
        TaskwarriorTaskId(task.0)
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
    pub parent: Option<TaskwarriorTaskId>,
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

impl From<TaskwarriorTask> for Task {
    fn from(task: TaskwarriorTask) -> Self {
        (&task).into()
    }
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
            id: task.uuid.clone().into(),
            entry: task.entry,
            modified: task.modified,
            status: task.status,
            description: task.description.clone(),
            urgency: task.urgency,
            due: task.due,
            start: task.start,
            end: task.end,
            wait: task.wait,
            parent: task.parent.clone().map(|id| id.into()),
            project: task.project.clone(),
            priority: task.priority,
            recur: task.recur,
            tags: task.tags.clone(),
            contextswitch: cs_data,
        }
    }
}

// TaskwarriorAction
#[derive(Debug)]
pub struct TaskwarriorAction {
    pub uuid: TaskwarriorTaskId,
    pub args: Vec<String>,
}

fn to_arg(arg: &str) -> impl Fn(String) -> String + '_ {
    move |value: String| format!("{}:{}", arg, value)
}

fn format_date(date: DateTime<Utc>) -> String {
    date.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

impl TryFrom<Task> for TaskwarriorAction {
    type Error = ContextswitchError;

    fn try_from(task: Task) -> Result<Self, Self::Error> {
        (&task).try_into()
    }
}

fn format_json<T>(data_opt: &Option<T>) -> Result<Option<String>, ContextswitchError>
where
    T: Sized + Serialize,
{
    data_opt
        .as_ref()
        .map(|data| serde_json::to_string(data).map_err(ContextswitchError::InvalidDataError))
        .transpose()
}

impl TryFrom<&Task> for TaskwarriorAction {
    type Error = ContextswitchError;

    fn try_from(task: &Task) -> Result<Self, Self::Error> {
        let args = vec![task.description.clone()];
        let tags_args = task
            .tags
            .clone()
            .map(|tags| {
                tags.iter()
                    .map(|tag| format!("+{}", tag)) // TODO remove tags
                    .collect::<Vec<String>>()
            })
            .unwrap_or_else(Vec::new);
        let opt_args = [
            task.due
                .map(format_date)
                .or_else(|| Some("".to_string()))
                .map(to_arg("due")),
            task.start
                .map(format_date)
                .or_else(|| Some("".to_string()))
                .map(to_arg("start")),
            task.end
                .map(format_date)
                .or_else(|| Some("".to_string()))
                .map(to_arg("end")),
            task.wait
                .map(format_date)
                .or_else(|| Some("".to_string()))
                .map(to_arg("wait")),
            task.parent
                .as_ref()
                .map(|id| id.to_string())
                .or_else(|| Some("".to_string()))
                .map(to_arg("parent")),
            task.project
                .clone()
                .or_else(|| Some("".to_string()))
                .map(to_arg("project")),
            task.priority
                .map(|priority| priority.to_string())
                .or_else(|| Some("".to_string()))
                .map(to_arg("priority")),
            task.recur
                .map(|recur| recur.to_string())
                .or_else(|| Some("".to_string()))
                .map(to_arg("recur")),
            format_json(&task.contextswitch)?
                .or_else(|| Some("".to_string()))
                .map(to_arg("contextswitch")),
        ];

        Ok(TaskwarriorAction {
            uuid: task.id.clone().into(),
            args: [
                args,
                tags_args,
                opt_args
                    .iter()
                    .filter_map(|arg| arg.clone())
                    .collect::<Vec<String>>(),
            ]
            .concat(),
        })
    }
}

// Errors
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

// Taskwarrior config functions
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

pub fn load_config(settings: &TaskwarriorSettings) -> String {
    if let Some(taskrc_location) = &settings.taskrc {
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

        env::set_var("TASKRC", &taskrc_location);
        debug!(
            "Extracted data location `{}` from existing taskrc `{}`",
            data_location, taskrc_location
        );

        data_location
    } else {
        let data_location = settings
            .data_location
            .as_ref()
            .expect("Expecting taskwarrior.taskrc or taskwarrior.data_location setting to be set")
            .to_string();
        let taskrc_location = write_default_config(&data_location);

        env::set_var("TASKRC", &taskrc_location);
        debug!("Default taskrc written in `{}`", &taskrc_location);

        data_location
    }
}

#[cfg(test)]
mod tests {

    mod from_taskwarrior_task_to_contextswitch_task {
        use super::super::*;
        use chrono::TimeZone;
        use contextswitch_types::Bookmark;
        use http::uri::Uri;
        use proptest::prelude::*;

        #[test]
        fn test_successful_full_convertion() {
            let tw_task = TaskwarriorTask {
                uuid: TaskwarriorTaskId(Uuid::new_v4()),
                id: TaskwarriorTaskLocalId(42),
                entry: Utc.ymd(2022, 1, 1).and_hms(1, 0, 0),
                modified: Utc.ymd(2022, 1, 1).and_hms(1, 0, 1),
                status: contextswitch_types::Status::Pending,
                description: "simple task".to_string(),
                urgency: 0.5,
                due: Some(Utc.ymd(2022, 1, 1).and_hms(1, 0, 2)),
                start: Some(Utc.ymd(2022, 1, 1).and_hms(1, 0, 3)),
                end: Some(Utc.ymd(2022, 1, 1).and_hms(1, 0, 4)),
                wait: Some(Utc.ymd(2022, 1, 1).and_hms(1, 0, 5)),
                parent: Some(TaskwarriorTaskId(Uuid::new_v4())),
                project: Some("simple project".to_string()),
                priority: Some(contextswitch_types::Priority::H),
                recur: Some(contextswitch_types::Recurrence::Daily),
                tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
                contextswitch: Some(String::from(
                    r#"{"bookmarks": [{"uri": "https://www.example.com/path"}]}"#,
                )),
            };
            let cs_task: Task = (&tw_task).into();

            assert_eq!(tw_task.uuid.0, cs_task.id.0);
            assert_eq!(tw_task.entry, cs_task.entry);
            assert_eq!(tw_task.modified, cs_task.modified);
            assert_eq!(tw_task.status, cs_task.status);
            assert_eq!(tw_task.description, cs_task.description);
            assert_eq!(tw_task.urgency, cs_task.urgency);
            assert_eq!(tw_task.due, cs_task.due);
            assert_eq!(tw_task.start, cs_task.start);
            assert_eq!(tw_task.end, cs_task.end);
            assert_eq!(tw_task.wait, cs_task.wait);
            assert_eq!(
                tw_task.parent.map(|id| id.to_string()),
                cs_task.parent.map(|id| id.to_string())
            );
            assert_eq!(tw_task.project, cs_task.project);
            assert_eq!(tw_task.priority, cs_task.priority);
            assert_eq!(tw_task.recur, cs_task.recur);
            assert_eq!(tw_task.tags, cs_task.tags);
            assert_eq!(
                Some(ContextswitchData {
                    bookmarks: vec![Bookmark {
                        uri: "https://www.example.com/path".parse::<Uri>().unwrap(),
                        content: None
                    }]
                }),
                cs_task.contextswitch
            );
        }

        proptest! {
            #[test]
            fn test_conversion_with_invalid_contextswitch_data_format(cs_data in ".*") {
                let tw_task = TaskwarriorTask {
                    uuid: TaskwarriorTaskId(Uuid::new_v4()),
                    id: TaskwarriorTaskLocalId(42),
                    entry: Utc.ymd(2022, 1, 1).and_hms(1, 0, 0),
                    modified: Utc.ymd(2022, 1, 1).and_hms(1, 0, 1),
                    status: contextswitch_types::Status::Pending,
                    description: "simple task".to_string(),
                    urgency: 0.5,
                    due: None,
                    start: None,
                    end: None,
                    wait: None,
                    parent: None,
                    project: None,
                    priority: None,
                    recur: None,
                    tags: None,
                    contextswitch: Some(cs_data),
                };
                let cs_task: Task = (&tw_task).into();

                assert_eq!(tw_task.uuid.0, cs_task.id.0);
                assert_eq!(tw_task.entry, cs_task.entry);
                assert_eq!(tw_task.modified, cs_task.modified);
                assert_eq!(tw_task.status, cs_task.status);
                assert_eq!(tw_task.description, cs_task.description);
                assert_eq!(tw_task.urgency, cs_task.urgency);
                assert_eq!(None, cs_task.contextswitch);
            }
        }
    }

    mod from_contextswitch_task_to_taskwarrior_action {
        use super::super::*;
        use chrono::TimeZone;
        use contextswitch_types::{Bookmark, Priority, Recurrence};
        use http::Uri;

        #[test]
        fn test_successful_convertion() {
            let task = Task {
                id: TaskId(Uuid::new_v4()),
                entry: Utc.ymd(2022, 1, 1).and_hms(1, 0, 0),
                modified: Utc.ymd(2022, 1, 1).and_hms(1, 0, 1),
                status: contextswitch_types::Status::Pending,
                description: "simple task".to_string(),
                urgency: 0.5,
                due: None,
                start: None,
                end: None,
                wait: None,
                parent: None,
                project: None,
                priority: None,
                recur: None,
                tags: None,
                contextswitch: None,
            };
            let action: TaskwarriorAction = (&task)
                .try_into()
                .expect("Failed to convert Task into TaskwarriorAction");

            assert_eq!(task.id.0, action.uuid.0);
            assert_eq!(
                vec![
                    task.description,
                    "due:".to_string(),
                    "start:".to_string(),
                    "end:".to_string(),
                    "wait:".to_string(),
                    "parent:".to_string(),
                    "project:".to_string(),
                    "priority:".to_string(),
                    "recur:".to_string(),
                    "contextswitch:".to_string(),
                ],
                action.args
            );
        }

        #[test]
        fn test_successful_full_convertion() {
            let task = Task {
                id: TaskId(Uuid::new_v4()),
                entry: Utc.ymd(2022, 1, 1).and_hms(1, 0, 0),
                modified: Utc.ymd(2022, 1, 1).and_hms(1, 0, 1),
                status: contextswitch_types::Status::Pending,
                description: "simple task".to_string(),
                urgency: 0.5,
                due: Some(Utc.ymd(2022, 1, 1).and_hms(1, 0, 2)),
                start: Some(Utc.ymd(2022, 1, 1).and_hms(1, 0, 3)),
                end: Some(Utc.ymd(2022, 1, 1).and_hms(1, 0, 4)),
                wait: Some(Utc.ymd(2022, 1, 1).and_hms(1, 0, 5)),
                parent: Some(TaskId(Uuid::new_v4())),
                project: Some("myproject".to_string()),
                priority: Some(Priority::H),
                recur: Some(Recurrence::Monthly),
                tags: Some(vec!["tag1".to_string(), "tag2".to_string()]),
                contextswitch: Some(ContextswitchData {
                    bookmarks: vec![
                        Bookmark {
                            uri: "https://www.example.com/path".parse::<Uri>().unwrap(),
                            content: None,
                        },
                        Bookmark {
                            uri: "https://www.example.com/path2".parse::<Uri>().unwrap(),
                            content: None,
                        },
                    ],
                }),
            };
            let action: TaskwarriorAction = (&task)
                .try_into()
                .expect("Failed to convert Task into TaskwarriorAction");

            assert_eq!(task.id.0, action.uuid.0);
            assert_eq!(
                vec![
                    task.description,
                    "+tag1".to_string(),
                    "+tag2".to_string(),
                    "due:2022-01-01T01:00:02Z".to_string(),
                    "start:2022-01-01T01:00:03Z".to_string(),
                    "end:2022-01-01T01:00:04Z".to_string(),
                    "wait:2022-01-01T01:00:05Z".to_string(),
                    format!("parent:{}", task.parent.unwrap()),
                    "project:myproject".to_string(),
                    "priority:H".to_string(),
                    "recur:monthly".to_string(),
                    String::from(
                        r#"contextswitch:{"bookmarks":[{"uri":"https://www.example.com/path"},{"uri":"https://www.example.com/path2"}]}"#
                    )
                ],
                action.args
            );
        }
    }
}
