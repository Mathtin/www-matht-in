pub mod core_dist;
pub mod paths;
pub mod web_dist;

use core_dist::{OK, TaskResult, make_each_directory};
use std::{collections::HashMap, env};
use web_dist::{
    build_web_distribution, build_web_distribution_dev,
    prepare_serve_web_distribution, prepare_serve_web_distribution_dev,
    serve_web_distribution, serve_web_distribution_dev,
};

////////////////////////////////////////////////////////////////////////////////
// CLI Task Bindings
////////////////////////////////////////////////////////////////////////////////

type TaskFn = fn() -> TaskResult;
type TaskKey = &'static str;
type TaskDescription = &'static str;
type TasksListEntry = (TaskKey, TaskFn, TaskDescription);

static TASKS_LIST: &[TasksListEntry] = &[
    (
        "build-web-dist",
        build_web_distribution,
        "build distribution for web (HTML + WASM)",
    ),
    (
        "serve-web-dist",
        serve_web_distribution,
        "serve distribution for web on http://127.0.0.1:8080 (via simple-http-server)",
    ),
    (
        "build-windows-dist",
        todo_placeholder,
        "build distribution for windows (x64 exe)",
    ),
    (
        "build-android-dist",
        todo_placeholder,
        "build distribution for android (arm64 apk)",
    ),
    (
        "build-web-dist-dev",
        build_web_distribution_dev,
        "build developer distribution for web (HTML + WASM)",
    ),
    (
        "serve-web-dist-dev",
        serve_web_distribution_dev,
        "serve developer distribution for web on http://127.0.0.1:8080 (via simple-http-server)",
    ),
    (
        "build-windows-dist-dev",
        todo_placeholder,
        "build developer distribution for windows (x64 exe)",
    ),
    (
        "build-android-dist-dev",
        todo_placeholder,
        "build developer distribution for android (arm64 apk)",
    ),
    // For VS launch "Serve (Dev) Web Package"
    (
        "prepare-serve-web-dist",
        prepare_serve_web_distribution,
        "build distribution for web and install simple-http-server",
    ),
    (
        "prepare-serve-web-dist-dev",
        prepare_serve_web_distribution_dev,
        "build developer distribution for web and install simple-http-server",
    ),
    ("help", print_help, "print help (this) message"),
];

////////////////////////////////////////////////////////////////////////////////
// CLI Task Runner
////////////////////////////////////////////////////////////////////////////////

pub fn run() -> TaskResult {
    make_each_directory(&paths::BUILD_PATH)?;

    let tasks_map: HashMap<_, _> =
        TASKS_LIST.iter().map(|(cmd, f, _)| (*cmd, f)).collect();
    let fallback_task = &(print_help as TaskFn);

    let chosen_task = match env::args().nth(1) {
        Some(task_name) => {
            tasks_map.get(task_name.as_str()).unwrap_or(&fallback_task)
        }
        None => fallback_task,
    };

    chosen_task()
}

////////////////////////////////////////////////////////////////////////////////
// CLI Tasks
////////////////////////////////////////////////////////////////////////////////

fn print_help() -> TaskResult {
    const TAB_SIZE: usize = 4;
    let max_task_name_length = TASKS_LIST
        .iter()
        .map(|(name, _, __)| name.len())
        .max()
        .unwrap_or_default();
    let task_width =
        max_task_name_length - (max_task_name_length % TAB_SIZE) + 2 * TAB_SIZE;
    let task_table = TASKS_LIST
        .iter()
        .map(|(name, _, desc)| {
            format!("{:width$}{}", name, desc, width = task_width)
        })
        .fold(String::new(), |line1, line2| line1 + "\n" + &line2);

    eprintln!("Available Tasks:\n{}", task_table);

    OK
}

fn todo_placeholder() -> TaskResult {
    todo!()
}
