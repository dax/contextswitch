extern crate dotenv;
extern crate listenfd;

use contextswitch_api::observability::{get_subscriber, init_subscriber};
use contextswitch_api::{run, taskwarrior};
use dotenv::dotenv;
use std::env;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("info".into());
    init_subscriber(subscriber);

    dotenv().ok();

    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    taskwarrior::load_config(None);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).expect("Failed to bind port");
    run(listener)?.await
}
