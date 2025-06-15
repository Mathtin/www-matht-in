use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::thread;
use std::time::Duration;

use crate::core_dist::{
    BUILD_PATH, PROJECT_ROOT, TaskResult, cargo, copy_file_tree_filtered,
    for_each_file_recursively, make_each_directory, shell, shell_log_piped,
};
use crate::paths;


const HTTP_SERVER: &str = "simple-http-server";
const WASM_PACK: &str = "wasm-pack";
const MINHTML: &str = "minhtml";
const FRONT_PAGE_DIR: &str = "front-page";
const ERROR_PAGE_SUBDIR: &str = "error_pages";
const HIDDEN_ERROR_PAGE_DIR: &str = ".error_pages";

static ERROR_PAGE_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| PROJECT_ROOT.join(FRONT_PAGE_DIR).join(ERROR_PAGE_SUBDIR));

const MINIFY_EXTENSIONS: &[&[u8]] = &[b"html", b"css", b"js"];


pub fn build_web_distribution() -> TaskResult {
    let web_dist_path = BUILD_PATH.join(paths::WEB_DIST_SUBDIRECTORY);
    let wasm_pkg_path = BUILD_PATH.join(paths::WASM_PKG_SUBDIRECTORY);
    build_web_distribution_by_path(&web_dist_path, &wasm_pkg_path, true)
}


pub fn serve_web_distribution() -> TaskResult {
    build_web_distribution()?;
    let web_dist_path = BUILD_PATH.join(paths::WEB_DIST_SUBDIRECTORY);
    serve_web_distribution_by_path(&web_dist_path)
}


pub fn build_web_distribution_dev() -> TaskResult {
    let web_dist_path = BUILD_PATH.join(paths::WEB_DIST_DEV_SUBDIRECTORY);
    let wasm_pkg_path = BUILD_PATH.join(paths::WASM_PKG_DEV_SUBDIRECTORY);
    build_web_distribution_by_path(&web_dist_path, &wasm_pkg_path, false)
}


pub fn serve_web_distribution_dev() -> TaskResult {
    build_web_distribution_dev()?;
    let web_dist_path = BUILD_PATH.join(paths::WEB_DIST_DEV_SUBDIRECTORY);
    serve_web_distribution_by_path(&web_dist_path)
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

    if ! shell_log_piped(
        "wasm-pack", 
        wasm_pack_args, 
        &[("RUSTFLAGS", "-Ctarget-cpu=mvp")]
    )?.success() {
        return Err(Error::from(ErrorKind::Interrupted));
    };

    let front_page_path = PROJECT_ROOT.join(FRONT_PAGE_DIR);

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
    cargo(&["install", HTTP_SERVER])?;

    let web_dist_path_str = web_dist_path.to_string_lossy();

    shell(
        HTTP_SERVER, 
        &["--nocache", "-i", "-p", "8080", "--ip", "127.0.0.1", &web_dist_path_str], 
        &[],
        false
    )?;

    Ok(())
}


fn minify_swarm(input: &Path, output: &Path) -> TaskResult {
    let available_parallelism = thread::available_parallelism()?.get();
    assert!(available_parallelism > 0, "0 parallelism?!");
    thread::scope(|s| {
        let mut handles = vec![];
        for_each_file_recursively(input, |relative_path| {
            // check extension
            match relative_path
                .extension()
                .map(|os_s| os_s.as_encoded_bytes())
            {
                None => return,
                Some(ext) if !MINIFY_EXTENSIONS.contains(&ext) => return,
                Some(_) => {} // extension check passed
            }
            // Note: available_parallelism is always > 0
            while handles.len() >= available_parallelism {
                let extracted = handles
                    .extract_if(.., |h: &mut thread::ScopedJoinHandle<TaskResult>| {
                        h.is_finished()
                    })
                    .count();
                if extracted == 0 { 
                    thread::sleep(Duration::from_millis(1));
                }
            }
            // minify in thread
            let from_path = input.join(relative_path);
            let dest_path = output.join(relative_path);
            let h = s.spawn(move || minify(&from_path, &dest_path));
            handles.push(h);
        })
    })
}


fn minify(input: &Path, output: &Path) -> TaskResult {
    let names_equal = |path: &Path, name: &str| {
        path.file_name()
            .filter(|n| n.as_encoded_bytes() == name.as_bytes())
            .is_some()
    };

    // Inject error directory renaming
    let injected_path;
    let output = match input.parent() {
        // if input is PROJECT_ROOT/FRONT_PAGE_DIR/ERROR_PAGE_SUBDIR/file
        Some(p) if p == *ERROR_PAGE_PATH => match output.parent() {
            // if dest is .../ERROR_PAGE_SUBDIR/file
            Some(output_parent) if names_equal(output_parent, ERROR_PAGE_SUBDIR) => {
                match output_parent.parent() {
                Some(output_parent_parent) => {
                        injected_path = output_parent_parent.join(HIDDEN_ERROR_PAGE_DIR).join(
                            output
                                .file_name()
                                .expect("output path should point to file"),
                        );
                    &injected_path
                    }
                None => output,
                }
            }
            Some(_) => output,
            None => output,
        },
        Some(_) => output,
        None => output,
    };

    if let Some(dest_path_parent) = output.parent() {
        make_each_directory(dest_path_parent)?;
    }

    let output = output.to_str().unwrap_or_default();
    let input = input.to_str().unwrap_or_default();

    if ! shell_log_piped(
        "minhtml",
        &["--minify-css", "--minify-js", "-o", output, input],
        &[]
    )?.success() {
        return Err(Error::from(ErrorKind::Interrupted));
    };

    Ok(())
}
