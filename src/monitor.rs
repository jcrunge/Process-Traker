use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::export;
use crate::platform;

pub struct MonitorConfig {
    pub interval_ms: u64,
    pub cpu_threshold: f64,
    pub ram_threshold: f64,
    pub sustain_samples: u32,
    pub sustain_seconds: Option<u64>,
    pub spike_delta: f64,
    pub export_all_samples: bool,
}

struct PrevSample {
    cpu_ns: u64,
    last_cpu_pct: f64,
    sustain_count: u32,
}

pub fn run_stealth(cfg: MonitorConfig, mut exporter: Option<&mut export::Exporter>) -> Result<(), String> {
    let num_cpus = platform::num_cpus()? as f64;
    let total_mem = platform::total_mem_bytes()? as f64;
    let sustain_samples = resolve_sustain_samples(cfg.interval_ms, cfg.sustain_samples, cfg.sustain_seconds);
    let mut prev: HashMap<u32, PrevSample> = HashMap::new();

    loop {
        let start = Instant::now();
        let processes = platform::list_processes()?;
        let mut next: HashMap<u32, PrevSample> = HashMap::new();

        for proc in processes {
            let sample = match platform::sample_process(proc.pid) {
                Ok(sample) => sample,
                Err(_) => continue,
            };

            let (cpu_pct, spike, sustain_count) = if let Some(prev_sample) = prev.get(&proc.pid) {
                let delta_cpu = sample.cpu_ns.saturating_sub(prev_sample.cpu_ns) as f64;
                let interval_ns = (cfg.interval_ms as f64) * 1_000_000.0;
                let mut cpu_pct = if interval_ns > 0.0 && num_cpus > 0.0 {
                    (delta_cpu / interval_ns / num_cpus) * 100.0
                } else {
                    0.0
                };
                if cpu_pct < 0.0 {
                    cpu_pct = 0.0;
                }

                let spike = cpu_pct - prev_sample.last_cpu_pct >= cfg.spike_delta;
                let mut sustain = prev_sample.sustain_count;
                let ram_pct = if total_mem > 0.0 {
                    (sample.rss_bytes as f64 / total_mem) * 100.0
                } else {
                    0.0
                };

                if cpu_pct >= cfg.cpu_threshold || ram_pct >= cfg.ram_threshold {
                    sustain = sustain.saturating_add(1);
                } else {
                    sustain = 0;
                }
                (cpu_pct, spike, sustain)
            } else {
                (0.0, false, 0)
            };

            let ram_pct = if total_mem > 0.0 {
                (sample.rss_bytes as f64 / total_mem) * 100.0
            } else {
                0.0
            };

            let sustained = sustain_samples > 0 && sustain_count >= sustain_samples;
            let ts = export::now_ts();

            if cfg.export_all_samples {
                if let Some(exporter) = exporter.as_deref_mut() {
                    let event = export::SampleEvent {
                        ts,
                        pid: proc.pid,
                        uid: proc.uid,
                        ppid: proc.ppid,
                        name: &proc.name,
                        path: proc.path.as_deref(),
                        cpu_pct,
                        ram_pct,
                    };
                    if let Err(err) = exporter.write_sample(&event) {
                        eprintln!("export failed pid={} err={}", proc.pid, err);
                    }
                }
            }

            if spike || sustained {
                let reason = anomaly_reason(spike, sustained);
                report_anomaly(
                    ts,
                    proc.pid,
                    &proc.name,
                    proc.path.as_deref(),
                    cpu_pct,
                    ram_pct,
                    reason,
                );
                if let Some(exporter) = exporter.as_deref_mut() {
                    let event = export::AnomalyEvent {
                        ts,
                        pid: proc.pid,
                        name: &proc.name,
                        path: proc.path.as_deref(),
                        cpu_pct,
                        ram_pct,
                        reason,
                    };
                    if let Err(err) = exporter.write_anomaly(&event) {
                        eprintln!("export failed pid={} err={}", proc.pid, err);
                    }
                }
            }

            next.insert(
                proc.pid,
                PrevSample {
                    cpu_ns: sample.cpu_ns,
                    last_cpu_pct: cpu_pct,
                    sustain_count,
                },
            );
        }

        prev = next;

        let elapsed = start.elapsed();
        let interval = Duration::from_millis(cfg.interval_ms);
        if elapsed < interval {
            std::thread::sleep(interval - elapsed);
        }
    }
}

fn resolve_sustain_samples(interval_ms: u64, fallback: u32, sustain_seconds: Option<u64>) -> u32 {
    if let Some(seconds) = sustain_seconds {
        if interval_ms == 0 {
            return fallback;
        }
        let intervals = (seconds * 1000 + interval_ms - 1) / interval_ms;
        return intervals as u32;
    }
    fallback
}

fn anomaly_reason(spike: bool, sustained: bool) -> &'static str {
    match (spike, sustained) {
        (true, true) => "spike+sustained",
        (true, false) => "spike",
        (false, true) => "sustained",
        (false, false) => "threshold",
    }
}

fn report_anomaly(
    ts: u64,
    pid: u32,
    name: &str,
    path: Option<&str>,
    cpu_pct: f64,
    ram_pct: f64,
    reason: &str,
) {
    let path = path.unwrap_or("-");
    println!(
        "anomaly ts={} pid={} name={} path={} cpu={:.2} ram={:.2} reason={}",
        ts, pid, name, path, cpu_pct, ram_pct, reason
    );
}
