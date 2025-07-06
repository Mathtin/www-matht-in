#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use std::sync::OnceLock;

static START_SUCCESS: OnceLock<bool> = OnceLock::new();

fn first_startup() -> bool {
    log::init_log();

    log::debug!("Shards browser started!");

    return true;
}

fn start_impl() {
    let res = START_SUCCESS.get_or_init(first_startup);
    log::debug!("Start result: {}", res);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start() {
    start_impl()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn start() {
    start_impl()
}

#[cfg(test)]
mod tests {

    use super::*;
    use web_time::{Duration, Instant};

    const MAX_START_WARN_SECS: u64 = 4;
    const MAX_START_SECS: u64 = 16;
    const MAX_SECOND_START_MS: u64 = 256;

    #[test]
    fn it_starts_quick_enough() {
        timed_start_one_pass();

        const MAX_SECOND_START_DURATION: Duration =
            Duration::from_millis(MAX_SECOND_START_MS);

        let second_pass_duration = timed_start_one_pass();

        assert!(
            second_pass_duration < MAX_SECOND_START_DURATION,
            "Slow second start (should be effectively empty)!"
        );
    }

    fn timed_start_one_pass() -> Duration {
        let h = std::thread::spawn(start);

        const CHECK_SLEEP_DURATION: Duration =
            Duration::from_millis(MAX_SECOND_START_MS / 2);
        const MAX_DURATION_ALERT: Duration =
            Duration::from_secs(MAX_START_WARN_SECS);
        const MAX_DURATION: Duration = Duration::from_secs(MAX_START_SECS);

        assert!(MAX_DURATION_ALERT < MAX_DURATION);

        let mut real_time_measure = Instant::now();
        let mut passed: Duration = Default::default();

        while passed < MAX_DURATION_ALERT {
            if h.is_finished() {
                break;
            }
            std::thread::sleep(CHECK_SLEEP_DURATION);
            let new_real_time_measure = Instant::now();
            passed += new_real_time_measure - real_time_measure;
            real_time_measure = new_real_time_measure;
        }

        if passed >= MAX_DURATION_ALERT {
            log::warn!("[timed_start_one_pass] Slow start!");
        }

        while passed < MAX_DURATION {
            if h.is_finished() {
                break;
            }
            std::thread::sleep(CHECK_SLEEP_DURATION);
            let new_real_time_measure = Instant::now();
            passed += new_real_time_measure - real_time_measure;
            real_time_measure = new_real_time_measure;
        }

        assert!(passed < MAX_DURATION, "Starting too long!");

        return passed;
    }
}
