pub mod test_helper;

use contextswitch_api::taskwarrior;
use contextswitch_types::Task;

#[actix_rt::test]
async fn list_tasks() {
    let task_data_path = test_helper::setup_tasks();
    let address = test_helper::spawn_app();
    let client = reqwest::Client::new();
    taskwarrior::add(vec!["test1", "contextswitch:'{\"test\": 1}'"]).unwrap();

    let response = client
        .get(&format!("{}/tasks?filter=ls", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    test_helper::clear_tasks(task_data_path);
    let text_body = response.text_with_charset("utf-8").await.unwrap();
    let tasks: Vec<Task> = serde_json::from_str(&text_body).unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].description, "test1");

    let cs_metadata = tasks[0].contextswitch.as_ref().unwrap();
    assert_eq!(cs_metadata.test, 1);
}