use html_to_markdown_rs::{ConversionError, Result};
use std::path::PathBuf;

#[cfg(all(not(target_os = "windows"), feature = "profiling"))]
mod enabled {
    use super::{ConversionError, PathBuf, Result};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Mutex, OnceLock};

    const ENV_OUTPUT: &str = "HTML_TO_MARKDOWN_PROFILE_OUTPUT";
    const ENV_FREQUENCY: &str = "HTML_TO_MARKDOWN_PROFILE_FREQUENCY";
    const ENV_ONCE: &str = "HTML_TO_MARKDOWN_PROFILE_ONCE";
    const ENV_REPEAT: &str = "HTML_TO_MARKDOWN_PROFILE_REPEAT";

    static PROFILED_ONCE: AtomicBool = AtomicBool::new(false);
    static PROFILE_ACTIVE: AtomicBool = AtomicBool::new(false);

    struct EnvProfileConfig {
        output: Option<PathBuf>,
        profile_once: bool,
        repeat: usize,
        frequency: i32,
    }

    fn env_profile_config() -> &'static EnvProfileConfig {
        static ENV_CONFIG: OnceLock<EnvProfileConfig> = OnceLock::new();
        ENV_CONFIG.get_or_init(|| {
            let output = match std::env::var(ENV_OUTPUT) {
                Ok(value) if !value.trim().is_empty() => Some(PathBuf::from(value)),
                _ => None,
            };

            let profile_once = match std::env::var(ENV_ONCE) {
                Ok(value) => !matches!(value.as_str(), "0" | "false" | "no"),
                Err(_) => true,
            };

            let repeat = std::env::var(ENV_REPEAT)
                .ok()
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(1)
                .max(1);

            let frequency = std::env::var(ENV_FREQUENCY)
                .ok()
                .and_then(|value| value.parse::<i32>().ok())
                .unwrap_or(1000);

            EnvProfileConfig {
                output,
                profile_once,
                repeat,
                frequency,
            }
        })
    }

    struct ProfileState {
        guard: Option<pprof::ProfilerGuard<'static>>,
        output: Option<PathBuf>,
    }

    fn state() -> &'static Mutex<ProfileState> {
        static STATE: OnceLock<Mutex<ProfileState>> = OnceLock::new();
        STATE.get_or_init(|| {
            Mutex::new(ProfileState {
                guard: None,
                output: None,
            })
        })
    }

    pub fn start(output_path: PathBuf, frequency: i32) -> Result<()> {
        let mut state = state()
            .lock()
            .map_err(|_| ConversionError::Other("profiling state lock poisoned".to_string()))?;

        if state.guard.is_some() {
            return Err(ConversionError::Other("profiling already active".to_string()));
        }

        let guard = pprof::ProfilerGuardBuilder::default()
            .frequency(frequency)
            .blocklist(&["libc", "libpthread", "libgcc", "libm"])
            .build()
            .map_err(|err| ConversionError::Other(format!("Profiling init failed: {err}")))?;

        state.guard = Some(guard);
        state.output = Some(output_path);
        PROFILE_ACTIVE.store(true, Ordering::Release);
        Ok(())
    }

    pub fn stop() -> Result<()> {
        let (guard, output) = {
            let mut state = state()
                .lock()
                .map_err(|_| ConversionError::Other("profiling state lock poisoned".to_string()))?;
            let guard = state.guard.take();
            let output = state.output.take();
            (guard, output)
        };
        PROFILE_ACTIVE.store(false, Ordering::Release);

        let Some(guard) = guard else {
            return Err(ConversionError::Other("profiling not active".to_string()));
        };
        let Some(output_path) = output else {
            return Err(ConversionError::Other("profiling output path missing".to_string()));
        };

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(ConversionError::IoError)?;
        }

        let report = guard
            .report()
            .build()
            .map_err(|err| ConversionError::Other(format!("Profiling report failed: {err}")))?;

        let file = std::fs::File::create(&output_path).map_err(ConversionError::IoError)?;
        report
            .flamegraph(file)
            .map_err(|err| ConversionError::Other(format!("Flamegraph write failed: {err}")))?;
        PROFILE_ACTIVE.store(false, Ordering::Release);
        Ok(())
    }

    pub fn maybe_profile<T, F>(mut f: F) -> Result<T>
    where
        F: FnMut() -> Result<T>,
    {
        if PROFILE_ACTIVE.load(Ordering::Relaxed) {
            return f();
        }

        let config = env_profile_config();
        let Some(output_path) = config.output.as_ref() else {
            return f();
        };

        if config.profile_once && PROFILED_ONCE.swap(true, Ordering::SeqCst) {
            return f();
        }

        struct ActiveGuard;
        impl Drop for ActiveGuard {
            fn drop(&mut self) {
                PROFILE_ACTIVE.store(false, Ordering::Release);
            }
        }
        PROFILE_ACTIVE.store(true, Ordering::Release);
        let _active = ActiveGuard;

        let guard = pprof::ProfilerGuardBuilder::default()
            .frequency(config.frequency)
            .blocklist(&["libc", "libpthread", "libgcc", "libm"])
            .build()
            .map_err(|err| ConversionError::Other(format!("Profiling init failed: {err}")))?;

        let mut last_result = None;
        for _ in 0..config.repeat {
            let value = f()?;
            last_result = Some(value);
        }
        let result =
            last_result.ok_or_else(|| ConversionError::Other("Profiling repeat produced no result".to_string()))?;

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).map_err(ConversionError::IoError)?;
        }

        let report = guard
            .report()
            .build()
            .map_err(|err| ConversionError::Other(format!("Profiling report failed: {err}")))?;

        let file = std::fs::File::create(output_path).map_err(ConversionError::IoError)?;
        report
            .flamegraph(file)
            .map_err(|err| ConversionError::Other(format!("Flamegraph write failed: {err}")))?;

        Ok(result)
    }
}

#[cfg(all(not(target_os = "windows"), feature = "profiling"))]
pub use enabled::{maybe_profile, start, stop};

#[cfg(target_os = "windows")]
pub fn start(_output_path: PathBuf, _frequency: i32) -> Result<()> {
    Err(ConversionError::Other(
        "Profiling is not supported on Windows".to_string(),
    ))
}

#[cfg(all(not(target_os = "windows"), not(feature = "profiling")))]
pub fn start(_output_path: PathBuf, _frequency: i32) -> Result<()> {
    Err(ConversionError::Other(
        "Profiling is disabled; rebuild with the profiling feature".to_string(),
    ))
}

#[cfg(target_os = "windows")]
pub fn stop() -> Result<()> {
    Err(ConversionError::Other(
        "Profiling is not supported on Windows".to_string(),
    ))
}

#[cfg(all(not(target_os = "windows"), not(feature = "profiling")))]
pub fn stop() -> Result<()> {
    Err(ConversionError::Other(
        "Profiling is disabled; rebuild with the profiling feature".to_string(),
    ))
}

#[cfg(any(target_os = "windows", not(feature = "profiling")))]
pub fn maybe_profile<T, F>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    f()
}
