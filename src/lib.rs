use actix_web::{dev::Server, http, middleware, web, App, HttpServer};
use listenfd::ListenFd;
use std::env;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

#[macro_use]
extern crate lazy_static;

pub mod configuration;
pub mod contextswitch;
pub mod observability;
pub mod routes;

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
            .route("/ping", web::get().to(routes::ping))
            .route("/tasks", web::get().to(routes::list_tasks))
            .route("/tasks", web::post().to(routes::add_task))
            .route(
                "/tasks",
                web::method(http::Method::OPTIONS).to(routes::option_task),
            )
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
