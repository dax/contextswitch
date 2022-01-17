pub mod test_helper;
use rstest::*;
use test_helper::app_address;

#[rstest]
#[tokio::test]
async fn health_check_works(app_address: &str) {
    let response = reqwest::Client::new()
        .get(&format!("{}/ping", &app_address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
