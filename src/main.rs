mod config;
mod export;
mod monitor;
mod platform;
mod policy;
mod tree;

use std::collections::{HashMap, HashSet};
use std::env;
use std::path::PathBuf;
use std::time::Duration;

struct Args {
    config_path: PathBuf,
    enforce: bool,
    stealth: bool,
    daemon: bool,
    interval_ms: u64,
    cpu_threshold: f64,
    ram_threshold: f64,
    sustain_samples: u32,
    sustain_seconds: Option<u64>,
    spike_delta: f64,
    export_csv: Option<PathBuf>,
    export_jsonl: Option<PathBuf>,
    export_all_samples: bool,
    audit_log: Option<PathBuf>,
    profile: Option<String>,
    no_ignore_system: bool,
    show_help: bool,
}

fn usage() -> &'static str {
    "process_tracker [OPTIONS]

Options:
  --daemon              Activar modo daemon (loop continuo)
  --profile NAME        Cargar profiles/NAME.txt como allowlist
  --audit-log [FILE]    Log de eventos (default: audit.log)
  --no-ignore-system    No auto-detectar procesos de macOS
  --config FILE         allowlist file (default: allowlist.txt)
  --enforce             kill unknown processes
  --stealth             monitor anomalies only (CPU/RAM)
  --interval MS         sample interval in ms (default: 1000)
  --cpu-threshold PCT   CPU anomaly threshold (default: 80)
  --ram-threshold PCT   RAM anomaly threshold (default: 20)
  --sustain N           samples needed to flag sustained anomaly (default: 3)
  --sustain-seconds S   seconds needed to flag sustained anomaly
  --spike-delta PCT     delta CPU spike threshold (default: 30)
  --export-csv [FILE]   export CSV (default: export.csv)
  --export-jsonl [FILE] export JSONL (default: export.jsonl)
  --export-all-samples  export every sample in stealth mode
  -h, --help            show help\n"
}

