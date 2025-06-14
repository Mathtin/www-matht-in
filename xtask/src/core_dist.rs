use std::{
    cell::RefCell,
    env, fs,
    io::{Error, ErrorKind, Read},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::LazyLock,
    thread,
};

use crate::paths;


pub type TaskResult = Result<(), Error>;

pub static PROJECT_ROOT: LazyLock<&Path> = LazyLock::new(|| {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
});

pub static BUILD_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    PROJECT_ROOT.join(paths::BUILD_DIRECTORY)
});


pub fn make_each_directory(path: &Path) -> TaskResult {
    log::debug!("[xtask] making each directory in {}", path.display());

    if ! path.exists() {
        fs::create_dir_all(path)?;
        log::info!("[xtask] directory created");
    } else if ! path.is_dir() {
        return Err(Error::from(ErrorKind::NotADirectory));
    } else {
        log::debug!("[xtask] directory already exists");
    }
    
    Ok(())
}


fn log_pipe(proc_name: &str, pipe: &mut impl Read, is_stdout: bool) -> Result<(), Error> {
    const COMMON_MAX_LINE_LENGTH: usize = 120;
    let mut line_buffer = [0; COMMON_MAX_LINE_LENGTH];
    let mut line_filled_part: &mut [u8] = &mut [];
    let mut line_remaining_part: &mut [u8] = &mut line_buffer;
    let prefix = if is_stdout { "stdout" } else { "stderr" };
    loop {
        let slice_index = match pipe.read_exact(&mut line_remaining_part[..1]) {
            // If line ending in buffer (flush)
            Ok(_) if line_remaining_part[0] == b'\n' => {
                log::info!("[{} {}] {}", proc_name, prefix, String::from_utf8_lossy(&line_filled_part));
                0
            },
            // If line buffer filled up (flush)
            Ok(_) if line_remaining_part.len() == 1 => {
                log::info!("[{} {}] {}", proc_name, prefix, String::from_utf8_lossy(&line_buffer));
                0
            },
            // else accumulate
            Ok(_) => line_filled_part.len() + 1,
            // if error (flush n return)
            Err(e) => {
                if ! line_filled_part.is_empty() {
                    log::info!("[{} {}] {}", proc_name, prefix, String::from_utf8_lossy(&line_filled_part));
                }
                return if e.kind() == ErrorKind::UnexpectedEof { Ok(()) } else { Err(e) };
            },
        };
        (line_filled_part, line_remaining_part) = line_buffer.split_at_mut(slice_index);
    }
}


pub fn shell_log_piped(cmd: &str, args: &[&str], env: &[(&str, &str)]) -> Result<ExitStatus, Error> {
    shell(cmd, args, env, true)
}


pub fn shell(cmd: &str, args: &[&str], env: &[(&str, &str)], log_piped: bool) -> Result<ExitStatus, Error> {
    let mut shell_command = Command::new(cmd);

    shell_command
        .current_dir(PROJECT_ROOT.clone())
        .args(args);

    env.iter().for_each(|(k, v)| {shell_command.env(k, v);});

    log::debug!("[shell] $ {} {:?}", cmd, args);

    if ! log_piped {
        return shell_command
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();
    }

    let mut shell_process = shell_command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdout = shell_process.stdout.take().expect("failed to acquire piped stdout");
    let mut stderr = shell_process.stderr.take().expect("failed to acquire piped stderr");

    thread::scope(|s| {
        let h = s.spawn(|| log_pipe(&cmd, &mut stderr, false));
        if let Err(e) = log_pipe(&cmd, &mut stdout, true) {
            log::error!("[shell] error occurred while piping shell stdout: {}", e);
        }
        match h.join() {
            Ok(Err(e)) => log::error!("[shell] error occurred while piping shell stderr: {}", e),
            Err(_) => log::error!("[shell] panic occurred while piping shell stderr"),
            Ok(Ok(_)) => (),
        }
    });

    let status = shell_process.wait()?;
    log::debug!("[shell] {} process exited with {}", cmd, status);
    
    Ok(status)
}


pub fn cargo(args: &[&str]) -> Result<(), Error> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = shell(&cargo, args, &[], true)?;

    if ! status.success() {
        Err(Error::from(ErrorKind::Interrupted))
    } else {
        Ok(())
    }
}


