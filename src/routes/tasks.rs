use crate::contextswitch;
use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use contextswitch_types::{NewTask, Task};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TaskQuery {
    filter: Option<String>,
}

impl ResponseError for contextswitch::ContextswitchError {
    fn status_code(&self) -> StatusCode {
        match self {
            contextswitch::ContextswitchError::InvalidDataError { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            contextswitch::ContextswitchError::UnexpectedError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

#[tracing::instrument(level = "debug", skip_all, fields(filter = %task_query.filter.as_ref().unwrap_or(&"".to_string())))]
pub async fn list_tasks(
    task_query: web::Query<TaskQuery>,
) -> Result<HttpResponse, contextswitch::ContextswitchError> {
    let filter = task_query
        .filter
        .as_ref()
        .map_or(vec![], |filter| filter.split(' ').collect());
    let tasks: Vec<Task> = contextswitch::list_tasks(filter)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&tasks).context("Cannot serialize Contextswitch task")?))
}

#[tracing::instrument(level = "debug", skip_all, fields(definition = %task.definition))]
pub async fn add_task(
    task: web::Json<NewTask>,
) -> Result<HttpResponse, contextswitch::ContextswitchError> {
    let task: Task = contextswitch::add_task(task.definition.split(' ').collect()).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&task).context("Cannot serialize Contextswitch task")?))
}

#[tracing::instrument(level = "debug")]
pub fn option_task() -> HttpResponse {
    HttpResponse::Ok().finish()
}