fn parse_args() -> Result<Args, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut parsed = Args {
        config_path: PathBuf::from("allowlist.txt"),
        enforce: false,
        stealth: false,
        daemon: false,
        interval_ms: 1000,
        cpu_threshold: 80.0,
        ram_threshold: 20.0,
        sustain_samples: 3,
        sustain_seconds: None,
        spike_delta: 30.0,
        export_csv: None,
        export_jsonl: None,
        export_all_samples: false,
        audit_log: None,
        profile: None,
        no_ignore_system: false,
        show_help: false,
    };

    let mut idx = 0usize;
    while idx < args.len() {
        let arg = &args[idx];
        match arg.as_str() {
            "--config" => {
                let value = args.get(idx + 1).ok_or("missing --config value")?;
                parsed.config_path = PathBuf::from(value);
                idx += 1;
            }
            "--enforce" => parsed.enforce = true,
            "--stealth" => parsed.stealth = true,
            "--interval" => {
                let value = args.get(idx + 1).ok_or("missing --interval value")?;
                parsed.interval_ms = value.parse().map_err(|_| "bad --interval")?;
                idx += 1;
            }
            "--cpu-threshold" => {
                let value = args.get(idx + 1).ok_or("missing --cpu-threshold value")?;
                parsed.cpu_threshold = value.parse().map_err(|_| "bad --cpu-threshold")?;
                idx += 1;
            }
            "--ram-threshold" => {
                let value = args.get(idx + 1).ok_or("missing --ram-threshold value")?;
                parsed.ram_threshold = value.parse().map_err(|_| "bad --ram-threshold")?;
                idx += 1;
            }
            "--sustain" => {
                let value = args.get(idx + 1).ok_or("missing --sustain value")?;
                parsed.sustain_samples = value.parse().map_err(|_| "bad --sustain")?;
                idx += 1;
            }
            "--sustain-seconds" => {
                let value = args.get(idx + 1).ok_or("missing --sustain-seconds value")?;
                parsed.sustain_seconds = Some(value.parse().map_err(|_| "bad --sustain-seconds")?);
                idx += 1;
            }
            "--spike-delta" => {
                let value = args.get(idx + 1).ok_or("missing --spike-delta value")?;
                parsed.spike_delta = value.parse().map_err(|_| "bad --spike-delta")?;
                idx += 1;
            }
            "--export-csv" => {
                if let Some(value) = args.get(idx + 1) {
                    if !value.starts_with('-') {
                        parsed.export_csv = Some(PathBuf::from(value));
                        idx += 1;
                    } else {
                        parsed.export_csv = Some(PathBuf::from("export.csv"));
                    }
                } else {
                    parsed.export_csv = Some(PathBuf::from("export.csv"));
                }
            }
            "--export-jsonl" => {
                if let Some(value) = args.get(idx + 1) {
                    if !value.starts_with('-') {
                        parsed.export_jsonl = Some(PathBuf::from(value));
                        idx += 1;
                    } else {
                        parsed.export_jsonl = Some(PathBuf::from("export.jsonl"));
                    }
                } else {
                    parsed.export_jsonl = Some(PathBuf::from("export.jsonl"));
                }
            }
            "--export-all-samples" => {
                parsed.export_all_samples = true;
            }
            "--daemon" => parsed.daemon = true,
            "--no-ignore-system" => parsed.no_ignore_system = true,
            "--audit-log" => {
                if let Some(value) = args.get(idx + 1) {
                    if !value.starts_with('-') {
                        parsed.audit_log = Some(PathBuf::from(value));
                        idx += 1;
                    } else {
                        parsed.audit_log = Some(PathBuf::from("audit.log"));
                    }
                } else {
                    parsed.audit_log = Some(PathBuf::from("audit.log"));
                }
            }
            "--profile" => {
                let value = args.get(idx + 1).ok_or("missing --profile value")?;
                parsed.config_path = PathBuf::from(format!("profiles/{}.txt", value));
                parsed.profile = Some(value.clone());
                idx += 1;
            }
            "-h" | "--help" => parsed.show_help = true,
            _ => return Err(format!("unknown argument: {}", arg)),
        }
        idx += 1;
    }

    Ok(parsed)
}

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(err) => {
            eprintln!("error: {}", err);
            eprintln!("{}", usage());
            std::process::exit(2);
        }
    };

    if args.show_help {
        println!("{}", usage());
        return;
    }

    let export_config = export::ExportConfig {
        csv_path: args.export_csv.clone(),
        jsonl_path: args.export_jsonl.clone(),
    };
    let mut exporter = match export::Exporter::new(&export_config) {
        Ok(exporter) => exporter,
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(2);
        }
    };

    if args.stealth {
        let cfg = monitor::MonitorConfig {
            interval_ms: args.interval_ms,
            cpu_threshold: args.cpu_threshold,
            ram_threshold: args.ram_threshold,
            sustain_samples: args.sustain_samples,
            sustain_seconds: args.sustain_seconds,
            spike_delta: args.spike_delta,
            export_all_samples: args.export_all_samples,
        };
        if let Err(err) = monitor::run_stealth(cfg, exporter.as_mut()) {
            eprintln!("error: {}", err);
            std::process::exit(2);
        }
        return;
    }

    let allowlist = match config::load_allowlist(&args.config_path) {
        Ok(allowlist) => allowlist,
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(2);
        }
    };

    if args.daemon {
        run_daemon_loop(args, allowlist, exporter.as_mut());
    } else {
        run_single_shot(args, allowlist, exporter.as_mut());
    }
}

