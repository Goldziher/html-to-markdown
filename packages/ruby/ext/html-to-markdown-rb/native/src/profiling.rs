use html_to_markdown_rs::{ConversionError, Result};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

const ENV_OUTPUT: &str = "HTML_TO_MARKDOWN_PROFILE_OUTPUT";
const ENV_FREQUENCY: &str = "HTML_TO_MARKDOWN_PROFILE_FREQUENCY";
const ENV_ONCE: &str = "HTML_TO_MARKDOWN_PROFILE_ONCE";

static PROFILED_ONCE: AtomicBool = AtomicBool::new(false);

#[cfg(not(target_os = "windows"))]
struct ProfileState {
    guard: Option<pprof::ProfilerGuard<'static>>,
    output: Option<PathBuf>,
}

#[cfg(not(target_os = "windows"))]
fn state() -> &'static Mutex<ProfileState> {
    static STATE: OnceLock<Mutex<ProfileState>> = OnceLock::new();
    STATE.get_or_init(|| {
        Mutex::new(ProfileState {
            guard: None,
            output: None,
        })
    })
}

#[cfg(not(target_os = "windows"))]
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
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn stop() -> Result<()> {
    let (guard, output) = {
        let mut state = state()
            .lock()
            .map_err(|_| ConversionError::Other("profiling state lock poisoned".to_string()))?;
        let guard = state.guard.take();
        let output = state.output.take();
        (guard, output)
    };

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
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn start(_output_path: PathBuf, _frequency: i32) -> Result<()> {
    Err(ConversionError::Other(
        "Profiling is not supported on Windows".to_string(),
    ))
}

#[cfg(target_os = "windows")]
pub fn stop() -> Result<()> {
    Err(ConversionError::Other(
        "Profiling is not supported on Windows".to_string(),
    ))
}

#[cfg(not(target_os = "windows"))]
pub fn maybe_profile<T, F>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    if let Ok(state) = state().lock() {
        if state.guard.is_some() {
            return f();
        }
    }

    let output_path = match std::env::var(ENV_OUTPUT) {
        Ok(value) if !value.trim().is_empty() => Some(PathBuf::from(value)),
        _ => None,
    };

    let Some(output_path) = output_path else {
        return f();
    };

    let profile_once = match std::env::var(ENV_ONCE) {
        Ok(value) => !matches!(value.as_str(), "0" | "false" | "no"),
        Err(_) => true,
    };

    if profile_once && PROFILED_ONCE.swap(true, Ordering::SeqCst) {
        return f();
    }

    let frequency = std::env::var(ENV_FREQUENCY)
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(1000);

    let guard = pprof::ProfilerGuardBuilder::default()
        .frequency(frequency)
        .blocklist(&["libc", "libpthread", "libgcc", "libm"])
        .build()
        .map_err(|err| ConversionError::Other(format!("Profiling init failed: {err}")))?;

    let result = f();

    if result.is_ok() {
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
    }

    result
}

#[cfg(target_os = "windows")]
pub fn maybe_profile<T, F>(f: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    f()
}
