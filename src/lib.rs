use actix_web::{dev::Server, middleware::Logger, web, App, HttpResponse, HttpServer};
use chrono::{DateTime, Utc};
use listenfd::ListenFd;
use serde::{Deserialize, Serialize};
use std::io::Error;
use std::net::TcpListener;
use uuid::Uuid;

pub mod taskwarrior;

#[derive(Deserialize)]
struct TaskQuery {
    filter: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub uuid: Uuid,
    pub id: u32,
    #[serde(with = "taskwarrior::tw_date_format")]
    pub entry: DateTime<Utc>,
    #[serde(with = "taskwarrior::tw_date_format")]
    pub modified: DateTime<Utc>,
    pub status: taskwarrior::Status,
    pub description: String,
    pub urgency: f64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "taskwarrior::opt_tw_date_format"
    )]
    pub due: Option<DateTime<Utc>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "taskwarrior::opt_tw_date_format"
    )]
    pub end: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recur: Option<taskwarrior::Recurrence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contextswitch: Option<taskwarrior::ContextSwitchMetadata>,
}

async fn list_tasks(task_query: web::Query<TaskQuery>) -> Result<HttpResponse, Error> {
    let tasks = taskwarrior::export(task_query.filter.split(' ').collect())?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&tasks)?))
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let mut server = HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/ping", web::get().to(health_check))
            .route("/tasks", web::get().to(list_tasks))
    })
    .keep_alive(60)
    .shutdown_timeout(60);

    let mut listenfd = ListenFd::from_env();

    server = if let Some(fdlistener) = listenfd.take_tcp_listener(0)? {
        server.listen(fdlistener)?
    } else {
        server.listen(listener)?
    };

    Ok(server.run())
}
