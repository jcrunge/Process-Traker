#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
use macos as platform_impl;
#[cfg(target_os = "linux")]
use linux as platform_impl;
#[cfg(target_os = "windows")]
use windows as platform_impl;

#[derive(Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub uid: u32,
    pub name: String,
    pub path: Option<String>,
    pub args: Vec<String>,
}

pub struct ProcSample {
    pub cpu_ns: u64,
    pub rss_bytes: u64,
}

pub fn list_processes() -> Result<Vec<ProcessInfo>, String> {
    platform_impl::list_processes()
}

pub fn sample_process(pid: u32) -> Result<ProcSample, String> {
    platform_impl::sample_process(pid)
}

pub fn num_cpus() -> Result<u32, String> {
    platform_impl::num_cpus()
}

pub fn total_mem_bytes() -> Result<u64, String> {
    platform_impl::total_mem_bytes()
}

pub fn kill_process(pid: u32) -> Result<(), String> {
    platform_impl::kill_process(pid)
}
