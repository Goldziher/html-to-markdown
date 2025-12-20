use crate::types::ResourceStats;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Debug, Clone, Copy)]
struct ResourceSample {
    memory_bytes: u64,
    cpu_percent: f64,
}

pub struct ResourceMonitor {
    samples: Arc<Mutex<Vec<ResourceSample>>>,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl ResourceMonitor {
    pub fn start(child_pid: u32, interval: Duration) -> Option<Self> {
        let pid = pid_from(child_pid)?;
        let samples = Arc::new(Mutex::new(Vec::new()));
        let running = Arc::new(AtomicBool::new(true));

        let samples_clone = Arc::clone(&samples);
        let running_clone = Arc::clone(&running);

        let handle = thread::spawn(move || {
            let mut system = System::new();
            let refresh_kind = ProcessRefreshKind::nothing().with_cpu().with_memory();

            while running_clone.load(Ordering::Relaxed) {
                system.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), false, refresh_kind);

                if let Some(process) = system.process(pid) {
                    samples_clone
                        .lock()
                        .expect("monitor samples poisoned")
                        .push(ResourceSample {
                            memory_bytes: process.memory(),
                            cpu_percent: process.cpu_usage() as f64,
                        });
                }

                thread::sleep(interval);
            }
        });

        Some(Self {
            samples,
            running,
            handle: Some(handle),
        })
    }

    pub fn stop(mut self) -> ResourceStats {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        let samples = self.samples.lock().expect("monitor samples poisoned");
        ResourceStats::from_samples(&samples)
    }
}

impl ResourceStats {
    fn from_samples(samples: &[ResourceSample]) -> Self {
        if samples.is_empty() {
            return Self::default();
        }

        let peak_memory_bytes = samples.iter().map(|s| s.memory_bytes).max().unwrap_or_default();
        let avg_cpu_percent = samples.iter().map(|s| s.cpu_percent).sum::<f64>() / samples.len() as f64;
        let mut memories: Vec<u64> = samples.iter().map(|s| s.memory_bytes).collect();
        memories.sort_unstable();

        let p50_memory_bytes = percentile(&memories, 0.50);
        let p95_memory_bytes = percentile(&memories, 0.95);
        let p99_memory_bytes = percentile(&memories, 0.99);

        Self {
            peak_memory_bytes,
            avg_cpu_percent,
            p50_memory_bytes,
            p95_memory_bytes,
            p99_memory_bytes,
        }
    }
}

fn percentile(values: &[u64], percentile: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let idx = ((values.len() as f64 - 1.0) * percentile).round() as usize;
    values[idx.min(values.len() - 1)]
}

fn pid_from(child_pid: u32) -> Option<Pid> {
    #[cfg(any(unix, windows))]
    {
        Some(Pid::from_u32(child_pid))
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = child_pid;
        None
    }
}
