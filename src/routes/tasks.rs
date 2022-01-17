use crate::contextswitch;
use actix_web::{web, HttpResponse};
use contextswitch_types::TaskDefinition;
use serde::Deserialize;
use serde_json::json;
use std::io::Error;

#[derive(Deserialize)]
pub struct TaskQuery {
    filter: Option<String>,
}

#[tracing::instrument(level = "debug", skip_all, fields(filter = %task_query.filter.as_ref().unwrap_or(&"".to_string())))]
pub async fn list_tasks(task_query: web::Query<TaskQuery>) -> Result<HttpResponse, Error> {
    let filter = task_query
        .filter
        .as_ref()
        .map_or(vec![], |filter| filter.split(' ').collect());
    let tasks = contextswitch::list_tasks(filter)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&tasks)?))
}

#[tracing::instrument(level = "debug", skip_all, fields(definition = %task_definition.definition))]
pub async fn add_task(task_definition: web::Json<TaskDefinition>) -> Result<HttpResponse, Error> {
    let task_id = contextswitch::add_task(task_definition.definition.split(' ').collect()).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(json!({ "id": task_id }).to_string()))
}

#[tracing::instrument(level = "debug")]
pub fn option_task() -> HttpResponse {
    HttpResponse::Ok().finish()
}
