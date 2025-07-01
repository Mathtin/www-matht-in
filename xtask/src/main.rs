use std::process;

fn main() {
    log::init_log();
    if let Err(e) = xtask::run() {
        log::error!("{}", e);
        process::exit(-1);
    }
}