fn run_single_shot(args: Args, allowlist: config::Allowlist, mut exporter: Option<&mut export::Exporter>) {
    let processes = match platform::list_processes() {
        Ok(processes) => processes,
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(2);
        }
    };

    let tree = tree::ProcTree::from_processes(processes);
    let mut hash_cache = HashMap::new();
    for proc in tree.walk() {
        if policy::is_allowed(proc, &allowlist, &mut hash_cache) {
            continue;
        }

        let path = proc.path.as_deref().unwrap_or("-");
        println!(
            "\x1b[31m[UNKNOWN]\x1b[0m pid={} uid={} ppid={} name=\x1b[1m{}\x1b[0m path=\x1b[36m{}\x1b[0m",
            proc.pid, proc.uid, proc.ppid, proc.name, path
        );

        if let Some(exporter) = exporter.as_deref_mut() {
            let ts = export::now_ts();
            let event = export::UnknownEvent {
                ts,
                pid: proc.pid,
                uid: proc.uid,
                ppid: proc.ppid,
                name: &proc.name,
                path: proc.path.as_deref(),
            };
            if let Err(err) = exporter.write_unknown(&event) {
                eprintln!("export failed pid={} err={}", proc.pid, err);
            }
        }

        if args.enforce {
            match platform::kill_process(proc.pid) {
                Ok(()) => println!("\x1b[31m\x1b[1m[KILLED]\x1b[0m pid={}", proc.pid),
                Err(err) => eprintln!("\x1b[31m[ERROR]\x1b[0m kill failed pid={} err={}", proc.pid, err),
            }
        }
    }
}

