pub mod test_helper;

use contextswitch_api::taskwarrior;
use contextswitch_types::Task;
use contextswitch_types::TaskDefinition;
use rstest::*;
use test_helper::app_address;

#[rstest]
#[tokio::test]
async fn list_tasks(app_address: &str) {
    let task_id = taskwarrior::add(vec!["test", "list_tasks", "contextswitch:'{\"test\": 1}'"])
        .await
        .unwrap();

    println!("LIST TASKS ID: {}", task_id);
    let tasks: Vec<Task> = reqwest::Client::new()
        .get(&format!("{}/tasks?filter={}", &app_address, task_id))
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
        .json(&TaskDefinition {
            definition: "test add_task contextswitch:{\"test\":1}".to_string(),
        })
        .send()
        .await
        .expect("Failed to execute request")
        .json()
        .await
        .expect("Cannot parse JSON result");
    println!("ADD RESPONSE: {:?}", response);
    let new_task_id = response["id"].as_u64().unwrap();
    let tasks = taskwarrior::export(vec![&new_task_id.to_string()]).unwrap();
    println!("TASKS={:?}", tasks);

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, new_task_id);
    assert_eq!(tasks[0].description, "test add_task");
    assert_eq!(
        tasks[0].contextswitch.as_ref().unwrap(),
        &"{\"test\":1}".to_string()
    );
}
