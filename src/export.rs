use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub struct ExportConfig {
    pub csv_path: Option<PathBuf>,
    pub jsonl_path: Option<PathBuf>,
}

pub struct Exporter {
    csv: Option<BufWriter<File>>,
    jsonl: Option<BufWriter<File>>,
    csv_has_header: bool,
}

pub struct UnknownEvent<'a> {
    pub ts: u64,
    pub pid: u32,
    pub uid: u32,
    pub ppid: u32,
    pub name: &'a str,
    pub path: Option<&'a str>,
}

pub struct AnomalyEvent<'a> {
    pub ts: u64,
    pub pid: u32,
    pub name: &'a str,
    pub path: Option<&'a str>,
    pub cpu_pct: f64,
    pub ram_pct: f64,
    pub reason: &'a str,
}

pub struct SampleEvent<'a> {
    pub ts: u64,
    pub pid: u32,
    pub uid: u32,
    pub ppid: u32,
    pub name: &'a str,
    pub path: Option<&'a str>,
    pub cpu_pct: f64,
    pub ram_pct: f64,
}

impl Exporter {
    pub fn new(config: &ExportConfig) -> Result<Option<Exporter>, String> {
        let csv = match config.csv_path.as_ref() {
            Some(path) => Some(open_append(path)?),
            None => None,
        };
        let jsonl = match config.jsonl_path.as_ref() {
            Some(path) => Some(open_append(path)?),
            None => None,
        };

        if csv.is_none() && jsonl.is_none() {
            return Ok(None);
        }

        let csv_has_header = if let Some(path) = config.csv_path.as_ref() {
            file_has_content(path)
        } else {
            false
        };

        Ok(Some(Exporter {
            csv,
            jsonl,
            csv_has_header,
        }))
    }

    pub fn write_unknown(&mut self, event: &UnknownEvent) -> Result<(), String> {
        self.write_csv(
            event.ts,
            "unknown",
            Some(event.pid),
            Some(event.uid),
            Some(event.ppid),
            Some(event.name),
            event.path,
            None,
            None,
            None,
        )?;
        self.write_json(event.ts, "unknown", event.pid, Some(event.uid), Some(event.ppid), event.name, event.path, None, None, None)?;
        Ok(())
    }

    pub fn write_anomaly(&mut self, event: &AnomalyEvent) -> Result<(), String> {
        self.write_csv(
            event.ts,
            "anomaly",
            Some(event.pid),
            None,
            None,
            Some(event.name),
            event.path,
            Some(event.cpu_pct),
            Some(event.ram_pct),
            Some(event.reason),
        )?;
        self.write_json(
            event.ts,
            "anomaly",
            event.pid,
            None,
            None,
            event.name,
            event.path,
            Some(event.cpu_pct),
            Some(event.ram_pct),
            Some(event.reason),
        )?;
        Ok(())
    }

    pub fn write_sample(&mut self, event: &SampleEvent) -> Result<(), String> {
        self.write_csv(
            event.ts,
            "sample",
            Some(event.pid),
            Some(event.uid),
            Some(event.ppid),
            Some(event.name),
            event.path,
            Some(event.cpu_pct),
            Some(event.ram_pct),
            None,
        )?;
        self.write_json(
            event.ts,
            "sample",
            event.pid,
            Some(event.uid),
            Some(event.ppid),
            event.name,
            event.path,
            Some(event.cpu_pct),
            Some(event.ram_pct),
            None,
        )?;
        Ok(())
    }

    fn write_csv(
        &mut self,
        ts: u64,
        kind: &str,
        pid: Option<u32>,
        uid: Option<u32>,
        ppid: Option<u32>,
        name: Option<&str>,
        path: Option<&str>,
        cpu: Option<f64>,
        ram: Option<f64>,
        reason: Option<&str>,
    ) -> Result<(), String> {
        let Some(writer) = self.csv.as_mut() else {
            return Ok(());
        };
        if !self.csv_has_header {
            writer
                .write_all(b"ts,kind,pid,uid,ppid,name,path,cpu,ram,reason\n")
                .map_err(|err| err.to_string())?;
            self.csv_has_header = true;
        }

        let fields = [
            ts.to_string(),
            kind.to_string(),
            opt_u32(pid),
            opt_u32(uid),
            opt_u32(ppid),
            opt_str(name),
            opt_str(path),
            opt_f64(cpu),
            opt_f64(ram),
            opt_str(reason),
        ];
        let mut line = String::new();
        for (idx, field) in fields.iter().enumerate() {
            if idx > 0 {
                line.push(',');
            }
            line.push_str(&csv_escape(field));
        }
        line.push('\n');
        writer.write_all(line.as_bytes()).map_err(|err| err.to_string())
    }

    fn write_json(
        &mut self,
        ts: u64,
        kind: &str,
        pid: u32,
        uid: Option<u32>,
        ppid: Option<u32>,
        name: &str,
        path: Option<&str>,
        cpu: Option<f64>,
        ram: Option<f64>,
        reason: Option<&str>,
    ) -> Result<(), String> {
        let Some(writer) = self.jsonl.as_mut() else {
            return Ok(());
        };

        let mut line = String::new();
        line.push('{');
        line.push_str(&format!("\"ts\":{},", ts));
        line.push_str(&format!("\"kind\":\"{}\",", json_escape(kind)));
        line.push_str(&format!("\"pid\":{},", pid));
        line.push_str(&json_opt_u32("uid", uid));
        line.push_str(&json_opt_u32("ppid", ppid));
        line.push_str(&format!("\"name\":\"{}\",", json_escape(name)));
        line.push_str(&json_opt_str("path", path));
        line.push_str(&json_opt_f64("cpu", cpu));
        line.push_str(&json_opt_f64("ram", ram));
        line.push_str(&json_opt_str("reason", reason));
        if line.ends_with(',') {
            line.pop();
        }
        line.push_str("}\n");
        writer.write_all(line.as_bytes()).map_err(|err| err.to_string())
    }
}

pub fn now_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn open_append(path: &Path) -> Result<BufWriter<File>, String> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| format!("failed to open {}: {}", path.display(), err))?;
    Ok(BufWriter::new(file))
}

fn file_has_content(path: &Path) -> bool {
    std::fs::metadata(path).map(|m| m.len() > 0).unwrap_or(false)
}

fn opt_u32(value: Option<u32>) -> String {
    value.map(|v| v.to_string()).unwrap_or_default()
}

fn opt_f64(value: Option<f64>) -> String {
    value
        .map(|v| format!("{:.2}", v))
        .unwrap_or_default()
}

fn opt_str(value: Option<&str>) -> String {
    value.unwrap_or_default().to_string()
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        let escaped = value.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        value.to_string()
    }
}

fn json_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn json_opt_str(key: &str, value: Option<&str>) -> String {
    match value {
        Some(v) => format!("\"{}\":\"{}\",", key, json_escape(v)),
        None => format!("\"{}\":null,", key),
    }
}

fn json_opt_u32(key: &str, value: Option<u32>) -> String {
    match value {
        Some(v) => format!("\"{}\":{},", key, v),
        None => format!("\"{}\":null,", key),
    }
}

fn json_opt_f64(key: &str, value: Option<f64>) -> String {
    match value {
        Some(v) => format!("\"{}\":{:.2},", key, v),
        None => format!("\"{}\":null,", key),
    }
}
