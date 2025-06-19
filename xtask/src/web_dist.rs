use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;

use crate::core_dist::{
    cargo, contains_any_extension, copy_file_tree_filtered, cut_path_children, for_each_file_recursively, make_each_directory, relative_path_depth, shell_log_piped, transparent_shell, TaskResult
};
use crate::paths;


const HTTP_SERVER: &str = "simple-http-server";
const WASM_PACK: &str = "wasm-pack";
const MINHTML: &str = "minhtml";
const FRONT_PAGE_DIR: &str = "front-page";
const ERROR_PAGE_SUBDIR: &str = "error_pages";
const HIDDEN_ERROR_PAGE_DIR: &str = ".error_pages";

const MINIFY_EXTENSIONS: &[&[u8]] = &[b"html", b"css", b"js"];

const MINIFY_DIR_RENAMING: LazyLock<[(PathBuf, PathBuf); 1]> = LazyLock::new(|| [
    (paths::PROJECT_ROOT.join(FRONT_PAGE_DIR).join(ERROR_PAGE_SUBDIR), PathBuf::from(HIDDEN_ERROR_PAGE_DIR)),
]);


pub fn build_web_distribution() -> TaskResult {
    let web_dist_path = paths::BUILD_PATH.join(paths::WEB_DIST_SUBDIRECTORY);
    let wasm_pkg_path = paths::BUILD_PATH.join(paths::WASM_PKG_SUBDIRECTORY);
    build_web_distribution_by_path(&web_dist_path, &wasm_pkg_path, true)
}


pub fn serve_web_distribution() -> TaskResult {
    prepare_serve_web_distribution()?;
    let web_dist_path = paths::BUILD_PATH.join(paths::WEB_DIST_SUBDIRECTORY);
    serve_web_distribution_by_path(&web_dist_path)
}


pub fn prepare_serve_web_distribution() -> TaskResult {
    build_web_distribution()?;
    cargo(&["install", HTTP_SERVER])?;
    Ok(())
}


pub fn build_web_distribution_dev() -> TaskResult {
    let web_dist_path = paths::BUILD_PATH.join(paths::WEB_DIST_DEV_SUBDIRECTORY);
    let wasm_pkg_path = paths::BUILD_PATH.join(paths::WASM_PKG_DEV_SUBDIRECTORY);
    build_web_distribution_by_path(&web_dist_path, &wasm_pkg_path, false)
}


pub fn serve_web_distribution_dev() -> TaskResult {
    prepare_serve_web_distribution_dev()?;
    let web_dist_path = paths::BUILD_PATH.join(paths::WEB_DIST_DEV_SUBDIRECTORY);
    serve_web_distribution_by_path(&web_dist_path)
}


pub fn prepare_serve_web_distribution_dev() -> TaskResult {
    build_web_distribution_dev()?;
    cargo(&["install", HTTP_SERVER])?;
    Ok(())
}


fn build_web_distribution_by_path(
    web_dist_path: &Path,
    wasm_pkg_path: &Path,
    release: bool,
) -> TaskResult {
    cargo(&["install", WASM_PACK])?;

    make_each_directory(&web_dist_path)?;

    let wasm_pkg_path_arg = wasm_pkg_path.to_string_lossy();
    let wasm_pack_args: &[&str] = if release {
        &["--verbose", "build", "shards-browser", "--target", "web", "--out-dir", &wasm_pkg_path_arg]
    } else {
        &["--verbose", "build", "shards-browser", "--dev", "--target", "web", "--out-dir", &wasm_pkg_path_arg]
    };

    shell_log_piped(
        "wasm-pack", 
        wasm_pack_args, 
        &[("RUSTFLAGS", "-Ctarget-cpu=mvp")]
    )?;

    let front_page_path = paths::PROJECT_ROOT.join(FRONT_PAGE_DIR);

    if release {
        cargo(&["install", MINHTML])?;
        // copy all wasm modules and js-bindings
        copy_file_tree_filtered(wasm_pkg_path, web_dist_path, &[b"js", b"wasm"], true)?;
        // minify front-page
        minify_swarm(&front_page_path, web_dist_path)?;
        // copy rest from front-page
        copy_file_tree_filtered(&front_page_path, web_dist_path, MINIFY_EXTENSIONS, false)?;
    } else {
        // copy all wasm modules and js-bindings
        copy_file_tree_filtered(wasm_pkg_path, web_dist_path, &[b"js", b"wasm"], true)?;
        // copy whole front-page
        copy_file_tree_filtered(&front_page_path, web_dist_path, &[], false)?;
    }

    log::info!("[xtask] Done! Check: {}", web_dist_path.display());
    Ok(())
}


