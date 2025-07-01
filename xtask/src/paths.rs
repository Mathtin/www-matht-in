pub const BUILD_DIRECTORY: &str = "target";

pub const WEB_DIST_SUBDIRECTORY: &str = "web-dist";

pub const WEB_DIST_DEV_SUBDIRECTORY: &str = "web-dist-dev";

pub const WASM_PKG_SUBDIRECTORY: &str = "shards-browser-pkg";
pub const WASM_PKG_DEV_SUBDIRECTORY: &str = "shards-browser-dev-pkg";

// Calculated

use std::path::{Path, PathBuf};
use std::sync::LazyLock;

pub static PROJECT_ROOT: LazyLock<&Path> = LazyLock::new(|| {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
});

pub static BUILD_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| PROJECT_ROOT.join(BUILD_DIRECTORY));
