use crate::helpers::app_address;
use contextswitch_api::contextswitch;
use contextswitch_types::{Bookmark, ContextswitchData, NewTask, Task, TaskId};
use http::uri::Uri;
use rstest::*;
use uuid::Uuid;

#[rstest]
#[tokio::test]
async fn list_tasks(app_address: &str) {
    let task = contextswitch::add_task(vec![
        "test",
        "list_tasks",
        "contextswitch:'{\"bookmarks\":[{\"uri\":\"https://example.com/path?filter=1\"}]}'",
    ])
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
    let cs_data = tasks[0].contextswitch.as_ref().unwrap();
    assert_eq!(cs_data.bookmarks.len(), 1);
    assert_eq!(cs_data.bookmarks[0].content, None);
    assert_eq!(
        cs_data.bookmarks[0].uri,
        "https://example.com/path?filter=1".parse::<Uri>().unwrap()
    );
}

#[rstest]
#[tokio::test]
async fn list_tasks_with_unknown_contextswitch_data(app_address: &str) {
    let task = contextswitch::add_task(vec![
        "test",
        "list_tasks_with_unknown_contextswitch_data",
        "contextswitch:'{\"unknown\": 1}'",
    ])
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
    assert_eq!(
        tasks[0].description,
        "test list_tasks_with_unknown_contextswitch_data"
    );
    assert!(tasks[0].contextswitch.is_none());
}

#[rstest]
#[tokio::test]
async fn list_tasks_with_invalid_contextswitch_data(app_address: &str) {
    let task = contextswitch::add_task(vec![
        "test",
        "list_tasks_with_invalid_contextswitch_data",
        "contextswitch:'}'",
    ])
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
    assert_eq!(
        tasks[0].description,
        "test list_tasks_with_invalid_contextswitch_data"
    );
    assert!(tasks[0].contextswitch.is_none());
}

#[rstest]
#[tokio::test]
async fn add_task(app_address: &str) {
    let response: serde_json::Value = reqwest::Client::new()
        .post(&format!("{}/tasks", &app_address))
        .json(&NewTask {
            definition:
            "test add_task contextswitch:{\"bookmarks\":[{\"uri\":\"https://example.com/path?filter=1\"}]}"
                    .to_string(),
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
        &ContextswitchData {
            bookmarks: vec![Bookmark {
                uri: "https://example.com/path?filter=1".parse::<Uri>().unwrap(),
                content: None
            }]
        }
    );
}
