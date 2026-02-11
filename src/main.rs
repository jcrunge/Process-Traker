mod config;
mod export;
mod monitor;
mod platform;
mod policy;
mod tree;

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

struct Args {
    config_path: PathBuf,
    enforce: bool,
    stealth: bool,
    interval_ms: u64,
    cpu_threshold: f64,
    ram_threshold: f64,
    sustain_samples: u32,
    sustain_seconds: Option<u64>,
    spike_delta: f64,
    export_csv: Option<PathBuf>,
    export_jsonl: Option<PathBuf>,
    export_all_samples: bool,
    show_help: bool,
}

fn usage() -> &'static str {
    "process_tracker [--config FILE] [--enforce] [--stealth]\n\
    \n\
    Options:\n\
      --config FILE         allowlist file (default: allowlist.txt)\n\
      --enforce             kill unknown processes (non-stealth only)\n\
      --stealth             monitor anomalies only (CPU/RAM)\n\
      --interval MS         sample interval in ms (default: 1000)\n\
      --cpu-threshold PCT   CPU anomaly threshold (default: 80)\n\
      --ram-threshold PCT   RAM anomaly threshold (default: 20)\n\
      --sustain N           samples needed to flag sustained anomaly (default: 3)\n\
      --sustain-seconds S   seconds needed to flag sustained anomaly\n\
      --spike-delta PCT     delta CPU spike threshold (default: 30)\n\
      --export-csv [FILE]   export CSV (default: export.csv)\n\
      --export-jsonl [FILE] export JSONL (default: export.jsonl)\n\
      --export-all-samples  export every sample in stealth mode\n\
      -h, --help            show help\n"
}

fn parse_args() -> Result<Args, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut parsed = Args {
        config_path: PathBuf::from("allowlist.txt"),
        enforce: false,
        stealth: false,
        interval_ms: 1000,
        cpu_threshold: 80.0,
        ram_threshold: 20.0,
        sustain_samples: 3,
        sustain_seconds: None,
        spike_delta: 30.0,
        export_csv: None,
        export_jsonl: None,
        export_all_samples: false,
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
            "unknown pid={} uid={} ppid={} name={} path={}",
            proc.pid, proc.uid, proc.ppid, proc.name, path
        );

        if let Some(exporter) = exporter.as_mut() {
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
                Ok(()) => println!("killed pid={}", proc.pid),
                Err(err) => eprintln!("kill failed pid={} err={}", proc.pid, err),
            }
        }
    }
}
