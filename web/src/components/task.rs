use contextswitch;
use uikit_rs as uk;
use yew::{classes, function_component, html, Callback, Classes, Html, MouseEvent, Properties};

#[derive(Properties, PartialEq)]
pub struct TaskProps {
    pub task: contextswitch::Task,
    #[prop_or_default]
    pub selected: bool,
    #[prop_or_default]
    pub on_task_select: Callback<Option<contextswitch::Task>>,
}

#[function_component(Task)]
pub fn task(
    TaskProps {
        task,
        selected,
        on_task_select,
    }: &TaskProps,
) -> Html {
    let toggle_details = {
        let task = task.clone();
        let on_task_select = on_task_select.clone();
        let is_task_selected = *selected;
        Callback::from(move |_| {
            on_task_select.emit(if is_task_selected {
                None
            } else {
                Some(task.clone())
            })
        })
    };

    let text_style = if task.status == contextswitch::Status::Completed {
        uk::Text::Success
    } else {
        uk::Text::Emphasis
    };
    let arrow = if *selected {
        uk::IconType::TriangleDown
    } else {
        uk::IconType::TriangleRight
    };

    let task_status_class: Classes = format!("task-status-{}", task.status).into();
    let bookmark_count = if let Some(contextswitch) = &task.contextswitch {
        contextswitch.bookmarks.len()
    } else {
        0
    };

    html! {
        <uk::Card size={uk::CardSize::Small}
                  style={uk::CardStyle::Default}
                  hover={true}
                  width={uk::Width::_1_1}
                  class={task_status_class}>
          <uk::CardBody padding={vec![uk::Padding::RemoveVertical]}
                        margin={vec![uk::Margin::SmallTop, uk::Margin::SmallBottom]}>
            <uk::Grid gap_size={uk::GridGapSize::Small}
                      vertical_alignement={uk::FlexVerticalAlignement::Middle}>
              <uk::Icon icon_type={uk::IconType::Check}
                        text_style={vec![text_style]} />
              <TaskDescription task={task.clone()}
                               onclick={toggle_details.clone()} />
              <uk::IconNav>
                <li>
                  <uk::Icon icon_type={uk::IconType::FileEdit} href="#" />
                </li>
                <li>
                  <uk::Link href="#" onclick={toggle_details.clone()}>
                    <uk::Icon icon_type={uk::IconType::Bookmark} />
                    <span> {bookmark_count}</span>
                  </uk::Link>
                </li>
              </uk::IconNav>
              <uk::Icon icon_type={arrow} href="#" onclick={toggle_details} />
            </uk::Grid>
            {
                if *selected {
                    html! {
                        <TaskDetails task={task.clone()} />
                    }
                } else { html! {} }
            }
          </uk::CardBody>
        </uk::Card>
    }
}

#[derive(Properties, PartialEq)]
pub struct TaskDescriptionProps {
    pub task: contextswitch::Task,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

#[function_component(TaskDescription)]
pub fn task_description(TaskDescriptionProps { task, onclick }: &TaskDescriptionProps) -> Html {
    html! {
        <uk::Flex width={uk::Width::_Expand} onclick={onclick}>
          <uk::CardTitle text_style={vec![uk::Text::Lighter]}>
          {task.description.clone()}
          </uk::CardTitle>
        </uk::Flex>
    }
}

#[derive(Properties, PartialEq)]
pub struct TaskDetailsProps {
    pub task: contextswitch::Task,
}

#[function_component(TaskDetails)]
pub fn task_details(TaskDetailsProps { task }: &TaskDetailsProps) -> Html {
    let priority = task
        .priority
        .as_ref()
        .map(|prio| prio.to_string())
        .unwrap_or_else(|| "-".to_string());
    let project = task
        .project
        .as_ref()
        .map(|proj| proj.to_string())
        .unwrap_or_else(|| "-".to_string());

    html! {
        <div class={classes!(uk::Margin::Small)}>
            <uk::Divider margin={vec![uk::Margin::Small]} />
            {
                if let Some(contextswitch) = &task.contextswitch {
                    html! {
                        <TaskContextswitch contextswitch={contextswitch.clone()} />
                    }
                } else { html! {} }
            }
          <uk::Grid gap_size={uk::GridGapSize::Small}
                    child_width={uk::ChildWidth::_Expand}>
            <span class={classes!(uk::Text::Meta)}>
              { format!("priority: {}", priority) }
            </span>
            <span class={classes!(uk::Text::Meta)}>
              { format!("project: {}", project) }
            </span>
          </uk::Grid>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TaskContextswitchProps {
    pub contextswitch: contextswitch::ContextswitchData,
}

#[function_component(TaskContextswitch)]
pub fn task_contextswitch(
    TaskContextswitchProps { contextswitch }: &TaskContextswitchProps,
) -> Html {
    html! {
        <uk::Grid gap_size={uk::GridGapSize::Small} height_match={true}>
          {
              contextswitch.bookmarks.iter().map(|bookmark| {
                  html! {
                      <TaskBookmark bookmark={bookmark.clone()} />
                  }
              }).collect::<Html>()
          }
          <uk::Icon icon_type={uk::IconType::Plus}
                    margin={vec![uk::Margin::Remove]} />
        </uk::Grid>
    }
}

#[derive(Properties, PartialEq)]
pub struct TaskBookmarkProps {
    pub bookmark: contextswitch::Bookmark,
}

#[function_component(TaskBookmark)]
pub fn task_bookmark(TaskBookmarkProps { bookmark }: &TaskBookmarkProps) -> Html {
    html! {
        <div class={classes!(uk::Width::_1_1, uk::Text::Small, uk::Margin::Remove)}>
            <uk::Grid gap_size={uk::GridGapSize::Small} vertical_alignement={uk::FlexVerticalAlignement::Middle}>
              <uk::Icon icon_type={uk::IconType::Bookmark} />
              <uk::Link href={bookmark.uri.to_string()}>{bookmark.uri.to_string()}</uk::Link>
            </uk::Grid>
        </div>
    }
}
