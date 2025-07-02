use std::{
    cell::RefCell,
    env, fs,
    io::{Error, ErrorKind, Read, Result},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    thread,
};

use crate::paths;

////////////////////////////////////////////////////////////////////////////////
// Types
////////////////////////////////////////////////////////////////////////////////

pub type TaskResult = Result<()>;
pub const OK: TaskResult = Ok(());

/// Some extra methods for Path
pub trait DistributionPath {
    fn contains_any_extension(&self, extensions: &[&[u8]]) -> bool;

    fn relative_depth(&self) -> usize;

    fn cut_children(&self, depth: usize) -> Option<&Path>;

    fn for_each_file_recursively<'a, F>(&self, f: F) -> TaskResult
    where
        F: FnMut(&Path) -> () + 'a;

    fn copy_file_tree_filtered<'a, P>(
        &self,
        dest_dir: &Path,
        predicate: P,
    ) -> TaskResult
    where
        P: FnMut(&Path) -> bool + 'a;
}

impl DistributionPath for Path {
    ////////////////////////////////////////////////////////////////////////////////
    // Path processing
    ////////////////////////////////////////////////////////////////////////////////

    /// Check if `test_path` ends with any of specified extensions
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use xtask::core_dist::DistributionPath;
    ///
    /// assert!(!Path::new("").contains_any_extension(&[b""]));
    /// assert!(!Path::new("foo").contains_any_extension(&[b"foo"]));
    /// assert!(!Path::new("foo.bar").contains_any_extension(&[b"foo"]));
    /// assert!(Path::new("foo.bar").contains_any_extension(&[b"foo", b"bar"]));
    /// ```
    fn contains_any_extension(&self, extensions: &[&[u8]]) -> bool {
        match self.extension().map(|os_s| os_s.as_encoded_bytes()) {
            Some(ext) => extensions.contains(&ext),
            None => false,
        }
    }

    /// Measures relative depth.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use xtask::core_dist::DistributionPath;
    ///
    /// assert_eq!(Path::new("").relative_depth(), 0);
    /// assert_eq!(Path::new("foo").relative_depth(), 1);
    /// assert_eq!(Path::new("foo/bar").relative_depth(), 2);
    /// 
    /// #[cfg(target_os = "linux")]
    /// {
    ///     assert!(
    ///         std::panic::catch_unwind(|| Path::new("/").relative_depth())
    ///             .is_err()
    ///     );
    ///     assert!(
    ///         std::panic::catch_unwind(|| Path::new("/foo").relative_depth())
    ///             .is_err()
    ///     );
    /// }
    /// 
    /// #[cfg(target_os = "windows")]
    /// {
    ///     assert!(
    ///         std::panic::catch_unwind(|| Path::new("C:\\").relative_depth())
    ///             .is_err()
    ///     );
    ///     assert!(
    ///         std::panic::catch_unwind(|| Path::new("C:\\foo").relative_depth())
    ///             .is_err()
    ///     );
    /// }
    /// ```
    fn relative_depth(&self) -> usize {
        assert!(self.is_relative());

        let mut depth = 0;
        let mut path = self;

        while let Some(parent) = path.parent() {
            depth += 1;
            path = parent;
        }

        depth
    }

    /// Cuts with some depth.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::Path;
    /// use xtask::core_dist::DistributionPath;
    ///
    /// assert_eq!(Path::new("").cut_children(0), Some(Path::new("")));
    /// assert_eq!(Path::new("").cut_children(1), None);
    ///
    /// assert_eq!(Path::new("/").cut_children(0), Some(Path::new("/")));
    /// assert_eq!(Path::new("/").cut_children(1), None);
    ///
    /// assert_eq!(Path::new("foo").cut_children(0), Some(Path::new("foo")));
    /// assert_eq!(Path::new("foo").cut_children(1), Some(Path::new("")));
    /// assert_eq!(Path::new("foo").cut_children(2), None);
    /// ```
    fn cut_children(&self, depth: usize) -> Option<&Path> {
        if depth == 0 {
            return Some(self);
        }

        let mut path = self;
        for _ in 0..depth {
            path = path.parent()?;
        }

        Some(path)
    }

    ////////////////////////////////////////////////////////////////////////////////
    // File System Manipulation Primitives
    ////////////////////////////////////////////////////////////////////////////////

