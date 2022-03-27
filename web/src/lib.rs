use components::tasks_list::TasksList;
use contextswitch::Task;
use reqwasm::http::Request;
use uikit_rs as uk;
use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod components;

#[wasm_bindgen(module = "/js/api.js")]
extern "C" {
    fn get_api_base_url() -> String;
}

#[function_component(App)]
pub fn app() -> Html {
    let tasks = use_state(Vec::new);
    {
        let tasks = tasks.clone();
        use_effect_with_deps(
            move |_| {
                wasm_bindgen_futures::spawn_local(async move {
                    let fetched_tasks: Vec<Task> =
                        Request::get(&format!("{}/tasks?filter=task", get_api_base_url()))
                            .send()
                            .await
                            .unwrap() // TODO
                            .json()
                            .await
                            .unwrap(); // TODO
                    tasks.set(fetched_tasks);
                });
                || ()
            },
            (),
        );
    }
    let selected_task = use_state(|| None);
    let on_task_select = {
        let selected_task = selected_task.clone();
        Callback::from(move |task: Option<Task>| {
            selected_task.set(task);
        })
    };

    html! {
        <uk::Section style={uk::SectionStyle::Default}>
          <uk::Container size={uk::ContainerSize::Small}>
            <uk::Filter target=".status-filter"
              filter_width={uk::Width::_Expand}
              filter_component={uk::UIKitComponent::SubNav}
              filter_class={"uk-subnav-pill"}
              filters={vec![uk::FilterData { class: "".to_string(), label: "all".to_string() },
                            uk::FilterData { class: ".task-status-pending".to_string(), label: "pending".to_string() },
                            uk::FilterData { class: ".task-status-completed".to_string(), label: "completed".to_string() }]}>
              <uk::Grid gap_size={uk::GridGapSize::Small}
                        margin={vec![uk::Margin::Default]}
                        height_match={true}
                        class={"status-filter"}>
                <TasksList tasks={(*tasks).clone()}
                           selected_task={(*selected_task).clone()}
                           on_task_select={on_task_select} />
              </uk::Grid>
            </uk::Filter>
          </uk::Container>
        </uk::Section>
    }
}
