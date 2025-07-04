use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crate::core_dist::{
    DistributionPath, OK, TaskResult, cargo, make_each_directory,
    shell_log_piped, transparent_shell,
};
use crate::paths;

////////////////////////////////////////////////////////////////////////////////
// Shell Commands
////////////////////////////////////////////////////////////////////////////////

const HTTP_SERVER: &str = "simple-http-server";
const WASM_PACK: &str = "wasm-pack";
const MINHTML: &str = "minhtml";

////////////////////////////////////////////////////////////////////////////////
// Web Distribution Paths
////////////////////////////////////////////////////////////////////////////////

const FRONT_PAGE_DIR: &str = "front-page";
const ERROR_PAGE_SUBDIR: &str = "error_pages";
const HIDDEN_ERROR_PAGE_DIR: &str = ".error_pages";

const MINIFY_EXTENSIONS: &[&[u8]] = &[b"html", b"css", b"js"];

static MINIFY_DIR_RENAMING: LazyLock<[(PathBuf, PathBuf); 1]> =
    LazyLock::new(|| {
        [(
            paths::PROJECT_ROOT
                .join(FRONT_PAGE_DIR)
                .join(ERROR_PAGE_SUBDIR),
            PathBuf::from(HIDDEN_ERROR_PAGE_DIR),
        )]
    });

////////////////////////////////////////////////////////////////////////////////
// CLI Tasks
////////////////////////////////////////////////////////////////////////////////

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
    OK
}

pub fn build_web_distribution_dev() -> TaskResult {
    let web_dist_path =
        paths::BUILD_PATH.join(paths::WEB_DIST_DEV_SUBDIRECTORY);
    let wasm_pkg_path =
        paths::BUILD_PATH.join(paths::WASM_PKG_DEV_SUBDIRECTORY);
    build_web_distribution_by_path(&web_dist_path, &wasm_pkg_path, false)
}

pub fn serve_web_distribution_dev() -> TaskResult {
    prepare_serve_web_distribution_dev()?;
    let web_dist_path =
        paths::BUILD_PATH.join(paths::WEB_DIST_DEV_SUBDIRECTORY);
    serve_web_distribution_by_path(&web_dist_path)
}

pub fn prepare_serve_web_distribution_dev() -> TaskResult {
    build_web_distribution_dev()?;
    cargo(&["install", HTTP_SERVER])?;
    OK
}

////////////////////////////////////////////////////////////////////////////////
// Private
////////////////////////////////////////////////////////////////////////////////

fn build_web_distribution_by_path(
    web_dist_path: &Path,
    wasm_pkg_path: &Path,
    release: bool,
) -> TaskResult {
    cargo(&["install", WASM_PACK])?;

    make_each_directory(web_dist_path)?;

    let wasm_pkg_path_arg = wasm_pkg_path.to_string_lossy();
    let wasm_pack_args: &[&str] = if release {
        &[
            "--verbose",
            "build",
            "shards-browser",
            "--target",
            "web",
            "--out-dir",
            &wasm_pkg_path_arg,
        ]
    } else {
        &[
            "--verbose",
            "build",
            "shards-browser",
            "--dev",
            "--target",
            "web",
            "--out-dir",
            &wasm_pkg_path_arg,
        ]
    };

    shell_log_piped(
        "wasm-pack",
        wasm_pack_args,
        &[("RUSTFLAGS", "-Ctarget-cpu=mvp")],
    )?;

    let front_page_path = paths::PROJECT_ROOT.join(FRONT_PAGE_DIR);

    // copy all wasm modules and js-bindings
    log::debug!(
        "[xtask] Copying js and wasm from {} to {}",
        wasm_pkg_path.display(),
        web_dist_path.display()
    );
    wasm_pkg_path.copy_file_tree_filtered(web_dist_path, |path| {
        path.contains_any_extension(&[b"js", b"wasm"])
    })?;

    if release {
        cargo(&["install", MINHTML])?;

        // minify front-page html, css and js files
        thread::scope(|s| minify_swarm(s, &front_page_path, web_dist_path))?;

        // copy rest
        log::debug!(
            "[xtask] Copying resources from {} to {}",
            front_page_path.display(),
            web_dist_path.display()
        );
        front_page_path.copy_file_tree_filtered(web_dist_path, |path| {
            !path.contains_any_extension(&MINIFY_EXTENSIONS)
        })?;
    } else {
        // copy whole front-page
        log::debug!(
            "[xtask] Copying everything from {} to {}",
            front_page_path.display(),
            web_dist_path.display()
        );
        front_page_path.copy_file_tree(web_dist_path)?;
    }

    log::info!("[xtask] Done! Check: {}", web_dist_path.display());
    OK
}

