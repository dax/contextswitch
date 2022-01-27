pub mod test_helper;

use contextswitch_api::contextswitch;
use contextswitch_types::{ContextSwitchMetadata, NewTask, Task, TaskId};
use rstest::*;
use test_helper::app_address;
use uuid::Uuid;

#[rstest]
#[tokio::test]
async fn list_tasks(app_address: &str) {
    let task = contextswitch::add_task(vec!["test", "list_tasks", "contextswitch:'{\"test\": 1}'"])
        .await
        .unwrap();

    let tasks: Vec<Task> = reqwest::Client::new()
        .get(&format!("{}/tasks?filter={}", &app_address, task.id))
        .send()
        .await
        .expect("Failed to execute request")
        .json()
        .await
        .expect("Cannot parse JSON result");

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].description, "test list_tasks");
    let cs_metadata = tasks[0].contextswitch.as_ref().unwrap();
    assert_eq!(cs_metadata.test, 1);
}

#[rstest]
#[tokio::test]
async fn add_task(app_address: &str) {
    let response: serde_json::Value = reqwest::Client::new()
        .post(&format!("{}/tasks", &app_address))
        .json(&NewTask {
            definition: "test add_task contextswitch:{\"test\":1}".to_string(),
        })
        .send()
        .await
        .expect("Failed to execute request")
        .json()
        .await
        .expect("Cannot parse JSON result");
    let new_task_id = TaskId(Uuid::parse_str(response["id"].as_str().unwrap()).unwrap());
    let tasks = contextswitch::list_tasks(vec![&new_task_id.to_string()]).unwrap();

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, new_task_id);
    assert_eq!(tasks[0].description, "test add_task");
    assert_eq!(
        tasks[0].contextswitch.as_ref().unwrap(),
        &ContextSwitchMetadata { test: 1 }
    );
}
