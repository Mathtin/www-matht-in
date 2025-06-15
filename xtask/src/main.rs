mod core_dist;
mod paths;
mod web_dist;


use core_dist::{make_all_directories, BUILD_PATH, TaskResult};
use web_dist::{
    build_web_distribution, 
    build_web_distribution_dev,
    serve_web_distribution,
    serve_web_distribution_dev
};
use std::{
    collections::HashMap, env, io::Error, sync::LazyLock
};


type TaskFn = fn() -> TaskResult;
type TaskKey<'a> = &'a str;
type TaskDescription<'a> = &'a str;

type TasksEntry<'a> = (TaskFn, TaskDescription<'a>);
type TasksListEntry<'a> = (TaskKey<'a>, TasksEntry<'a>);
type TasksCompactMap<'a> = HashMap<TaskKey<'a>, &'a TaskFn>;

static TASKS_LIST: &[TasksListEntry] = &[
    ("build-web-dist",          (build_web_distribution,        "build distribution for web (HTML + WASM)")),
    ("serve-web-dist",          (serve_web_distribution,        "serve distribution for web on http://127.0.0.1:8080")),
    ("build-windows-dist",      (todo_placeholder,              "build distribution for windows (x64 exe)")),
    ("build-android-dist",      (todo_placeholder,              "build distribution for android (arm64 apk)")),

    ("build-web-dist-dev",      (build_web_distribution_dev,    "build developer distribution for web (HTML + WASM)")),
    ("serve-web-dist-dev",      (serve_web_distribution_dev,    "serve developer distribution for web on http://127.0.0.1:8080")),
    ("build-windows-dist-dev",  (todo_placeholder,              "build developer distribution for windows (x64 exe)")),
    ("build-android-dist-dev",  (todo_placeholder,              "build developer distribution for android (arm64 apk)")),

    ("help",                    (print_help,                    "print help (this) message")),
];

static TASKS_MAP: LazyLock<TasksCompactMap> = LazyLock::new(||
    TASKS_LIST.iter()
        .map(|(cmd, (f, _))| (*cmd, f))
        .collect()
);


fn main() {
    log::init_log();
    if let Err(e) = try_main() {
        log::error!("{}", e);
        std::process::exit(-1);
    }
}


fn try_main() -> Result<(), Error> {
    make_all_directories(&BUILD_PATH)?;

    let chosen_task = env::args().nth(1);

    match chosen_task.as_deref() {
        Some(task_name) => (
            TASKS_MAP.get(task_name).unwrap_or(&&(print_help as TaskFn))
        )()?, // get task from map and call
        _ => print_help()?,
    }
    
    Ok(())
}


fn print_help() -> Result<(), Error> {
    let max_task_name_length = TASKS_LIST.iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or(0);
    let task_width = max_task_name_length - (max_task_name_length % 4) + 8;
    let task_table = TASKS_LIST.iter()
        .map(|(name, (_, desc))| format!("{:width$}{}", name, desc, width=task_width))
        .fold(String::new(), |line1, line2| line1 + "\n" + &line2);

    eprintln!("Tasks:\n{}", task_table);

    Ok(())
}


fn todo_placeholder() -> Result<(), Error> {
    todo!()
}
