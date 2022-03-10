use crate::helpers::app_address;
use contextswitch::{Bookmark, ContextswitchData, NewTask, Task};
use contextswitch_api::contextswitch as cs;
use http::uri::Uri;
use rstest::*;

mod list_tasks {
    use super::*;

    #[rstest]
    #[tokio::test]
    async fn list_tasks(app_address: &str) {
        let task = cs::add_task(vec![
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
    async fn list_tasks_with_unknown_cs_data(app_address: &str) {
        let task = cs::add_task(vec![
            "test",
            "list_tasks_with_unknown_cs_data",
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
        assert_eq!(tasks[0].description, "test list_tasks_with_unknown_cs_data");
        assert!(tasks[0].contextswitch.is_none());
    }

    #[rstest]
    #[tokio::test]
    async fn list_tasks_with_invalid_cs_data(app_address: &str) {
        let task = cs::add_task(vec![
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
}

mod add_task {
    use super::*;

    #[rstest]
    #[tokio::test]
    async fn add_task(app_address: &str) {
        let task: Task = reqwest::Client::new()
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

        assert_eq!(task.description, "test add_task");
        assert_eq!(
            task.contextswitch.as_ref().unwrap(),
            &ContextswitchData {
                bookmarks: vec![Bookmark {
                    uri: "https://example.com/path?filter=1".parse::<Uri>().unwrap(),
                    content: None
                }]
            }
        );
    }
}

mod update_task {
    use super::*;

    #[rstest]
    #[tokio::test]
    async fn update_task(app_address: &str) {
        let mut task = cs::add_task(vec![
            "test",
            "update_task",
            "contextswitch:'{\"bookmarks\":[{\"uri\":\"https://example.com/path?filter=1\"}]}'",
        ])
        .await
        .unwrap();

        task.description = "updated task description".to_string();
        let cs_data = task.contextswitch.as_mut().unwrap();
        cs_data.bookmarks.push(Bookmark {
            uri: "https://example.com/path2".parse::<Uri>().unwrap(),
            content: None,
        });

        let updated_task: Task = reqwest::Client::new()
            .put(&format!("{}/tasks/{}", &app_address, task.id))
            .json(&task)
            .send()
            .await
            .expect("Failed to execute request")
            .json()
            .await
            .expect("Cannot parse JSON result");

        assert_eq!(updated_task.description, "updated task description");
        assert_eq!(
            updated_task.contextswitch.as_ref().unwrap(),
            &ContextswitchData {
                bookmarks: vec![
                    Bookmark {
                        uri: "https://example.com/path?filter=1".parse::<Uri>().unwrap(),
                        content: None
                    },
                    Bookmark {
                        uri: "https://example.com/path2".parse::<Uri>().unwrap(),
                        content: None
                    }
                ]
            }
        );
    }

    // TODO : test incoherent task id
}