fn serve_web_distribution_by_path(web_dist_path: &Path) -> TaskResult {
    let web_dist_path_str = web_dist_path.to_string_lossy();

    transparent_shell(
        HTTP_SERVER, 
        &["--nocache", "-i", "-p", "8080", "--ip", "127.0.0.1", &web_dist_path_str], 
        &[]
    )?;

    Ok(())
}


fn minify_swarm(input: &Path, output: &Path) -> TaskResult {
    let available_parallelism = thread::available_parallelism()?.get();
    assert!(available_parallelism > 0, "0 parallelism?!");

    type MinifyTask = (PathBuf, PathBuf);
    type MinifyTaskSender = mpsc::SyncSender<MinifyTask>;
    type MinifyTaskReceiver = mpsc::Receiver<MinifyTask>;
    type MinifyPipe = (MinifyTaskSender, MinifyTaskReceiver);
    let pipes: Vec<MinifyPipe> = (0..available_parallelism).map(|_| mpsc::sync_channel(1)).collect();

    thread::scope(|s| {

        let pool: Vec<(mpsc::SyncSender<MinifyTask>, _)> = pipes.into_iter()
            .map(|(sender, receiver)| (sender, s.spawn(
                // thread listening pipe
                move || while let Ok((input, output)) = receiver.recv() {
                    if let Err(e) = minify(&input, &output) {
                        log::error!("[xtask] failed to call minify {} > {}: {}", input.display(), output.display(), e);
                    }
                })
            ))
            .collect();

        for_each_file_recursively(input, |relative_path| {
            if ! contains_any_extension(relative_path, MINIFY_EXTENSIONS) {
                return;
            }

            let from_path = input.join(relative_path);
            let dest_path = output.join(relative_path);

            let try_send = move |(p, _): &(MinifyTaskSender, _)| p.try_send((from_path.clone(), dest_path.clone())).is_ok();
            // try push loop
            while ! pool.iter().any(&try_send)
            {
                thread::sleep(Duration::from_millis(1));
            };
        })

    })
}


fn minify(full_input: &Path, full_output: &Path) -> TaskResult {
    assert!(full_input.is_absolute());
    assert!(full_output.is_absolute());

    // Inject error directory renaming
    let full_output = handle_minify_directory_renaming(&full_input, &full_output);

    if let Some(dest_path_parent) = full_output.parent() {
        make_each_directory(dest_path_parent)?;
    }

    let full_output = full_output.to_str().unwrap_or_default();
    let full_input = full_input.to_str().unwrap_or_default();

    shell_log_piped(
        "minhtml",
        &["--minify-css", "--minify-js", "-o", full_output, full_input],
        &[]
    )
}


fn handle_minify_directory_renaming(full_input: &Path, full_output: &Path) -> PathBuf {
    assert!(full_input.is_absolute());
    assert!(full_output.is_absolute());

    match MINIFY_DIR_RENAMING.iter()
            // Find by trying stripping prefix of input dir and taking first successful
            .filter_map(
                |(path, renamed_path)| full_input
                    .strip_prefix(path)
                    .ok()
                    .map(|v| (v, renamed_path))
            )
            .next()
    {
        Some((relative_path, renamed_path)) => {
            let renamed_relative_output = renamed_path.join(relative_path);
            let cut_depth = relative_path_depth(&renamed_relative_output);
            if let Some(output_base) = cut_path_children(full_output, cut_depth) {
                output_base.join(renamed_relative_output)
            } else {
                full_output.to_owned()
            }
        },
        None => full_output.to_owned(),
    }
}
