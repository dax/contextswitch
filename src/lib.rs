use actix_web::{dev::Server, http, middleware, web, App, HttpResponse, HttpServer};
use contextswitch_types::TaskDefinition;
use listenfd::ListenFd;
use serde::Deserialize;
use serde_json::json;
use std::env;
use std::io::Error;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

#[macro_use]
extern crate lazy_static;

pub mod contextswitch;
pub mod observability;
pub mod taskwarrior;

#[derive(Deserialize)]
struct TaskQuery {
    filter: Option<String>,
}

#[tracing::instrument(level = "debug", skip_all, fields(filter = %task_query.filter.as_ref().unwrap_or(&"".to_string())))]
async fn list_tasks(task_query: web::Query<TaskQuery>) -> Result<HttpResponse, Error> {
    let filter = task_query
        .filter
        .as_ref()
        .map_or(vec![], |filter| filter.split(' ').collect());
    let tasks = contextswitch::export(filter)?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&tasks)?))
}

#[tracing::instrument(level = "debug", skip_all, fields(definition = %task_definition.definition))]
async fn add_task(task_definition: web::Json<TaskDefinition>) -> Result<HttpResponse, Error> {
    let task_id = contextswitch::add(task_definition.definition.split(' ').collect()).await?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(json!({ "id": task_id }).to_string()))
}

async fn option_task() -> HttpResponse {
    HttpResponse::Ok().finish()
}

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let cs_front_base_url =
        env::var("CS_FRONT_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let mut server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .wrap(middleware::Compress::default())
            .wrap(
                middleware::DefaultHeaders::new()
                    .add(("Access-Control-Allow-Origin", cs_front_base_url.as_bytes()))
                    .add((
                        "Access-Control-Allow-Methods",
                        "POST, GET, OPTIONS".as_bytes(),
                    ))
                    .add(("Access-Control-Allow-Headers", "content-type".as_bytes())),
            )
            .route("/ping", web::get().to(health_check))
            .route("/tasks", web::get().to(list_tasks))
            .route("/tasks", web::post().to(add_task))
            .route("/tasks", web::method(http::Method::OPTIONS).to(option_task))
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
