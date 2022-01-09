pub mod test_helper;

#[actix_rt::test]
async fn health_check_works() {
    let address = test_helper::spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/ping", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}