use super::{ProcSample, ProcessInfo};

pub fn list_processes() -> Result<Vec<ProcessInfo>, String> {
    Err("linux not implemented".to_string())
}

pub fn sample_process(_pid: u32) -> Result<ProcSample, String> {
    Err("linux not implemented".to_string())
}

pub fn num_cpus() -> Result<u32, String> {
    Err("linux not implemented".to_string())
}

pub fn total_mem_bytes() -> Result<u64, String> {
    Err("linux not implemented".to_string())
}

pub fn kill_process(_pid: u32) -> Result<(), String> {
    Err("linux not implemented".to_string())
}