fn serve_web_distribution_by_path(web_dist_path: &Path) -> TaskResult {
    let web_dist_path_str = web_dist_path.to_string_lossy();

    transparent_shell(
        HTTP_SERVER,
        &[
            "--nocache",
            "-i",
            "-p",
            "8080",
            "--ip",
            "127.0.0.1",
            &web_dist_path_str,
        ],
        &[],
    )?;

    OK
}

fn minify_swarm<'a>(
    s: &'a thread::Scope<'a, '_>,
    input: &'a Path,
    output: &'a Path,
) -> TaskResult {
    let available_parallelism = thread::available_parallelism()?.get();
    assert!(available_parallelism > 0, "0 parallelism?!");

    let log_minify_error = |err| {
        log::error!(
            "[xtask] failed to call minify {} > {}: {}",
            input.display(),
            output.display(),
            err
        );
    };

    // Specify sender type to help infer rest
    type MinifyTask = (PathBuf, PathBuf); // minify(&input, &output)
    let make_channel = || mpsc::sync_channel::<MinifyTask>(0);

    // Spawn thread pool with pipe
    let pipes: Vec<_> = (0..available_parallelism)
        .map(|_| {
            let (sender, receiver) = make_channel();

            let minify_thread_func = move || {
                receiver
                    .iter()
                    .map(|(input, output)| minify(&input, &output))
                    .filter_map(|minify_result| minify_result.err())
                    .for_each(log_minify_error)
            };

            s.spawn(minify_thread_func);

            sender
        })
        .collect();

    input.for_each_file_recursively(|relative_path| {
        if !relative_path.contains_any_extension(MINIFY_EXTENSIONS) {
            return;
        }

        let mut minify_args =
            (input.join(relative_path), output.join(relative_path));

        'send_loop: loop {
            for pipe in pipes.iter() {
                minify_args = if let Err(e) = pipe.try_send(minify_args) {
                    match e {
                        mpsc::TrySendError::Full(args) => args,
                        mpsc::TrySendError::Disconnected(args) => args,
                    }
                } else {
                    break 'send_loop;
                }
            }
            thread::sleep(Duration::from_millis(1));
        }
    })
}

fn minify(full_input: &Path, full_output: &Path) -> TaskResult {
    assert!(full_input.is_absolute());
    assert!(full_output.is_absolute());

    // Inject error directory renaming
    let full_output =
        handle_minify_directory_renaming(&full_input, &full_output);

    if let Some(dest_path_parent) = full_output.parent() {
        make_each_directory(dest_path_parent)?;
    }

    let full_output = full_output.to_str().unwrap_or_default();
    let full_input = full_input.to_str().unwrap_or_default();

    shell_log_piped(
        "minhtml",
        &["--minify-css", "--minify-js", "-o", full_output, full_input],
        &[],
    )
}

fn handle_minify_directory_renaming(
    full_input: &Path,
    full_output: &Path,
) -> PathBuf {
    assert!(full_input.is_absolute());
    assert!(full_output.is_absolute());

    // Find by trying stripping prefix of input dir and taking all successful.
    // First candidate would be enough.
    let mut candidates_it = MINIFY_DIR_RENAMING
        .iter()
        .filter_map(|(path, renamed_path)| {
            // try strip path and take renamed_path with
            full_input
                .strip_prefix(path)
                .ok()
                .map(|relative_path| (relative_path, renamed_path))
        });

    if let Some((relative_path, renamed_path)) = candidates_it.next()
        && let renamed_relative_output = renamed_path.join(relative_path)
        && let cut_depth = renamed_relative_output.relative_depth()
        && let Some(output_base) = full_output.cut_children(cut_depth)
    {
        output_base.join(renamed_relative_output)
    } else {
        full_output.to_owned()
    }
}
