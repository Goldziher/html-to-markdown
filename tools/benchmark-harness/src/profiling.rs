use crate::Result;
#[cfg(all(feature = "profiling", not(target_os = "windows")))]
use std::path::Path;

#[cfg(all(feature = "profiling", not(target_os = "windows")))]
use std::time::Duration;

#[cfg(all(feature = "profiling", not(target_os = "windows")))]
pub struct ProfileGuard {
    guard: Option<pprof::ProfilerGuard<'static>>,
    start_time: std::time::Instant,
}

#[cfg(all(feature = "profiling", not(target_os = "windows")))]
impl ProfileGuard {
    pub fn new(frequency: i32) -> Result<Self> {
        let guard = pprof::ProfilerGuardBuilder::default()
            .frequency(frequency)
            .blocklist(&["libc", "libpthread", "libgcc", "libm"])
            .build()
            .map_err(|e| crate::Error::Profiling(format!("Failed to initialize profiler: {e}")))?;

        Ok(Self {
            guard: Some(guard),
            start_time: std::time::Instant::now(),
        })
    }

    pub fn finish(mut self) -> Result<ProfilingResult> {
        let duration = self.start_time.elapsed();
        let guard = self
            .guard
            .take()
            .ok_or_else(|| crate::Error::Profiling("Profiler already finished".to_string()))?;

        let report = guard
            .report()
            .build()
            .map_err(|e| crate::Error::Profiling(format!("Failed to generate profiler report: {e}")))?;

        Ok(ProfilingResult {
            duration,
            sample_count: 0,
            report,
        })
    }
}

#[cfg(all(feature = "profiling", not(target_os = "windows")))]
pub struct ProfilingResult {
    pub duration: Duration,
    pub sample_count: usize,
    pub report: pprof::Report,
}

#[cfg(all(feature = "profiling", not(target_os = "windows")))]
impl ProfilingResult {
    pub fn generate_flamegraph(&self, output_path: &Path) -> Result<()> {
        if let Some(parent) = output_path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::Error::Profiling(format!("Failed to create output directory: {e}")))?;
        }

        let file = std::fs::File::create(output_path)
            .map_err(|e| crate::Error::Profiling(format!("Failed to create output file: {e}")))?;

        self.report
            .flamegraph(file)
            .map_err(|e| crate::Error::Profiling(format!("Failed to generate flamegraph: {e}")))?;

        Ok(())
    }
}

#[cfg(any(not(feature = "profiling"), target_os = "windows"))]
pub struct ProfileGuard;

#[cfg(any(not(feature = "profiling"), target_os = "windows"))]
impl ProfileGuard {
    pub fn new(_frequency: i32) -> Result<Self> {
        Err(crate::Error::Profiling(
            "Profiling is disabled or unsupported on this platform".to_string(),
        ))
    }

    pub fn finish(self) -> Result<()> {
        Err(crate::Error::Profiling(
            "Profiling is disabled or unsupported on this platform".to_string(),
        ))
    }
}
