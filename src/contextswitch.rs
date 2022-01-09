use crate::taskwarrior;
use contextswitch_types::{ContextSwitchMetadata, Task};
use std::io::Error;

impl TryFrom<&taskwarrior::Task> for Task {
    type Error = std::io::Error;

    fn try_from(task: &taskwarrior::Task) -> Result<Self, Self::Error> {
        let cs_metadata = task.contextswitch.as_ref().map_or(
            Ok(None),
            |cs_string| -> Result<Option<ContextSwitchMetadata>, serde_json::Error> {
                if cs_string.is_empty() || cs_string == "{}" {
                    Ok(None)
                } else {
                    Some(serde_json::from_str(&cs_string)).transpose()
                }
            },
        )?;

        Ok(Task {
            uuid: task.uuid,
            id: task.id,
            entry: task.entry,
            modified: task.modified,
            status: task.status,
            description: task.description.clone(),
            urgency: task.urgency,
            due: task.due,
            end: task.end,
            parent: task.parent,
            project: task.project.clone(),
            recur: task.recur,
            tags: task.tags.clone(),
            contextswitch: cs_metadata,
        })
    }
}

pub fn export(filters: Vec<&str>) -> Result<Vec<Task>, Error> {
    let tasks: Result<Vec<Task>, Error> = taskwarrior::export(filters)?
        .iter()
        .map(|task| Task::try_from(task))
        .collect();
    tasks
}