fn run_daemon_loop(args: Args, allowlist: config::Allowlist, mut exporter: Option<&mut export::Exporter>) {
    let mut audit_writer = args.audit_log.as_ref().and_then(|path| {
        export::Exporter::new(&export::ExportConfig {
            csv_path: None,
            jsonl_path: Some(path.clone()),
        })
        .ok()
        .flatten()
    });

    let self_pid = std::process::id();
    let num_cpus = platform::num_cpus().unwrap_or(1) as f64;
    let total_mem = platform::total_mem_bytes().unwrap_or(1) as f64;

    let mut reported_unknowns = HashSet::new();
    let mut prev_cpu: HashMap<u32, u64> = HashMap::new();
    let mut hash_cache = HashMap::new();

    let logo = "\n\x1b[36m  _____                       \x1b[35m _______             _              \x1b[0m\n\
                \x1b[36m |  __ \\                      \x1b[35m|__   __|           | |             \x1b[0m\n\
                \x1b[36m | |__) | __ ___   ___ ___  ___  \x1b[35m| | _ __ __ _  ___| | _____ _ __  \x1b[0m\n\
                \x1b[36m |  ___/ '__/ _ \\ / __/ _ \\/ __| \x1b[35m| || '__/ _` |/ __| |/ / _ \\ '__|\x1b[0m\n\
                \x1b[36m | |   | | | (_) | (_|  __/\\__ \\ \x1b[35m| || | | (_| | (__|   <  __/ |    \x1b[0m\n\
                \x1b[36m |_|   |_|  \\___/ \\___\\___||___/ \x1b[35m|_||_|  \\__,_|\\___|_|\\_\\___|_|    \x1b[0m\n";
    println!("{}", logo);

    if let Some(profile_name) = &args.profile {
        println!("\x1b[1m\x1b[34m[PROFILE]\x1b[0m Activo: \x1b[32m{}\x1b[0m", profile_name);
    } else {
        println!("\x1b[1m\x1b[34m[PROFILE]\x1b[0m Activo: \x1b[32mdefault (allowlist.txt)\x1b[0m");
    }

    if !allowlist.names.is_empty() || !allowlist.paths.is_empty() {
        println!("\x1b[1m\x1b[34m[RULES]\x1b[0m Procesos listados en perfil:");
        for name in &allowlist.names {
            println!("  - \x1b[36m[Name]\x1b[0m {}", name);
        }
        for path in &allowlist.paths {
            println!("  - \x1b[35m[Path]\x1b[0m {}", path);
        }
        println!();
    }

    println!("\x1b[32m\x1b[1m🛡️  Process Tracker Daemon started. Monitoring processes...\x1b[0m\n");

    loop {
        let processes = match platform::list_processes() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("error fetching processes: {}", e);
                std::thread::sleep(Duration::from_millis(args.interval_ms));
                continue;
            }
        };

        let tree = tree::ProcTree::from_processes(processes);
        let mut alive_pids = HashSet::new();

        for proc in tree.walk() {
            alive_pids.insert(proc.pid);

            // 1. Self check
            if proc.pid == self_pid {
                continue;
            }

            // 2. macOS System process check
            if !args.no_ignore_system && platform::is_system_process(proc) {
                if let Ok(sample) = platform::sample_process(proc.pid) {
                    if let Some(prev_ns) = prev_cpu.get(&proc.pid) {
                        let delta_cpu = sample.cpu_ns.saturating_sub(*prev_ns) as f64;
                        let interval_ns = (args.interval_ms as f64) * 1_000_000.0;
                        let cpu_pct = if interval_ns > 0.0 && num_cpus > 0.0 {
                            (delta_cpu / interval_ns / num_cpus) * 100.0
                        } else {
                            0.0
                        };
                        let ram_pct = if total_mem > 0.0 {
                            (sample.rss_bytes as f64 / total_mem) * 100.0
                        } else {
                            0.0
                        };

                        if cpu_pct >= args.cpu_threshold || ram_pct >= args.ram_threshold {
                            println!(
                                "\x1b[33m[OVERLOAD]\x1b[0m pid={} name=\x1b[1m{}\x1b[0m cpu=\x1b[31m{:.2}%\x1b[0m ram=\x1b[31m{:.2}%\x1b[0m",
                                proc.pid, proc.name, cpu_pct, ram_pct
                            );
                            
                            let ts = export::now_ts();
                            let event = export::SystemOverloadEvent {
                                ts,
                                pid: proc.pid,
                                name: &proc.name,
                                path: proc.path.as_deref(),
                                cpu_pct,
                                ram_pct,
                            };
                            
                            if let Some(exp) = exporter.as_deref_mut() {
                                let _ = exp.write_system_overload(&event);
                            }
                            if let Some(audit) = audit_writer.as_mut() {
                                let _ = audit.write_system_overload(&event);
                            }
                        }
                    }
                    prev_cpu.insert(proc.pid, sample.cpu_ns);
                }
                continue; // Do not apply allowlist checks to system processes
            }

            // 3. Allowlist check
            if policy::is_allowed(proc, &allowlist, &mut hash_cache) {
                continue;
            }

            // 4. Report & Log (Deduplicated)
            if reported_unknowns.insert(proc.pid) {
                let path = proc.path.as_deref().unwrap_or("-");
                let action = if args.enforce { "\x1b[31mkilled\x1b[0m" } else { "\x1b[33mlogged\x1b[0m" };
                
                println!(
                    "\x1b[31m[UNKNOWN]\x1b[0m pid={} uid={} ppid={} name=\x1b[1m{}\x1b[0m path=\x1b[36m{}\x1b[0m action={}",
                    proc.pid, proc.uid, proc.ppid, proc.name, path, action
                );

                let ts = export::now_ts();
                let event = export::AuditEvent {
                    ts,
                    pid: proc.pid,
                    uid: proc.uid,
                    ppid: proc.ppid,
                    name: &proc.name,
                    path: proc.path.as_deref(),
                    action: if args.enforce { "killed" } else { "logged" },
                };

                if let Some(exp) = exporter.as_deref_mut() {
                    let _ = exp.write_audit(&event);
                }
                if let Some(audit) = audit_writer.as_mut() {
                    let _ = audit.write_audit(&event);
                }

                // 5. Enforce 
                if args.enforce {
                    match platform::kill_process(proc.pid) {
                        Ok(()) => println!("\x1b[31m\x1b[1m[KILLED]\x1b[0m pid={}", proc.pid),
                        Err(err) => eprintln!("\x1b[31m[ERROR]\x1b[0m kill failed pid={} err={}", proc.pid, err),
                    }
                }
            }
        }

        // Cleanup tracked PID state for processes that died
        reported_unknowns.retain(|pid| alive_pids.contains(pid));
        prev_cpu.retain(|pid, _| alive_pids.contains(pid));

        std::thread::sleep(Duration::from_millis(args.interval_ms));
    }
}
