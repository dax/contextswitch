use actix_web::{dev::Server, middleware, web, App, HttpResponse, HttpServer};
use listenfd::ListenFd;
use serde::Deserialize;
use std::env;
use std::io::Error;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub mod contextswitch;
pub mod observability;
pub mod taskwarrior;

#[derive(Deserialize)]
struct TaskQuery {
    filter: String,
}

#[tracing::instrument(level = "debug", skip(task_query))]
async fn list_tasks(task_query: web::Query<TaskQuery>) -> Result<HttpResponse, Error> {
    let tasks = contextswitch::export(task_query.filter.split(' ').collect())?;

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&tasks)?))
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
                    .add(("Access-Control-Allow-Origin", cs_front_base_url.as_bytes())),
            )
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
