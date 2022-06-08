use std::panic::{RefUnwindSafe, UnwindSafe};
use std::thread;
use std::time::Duration;

/// Runs a function forever.
/// Catches errors but not panics
pub fn error_boundary<F: Fn() -> anyhow::Result<!>>(function: F) -> ! {
    loop {
        let result = (function)();

        if let Err(error) = result {
            let thread = thread::current();
            let name = thread.name().unwrap_or("unnamed");
            eprintln!("{} thread encountered an error, message: {:?}", name, error);
        }

        thread::sleep(Duration::from_millis(500))
    }
}
