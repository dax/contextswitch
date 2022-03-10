use crate::contextswitch as cs;
use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use contextswitch::{NewTask, Task, TaskId};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TaskQuery {
    filter: Option<String>,
}

impl ResponseError for cs::ContextswitchError {
    fn status_code(&self) -> StatusCode {
        match self {
            cs::ContextswitchError::InvalidDataError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            cs::ContextswitchError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[tracing::instrument(level = "debug", skip_all, fields(filter = %task_query.filter.as_ref().unwrap_or(&"".to_string())))]
pub async fn list_tasks(
    task_query: web::Query<TaskQuery>,
) -> Result<HttpResponse, cs::ContextswitchError> {
    let filter = task_query
        .filter
        .as_ref()
        .map_or(vec![], |filter| filter.split(' ').collect());
    let tasks: Vec<Task> = cs::list_tasks(filter)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&tasks).context("Cannot serialize Contextswitch task")?))
}

#[tracing::instrument(level = "debug", skip_all, fields(definition = %new_task.definition))]
pub async fn add_task(
    new_task: web::Json<NewTask>,
) -> Result<HttpResponse, cs::ContextswitchError> {
    let task: Task = cs::add_task(new_task.definition.split(' ').collect()).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&task).context("Cannot serialize Contextswitch task")?))
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn update_task(
    path: web::Path<TaskId>,
    task: web::Json<Task>,
) -> Result<HttpResponse, cs::ContextswitchError> {
    let task_to_update = task.into_inner();
    if path.into_inner() != task_to_update.id {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let task_updated: Task = cs::update_task(task_to_update).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&task_updated).context("Cannot serialize Contextswitch task")?))
}

#[tracing::instrument(level = "debug")]
pub async fn option_task() -> HttpResponse {
    HttpResponse::Ok().finish()
}
