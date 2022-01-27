use crate::contextswitch::taskwarrior;
use contextswitch_types::Task;
use std::io::Error;

#[tracing::instrument(level = "debug")]
pub fn list_tasks(filters: Vec<&str>) -> Result<Vec<Task>, Error> {
    let tasks: Result<Vec<Task>, Error> = taskwarrior::list_tasks(filters)?
        .iter()
        .map(Task::try_from)
        .collect();
    tasks
}

#[tracing::instrument(level = "debug")]
pub async fn add_task(add_args: Vec<&str>) -> Result<Task, Error> {
    let taskwarrior_task = taskwarrior::add_task(add_args).await?;
    (&taskwarrior_task).try_into()
}