    /// DFS throughout `base_dir` based on lazy iterators
    ///
    /// Note: `f` receives path relative to `base_dir`
    fn for_each_file_recursively<'a, F>(&self, f: F) -> TaskResult
    where
        F: FnMut(&Path) -> () + 'a,
    {
        // Note: pop_files returns iterator of directories

        // Wrap for multiple mutable borrows inside pop_files
        let f = RefCell::new(f);
        let mut remaining_dirs_iterators = {
            let inner_dirs_iterator = pop_files(self, self, &f)?;
            vec![inner_dirs_iterator]
        };

        // DFS loop (pop stack top)
        while let Some(mut dirs) = remaining_dirs_iterators.pop() {
            match dirs.next() {
                Some(ref new_dir) => {
                    log::debug!(
                        "[xtask] processing recursively dir {}",
                        new_dir.display()
                    );
                    // bring parent level file-iterator back first
                    remaining_dirs_iterators.push(dirs);
                    // try add child level file-iterator
                    match pop_files(self, new_dir, &f) {
                        Ok(iterator) => remaining_dirs_iterators.push(iterator),
                        Err(e) => log::error!(
                            "[xtask] Failed to list {}: {}",
                            new_dir.display(),
                            e
                        ),
                    }
                }
                None => (),
            };
        }

        OK
    }

    /// Copy files from `from_dir` to `dest_dir` passing `predicate`.
    /// Does not stop on errors (just logs and skips).
    fn copy_file_tree_filtered<'a, P>(
        &self,
        dest_dir: &Path,
        mut predicate: P,
    ) -> TaskResult
    where
        P: FnMut(&Path) -> bool + 'a,
    {
        self.for_each_file_recursively(|relative_path| {
            // note: we only log errors and return nothing here
            if ! predicate(relative_path) {
                return;
            }
            let from_path = self.join(relative_path);
            let dest_path = dest_dir.join(relative_path);
            // make necessary directories
            if let Some(dest_path_parent) = dest_path.parent()
                && let Err(e) = make_each_directory(dest_path_parent)
            {
                log::error!(
                    "[xtask] Error creating directory {} while copying file tree: {}",
                    dest_path_parent.display(),
                    e
                );
                return;
            }
            // copy
            log::info!(
                "[xtask] Copying {} to {}",
                from_path.display(),
                dest_path.display()
            );
            match fs::copy(&from_path, &dest_path) {
                Err(e) => log::error!(
                    "[xtask] Error copying {} to {}: {}",
                    from_path.display(),
                    dest_path.display(),
                    e
                ),
                Ok(_) => {}
            }
        })
    }
}

/// Creates each directory in path (logged call to fs::create_dir_all).
pub fn make_each_directory(path: &Path) -> TaskResult {
    if path.exists() {
        return OK;
    }
    log::info!("[xtask] making each directory in {}", path.display());
    fs::create_dir_all(path)
}

////////////////////////////////////////////////////////////////////////////////
// Shell Manipulation Primitives
////////////////////////////////////////////////////////////////////////////////

/// Evaluate shell command and pipe output to logger (threaded).
/// Returns ErrorKind::Interrupted if ExitStatus was not successful.
pub fn shell_log_piped(
    cmd: &str,
    args: &[&str],
    env: &[(&str, &str)],
) -> TaskResult {
    let status = shell(cmd, args, env, false)?;

    if !status.success() {
        Err(Error::from(ErrorKind::Interrupted))
    } else {
        OK
    }
}

/// Evaluate shell command and pipe output directly to stdout(err).
/// Returns ExitStatus as success.
pub fn transparent_shell(
    cmd: &str,
    args: &[&str],
    env: &[(&str, &str)],
) -> Result<ExitStatus> {
    shell(cmd, args, env, true)
}

/// Evaluate cargo command and pipe output to logger (threaded).
/// Returns ErrorKind::Interrupted if cargo was not successful.
pub fn cargo(args: &[&str]) -> TaskResult {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    shell_log_piped(&cargo, args, &[])
}

////////////////////////////////////////////////////////////////////////////////
// Private
////////////////////////////////////////////////////////////////////////////////

