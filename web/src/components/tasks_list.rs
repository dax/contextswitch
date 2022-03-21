use crate::components::task;
use contextswitch::Task;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TasksListProps {
    #[prop_or_default]
    pub tasks: Vec<Task>,
    #[prop_or_default]
    pub selected_task: Option<Task>,
    #[prop_or_default]
    pub on_task_select: Callback<Option<Task>>,
}

#[function_component(TasksList)]
pub fn tasks_list(
    TasksListProps {
        tasks,
        selected_task,
        on_task_select,
    }: &TasksListProps,
) -> Html {
    tasks
        .iter()
        .map(|task| {
            let task_is_selected = selected_task
                .clone()
                .map(|t| t.id == task.id)
                .unwrap_or(false);

            html! {
                <task::Task selected={task_is_selected}
                            on_task_select={on_task_select}
                            task={task.clone()} />
            }
        })
        .collect()
}
