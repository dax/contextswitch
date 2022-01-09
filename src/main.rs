extern crate dotenv;
extern crate env_logger;
extern crate listenfd;

use contextswitch_api::run;
use dotenv::dotenv;
use std::env;
use std::net::TcpListener;

pub mod taskwarrior;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    dotenv().ok();

    let port = env::var("PORT").unwrap_or("8000".to_string());
    taskwarrior::load_config(None);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).expect("Failed to bind port");
    run(listener)?.await
}