/// Evaluate shell command and pipe output either to logger (threaded)
/// or directly to stdout(err).
/// Returns ExitStatus as success.
fn shell(
    cmd: &str,
    args: &[&str],
    env: &[(&str, &str)],
    transparent: bool,
) -> Result<ExitStatus> {
    let mut shell_command = Command::new(cmd);

    shell_command
        .current_dir(paths::PROJECT_ROOT.clone())
        .args(args);

    env.iter().for_each(|(k, v)| {
        shell_command.env(k, v);
    });

    if log::log_enabled!(log::Level::Info) {
        let env_table = env
            .iter()
            .map(|(k, v)| format!("{}=\"{}\" ", k, v))
            .fold(String::new(), |mut e, l| {
                e.push_str(l.as_str());
                e
            });
        log::info!("[shell] $ {} {} {:?}", env_table, cmd, args);
    }

    if transparent {
        // Spawn with transparent io
        return shell_command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();
    }

    // Spawn with piped io
    let mut shell_process = shell_command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Pipe output into log (with stderr in separate thread)
    let mut stdout = shell_process
        .stdout
        .take()
        .expect("failed to acquire piped stdout");
    let mut stderr = shell_process
        .stderr
        .take()
        .expect("failed to acquire piped stderr");

    thread::scope(|s| {
        let h = s.spawn(|| log_pipe(&cmd, &mut stderr, false));
        if let Err(e) = log_pipe(&cmd, &mut stdout, true) {
            log::error!(
                "[shell] error occurred while piping shell stdout: {}",
                e
            );
        }
        match h.join() {
            Ok(Err(e)) => log::error!(
                "[shell] error occurred while piping shell stderr: {}",
                e
            ),
            Err(_) => {
                log::error!("[shell] panic occurred while piping shell stderr")
            }
            Ok(Ok(_)) => (),
        }
    });

    // Collect exit status and exit
    let status = shell_process.wait()?;
    log::debug!("[shell] {} process exited with {}", cmd, status);

    Ok(status)
}

/// Log shell stdout/stderr.
fn log_pipe(
    proc_name: &str,
    pipe: &mut impl Read,
    is_stdout: bool,
) -> TaskResult {
    const COMMON_MAX_LINE_LENGTH: usize = 120;
    let mut line_buffer = [0; COMMON_MAX_LINE_LENGTH];
    let (mut line_filled_slice, mut line_remaining_slice) =
        line_buffer.split_at_mut(0);
    let prefix = if is_stdout { "stdout" } else { "stderr" };
    let flush_to_log = |line: &[u8]| {
        log::info!(
            "[{} {}] {}",
            proc_name,
            prefix,
            String::from_utf8_lossy(line)
        )
    };

    loop {
        let slice_index = match pipe.read_exact(&mut line_remaining_slice[..1])
        {
            // If line ending in buffer (flush)
            Ok(_) if line_remaining_slice[0] == b'\n' => {
                flush_to_log(line_filled_slice);
                0
            }
            // If line buffer filled up (flush)
            Ok(_) if line_remaining_slice.len() == 1 => {
                flush_to_log(&line_buffer);
                0
            }
            // else accumulate
            Ok(_) => line_filled_slice.len() + 1,
            // if error (flush n return)
            Err(e) => {
                if !line_filled_slice.is_empty() {
                    flush_to_log(line_filled_slice);
                }
                return if e.kind() == ErrorKind::UnexpectedEof {
                    OK
                } else {
                    Err(e)
                };
            }
        };
        (line_filled_slice, line_remaining_slice) =
            line_buffer.split_at_mut(slice_index);
    }
}

/// Pop files in `full_path` into `f` and pass dirs through (lazy).
/// `f` receives path relative to `base_path`.
/// `full_path` should be child of `base_path`.
fn pop_files<'a, 'b, F>(
    base_path: &'a Path,
    full_path: &'b Path,
    f: &'a RefCell<F>,
) -> Result<impl Iterator<Item = PathBuf> + 'a>
where
    F: FnMut(&Path) -> () + 'a,
{
    assert!(
        full_path.strip_prefix(base_path).is_ok(),
        "full_path should be child of base_path"
    );

    let listing = fs::read_dir(full_path)?;

    let lazy_map = listing.filter_map(move |listing_result| {
        match listing_result {
            Ok(entry) => match entry.file_type() {
                // if file -> callback and skip
                Ok(fd) if fd.is_file() => {
                    // calc relative path
                    let relative_path = entry
                        .path()
                        .strip_prefix(base_path)
                        .expect("listing entry should be taken from child of `base_dir` (or itself)")
                        .to_owned();
                    // Panic Safety: the only place where we do the borrow
                    f.borrow_mut()(&relative_path);
                    // filter out
                    None
                }
                // if dir -> pass further as path
                Ok(fd) if fd.is_dir() => Some(entry.path()),
                // if something else -> skip
                Ok(_) => None,
                // if error -> log and skip
                Err(e) => {
                    log::error!(
                        "[xtask] Error while processing {}: {}",
                        entry.file_name().display(),
                        e
                    );
                    None
                }
            },
            // if error -> log and skip
            Err(e) => {
                log::error!(
                    "[xtask] Error while processing files recursively: {}",
                    e
                );
                None
            }
        }
    });

    Ok(lazy_map)
}