/// Pop files into `f` and pass dirs through
/// 
/// Note: `f` receives path relative to `base_dir`
fn pop_files<'a, F>(
    base_dir: &'a Path,
    listing: fs::ReadDir,
    f: &'a RefCell<F>,
) -> impl Iterator<Item = PathBuf> + 'a
where
    F: FnMut(&Path) -> () + 'a,
{
    return listing.filter_map(move |entry| {
        match entry {
            // entry available
            Ok(entry) => match entry.file_type() {
                // is file (callback and skip)
                Ok(fd) if !fd.is_dir() => {
                    // calc relative path
                    let full_path = match entry.path().canonicalize() {
                        Ok(full_path) => full_path,
                        Err(e) => {
                            log::error!("[xtask] Error canonicalizing {}: {}", entry.path().display(), e);
                            return None;
                        },
                    };
                    let relative_path = full_path.strip_prefix(base_dir)
                        .expect("failed to remove path prefix") // can this even happen?
                        .to_owned();
                    // Only place where we do the borrow
                    f.borrow_mut()(&relative_path);
                    None
                },
                // is dir (pass further as path)
                Ok(_) => Some(entry.path()),
                // skip if error
                Err(e) => {
                    log::error!("[xtask] Error while processing {}: {}", entry.file_name().display(), e);
                    None
                },
            },
            // skip if error
            Err(e) => {
                log::error!("[xtask] Error while processing files recursively: {}", e);
                None
            },
        }
    });
}


/// DFS throughout `base_dir` based on lazy iterators
/// 
/// Note: `f` receives path relative to `base_dir`
pub fn for_each_file_recursively<'a, F>(base_dir: &Path, f: F) -> TaskResult
where
    F: FnMut(&Path) -> () + 'a,
{
    // Note: pop_files returns iterator of directories

    // Wrap for multiple mutable borrows inside pop_files
    let f = RefCell::new(f);
    let base_listing = fs::read_dir(base_dir)?;
    let mut remaining_dirs_iterators = {
        let inner_dirs_iterator = pop_files(base_dir, base_listing, &f);
        vec![inner_dirs_iterator]
    };

    // DFS loop (pop stack top)
    while let Some(mut dirs) = remaining_dirs_iterators.pop() {
        match dirs.next() {
            Some(dir) => {
                log::debug!("[xtask] processing recursively dir {}", dir.display());
                // bring parent level file-iterator back first
                remaining_dirs_iterators.push(dirs);
                match fs::read_dir(&dir) {
                    Ok(listing) => {
                        let inner_dirs = pop_files(base_dir, listing, &f);
                        // push child level file-iterator after parent level one (this makes it DFS)
                        remaining_dirs_iterators.push(inner_dirs);
                    },
                    Err(e) => log::error!("[xtask] Failed to list {}: {}", dir.display(), e),
                }
            },
            None => (),
        };
    }

    Ok(())
}


pub fn copy_file_tree_filtered(
    from_dir: &Path,
    dest_dir: &Path,
    extensions: &[&[u8]],
    white_list: bool,
) -> TaskResult {
    log::info!(
        "[xtask] Copying all{} {:?} from {} to {}", 
        (if white_list { "" } else { " besides" }),
        extensions
            .iter()
            .map(|e| String::from_utf8_lossy(e))
            .collect::<Vec<_>>(),
        from_dir.display(), 
        dest_dir.display()
    );

    let full_from_dir = from_dir.canonicalize()?;

    for_each_file_recursively(&full_from_dir, |relative_path| {
        // note: we only log errors and return nothing here
        // filter extensions
        match relative_path
            .extension()
            .map(|os_s| os_s.as_encoded_bytes())
        {
            None => return,
            Some(ext) if white_list ^ extensions.contains(&ext) => return,
            Some(_) => {} // extension check passed
        }
        // make necessary directories
        let from_path = from_dir.join(relative_path);
        let dest_path = dest_dir.join(relative_path);
        if let Some(dest_path_parent) = dest_path.parent() {
            if let Err(e) = make_each_directory(dest_path_parent) {
                log::error!("[xtask] Error creating directory {} while copying file tree: {}", dest_path_parent.display(), e);
                return;
            }
        }
        // copy
        log::debug!("[xtask] Copying {} to {}", from_path.display(), dest_path.display());
        match fs::copy(&from_path, &dest_path) {
            Err(e) => log::error!("[xtask] Error copying {} to {}: {}", from_path.display(), dest_path.display(), e),
            Ok(_) => {},
        }
    })
}
