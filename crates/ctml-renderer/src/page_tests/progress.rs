use std::{
    io::{self, IsTerminal, Write},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

pub(super) fn log_step(message: &str) {
    eprintln!(
        "{}",
        colorize(&format!("CSS parity  {message}"), "\x1b[36m")
    );
}

pub(super) fn log_success(message: &str) {
    eprintln!(
        "{}",
        colorize(&format!("CSS parity  {message}"), "\x1b[32m")
    );
}

pub(super) fn log_notice(message: &str) {
    eprintln!(
        "{}",
        colorize(&format!("CSS parity  {message}"), "\x1b[33m")
    );
}

pub(super) fn colorize(message: &str, color: &str) -> String {
    if std::env::var_os("NO_COLOR").is_some() {
        message.to_owned()
    } else {
        format!("{color}{message}\x1b[0m")
    }
}

pub(super) struct ProgressBar {
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}

impl ProgressBar {
    pub(super) fn start(message: &str) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let message = message.to_owned();
        let terminal = io::stderr().is_terminal();
        log_step(&format!("{message}..."));
        let worker = thread::spawn(move || {
            let started = Instant::now();
            let mut frame = 0;
            while !worker_stop.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(120));
                if worker_stop.load(Ordering::Relaxed) {
                    break;
                }
                frame += 1;
                let elapsed = started.elapsed().as_secs();
                if terminal {
                    let position = frame % 12;
                    let bar = (0..12)
                        .map(|index| if index == position { '█' } else { '─' })
                        .collect::<String>();
                    eprint!(
                        "\r{}",
                        colorize(
                            &format!("CSS parity  {message} [{bar}] {elapsed}s"),
                            "\x1b[36m"
                        )
                    );
                    let _ = io::stderr().flush();
                } else if frame % 42 == 0 {
                    eprintln!("CSS parity  {message}... {elapsed}s elapsed");
                }
            }
        });
        Self {
            stop,
            worker: Some(worker),
        }
    }

    pub(super) fn finish(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        if io::stderr().is_terminal() {
            eprint!("\r\x1b[2K");
        }
        log_success("Progress complete");
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        if io::stderr().is_terminal() {
            eprint!("\r\x1b[2K");
        }
    }
}
