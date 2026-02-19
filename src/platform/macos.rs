use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::ptr;

use super::{ProcSample, ProcessInfo};

const PROC_ALL_PIDS: c_uint = 1;
const PROC_PIDTBSDINFO: c_int = 3;
const RUSAGE_INFO_V2: c_int = 2;
const CTL_KERN: c_int = 1;
const KERN_PROCARGS2: c_int = 49;
const SIGKILL: c_int = 9;

#[repr(C)]
struct ProcBsdInfo {
    pbi_flags: u32,
    pbi_status: u32,
    pbi_xstatus: u32,
    pbi_pid: u32,
    pbi_ppid: u32,
    pbi_uid: u32,
    pbi_gid: u32,
    pbi_ruid: u32,
    pbi_rgid: u32,
    pbi_svuid: u32,
    pbi_svgid: u32,
    rfu_1: u32,
    pbi_comm: [u8; 17],
    pbi_name: [u8; 33],
    pbi_nfiles: u32,
    pbi_pgid: u32,
    pbi_pjobc: u32,
    e_tdev: u32,
    e_tpgid: u32,
    pbi_nice: i32,
    pbi_start_tvsec: u64,
    pbi_start_tvusec: u64,
}

#[repr(C)]
struct RusageInfoV2 {
    ri_uuid: [u8; 16],
    ri_user_time: u64,
    ri_system_time: u64,
    ri_pkg_idle_wkups: u64,
    ri_interrupt_wkups: u64,
    ri_pageins: u64,
    ri_wired_size: u64,
    ri_resident_size: u64,
    ri_phys_footprint: u64,
    ri_proc_start_abstime: u64,
    ri_proc_exit_abstime: u64,
    ri_child_user_time: u64,
    ri_child_system_time: u64,
    ri_child_pkg_idle_wkups: u64,
    ri_child_interrupt_wkups: u64,
    ri_child_pageins: u64,
    ri_child_elapsed_abstime: u64,
    ri_diskio_bytesread: u64,
    ri_diskio_byteswritten: u64,
}

extern "C" {
    fn proc_listpids(
        type_: c_uint,
        typeinfo: c_uint,
        buffer: *mut c_void,
        buffersize: c_int,
    ) -> c_int;
    fn proc_pidinfo(
        pid: c_int,
        flavor: c_int,
        arg: u64,
        buffer: *mut c_void,
        buffersize: c_int,
    ) -> c_int;
    fn proc_pidpath(pid: c_int, buffer: *mut c_void, buffersize: u32) -> c_int;
    fn proc_name(pid: c_int, buffer: *mut c_void, buffersize: u32) -> c_int;
    fn proc_pid_rusage(pid: c_int, flavor: c_int, buffer: *mut c_void) -> c_int;
    fn sysctl(
        mib: *mut c_int,
        miblen: u32,
        oldp: *mut c_void,
        oldlenp: *mut usize,
        newp: *mut c_void,
        newlen: usize,
    ) -> c_int;
    fn sysctlbyname(
        name: *const c_char,
        oldp: *mut c_void,
        oldlenp: *mut usize,
        newp: *mut c_void,
        newlen: usize,
    ) -> c_int;
    fn kill(pid: c_int, sig: c_int) -> c_int;
}

pub fn list_processes() -> Result<Vec<ProcessInfo>, String> {
    let mut count = 1024usize;
    let pids = loop {
        let mut buf = vec![0i32; count];
        let bytes = unsafe {
            proc_listpids(
                PROC_ALL_PIDS,
                0,
                buf.as_mut_ptr() as *mut c_void,
                (buf.len() * mem::size_of::<i32>()) as c_int,
            )
        };
        if bytes <= 0 {
            return Err("proc_listpids failed".to_string());
        }
        let used = (bytes as usize) / mem::size_of::<i32>();
        if used < buf.len() {
            buf.truncate(used);
            break buf;
        }
        count *= 2;
    };

    let mut processes = Vec::new();
    for pid in pids {
        if pid <= 0 {
            continue;
        }

        let mut name_buf = [0u8; 64];
        let name_len = unsafe { proc_name(pid, name_buf.as_mut_ptr() as *mut c_void, name_buf.len() as u32) };
        if name_len <= 0 {
            continue;
        }
        let name = unsafe {
            CStr::from_ptr(name_buf.as_ptr() as *const c_char)
                .to_string_lossy()
                .into_owned()
        };

        let mut bsdinfo: ProcBsdInfo = unsafe { mem::zeroed() };
        let ret = unsafe {
            proc_pidinfo(
                pid,
                PROC_PIDTBSDINFO,
                0,
                &mut bsdinfo as *mut _ as *mut c_void,
                mem::size_of::<ProcBsdInfo>() as c_int,
            )
        };
        if ret <= 0 {
            continue;
        }

        let mut path_buf = [0u8; 1024];
        let path_len = unsafe { proc_pidpath(pid, path_buf.as_mut_ptr() as *mut c_void, path_buf.len() as u32) };
        let path = if path_len > 0 {
            Some(
                unsafe { CStr::from_ptr(path_buf.as_ptr() as *const c_char) }
                    .to_string_lossy()
                    .into_owned(),
            )
        } else {
            None
        };

        let args = get_args(pid);

        processes.push(ProcessInfo {
            pid: pid as u32,
            ppid: bsdinfo.pbi_ppid,
            uid: bsdinfo.pbi_uid,
            name,
            path,
            args,
        });
    }

    Ok(processes)
}

pub fn sample_process(pid: u32) -> Result<ProcSample, String> {
    let mut info: RusageInfoV2 = unsafe { mem::zeroed() };
    let ret = unsafe { proc_pid_rusage(pid as c_int, RUSAGE_INFO_V2, &mut info as *mut _ as *mut c_void) };
    if ret != 0 {
        return Err("proc_pid_rusage failed".to_string());
    }
    Ok(ProcSample {
        cpu_ns: info.ri_user_time.saturating_add(info.ri_system_time),
        rss_bytes: info.ri_resident_size,
    })
}

pub fn num_cpus() -> Result<u32, String> {
    let mut value: u32 = 0;
    let mut size = mem::size_of::<u32>();
    let name = CString::new("hw.ncpu").map_err(|_| "bad sysctl name")?;
    let ret = unsafe {
        sysctlbyname(
            name.as_ptr(),
            &mut value as *mut _ as *mut c_void,
            &mut size,
            ptr::null_mut(),
            0,
        )
    };
    if ret != 0 {
        return Err("sysctl hw.ncpu failed".to_string());
    }
    Ok(value)
}

pub fn total_mem_bytes() -> Result<u64, String> {
    let mut value: u64 = 0;
    let mut size = mem::size_of::<u64>();
    let name = CString::new("hw.memsize").map_err(|_| "bad sysctl name")?;
    let ret = unsafe {
        sysctlbyname(
            name.as_ptr(),
            &mut value as *mut _ as *mut c_void,
            &mut size,
            ptr::null_mut(),
            0,
        )
    };
    if ret != 0 {
        return Err("sysctl hw.memsize failed".to_string());
    }
    Ok(value)
}

pub fn kill_process(pid: u32) -> Result<(), String> {
    let ret = unsafe { kill(pid as c_int, SIGKILL) };
    if ret != 0 {
        return Err("kill failed".to_string());
    }
    Ok(())
}

fn get_args(pid: i32) -> Vec<String> {
    let mut mib = [CTL_KERN, KERN_PROCARGS2, pid];
    let mut size: usize = 0;
    let ret = unsafe {
        sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            ptr::null_mut(),
            &mut size,
            ptr::null_mut(),
            0,
        )
    };
    if ret != 0 || size == 0 {
        return Vec::new();
    }

    let mut buf = vec![0u8; size];
    let ret = unsafe {
        sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            buf.as_mut_ptr() as *mut c_void,
            &mut size,
            ptr::null_mut(),
            0,
        )
    };
    if ret != 0 || size < mem::size_of::<u32>() {
        return Vec::new();
    }

    let argc = unsafe { *(buf.as_ptr() as *const u32) } as usize;
    let mut idx = mem::size_of::<u32>();

    while idx < size && buf[idx] != 0 {
        idx += 1;
    }
    while idx < size && buf[idx] == 0 {
        idx += 1;
    }

    let mut args = Vec::new();
    for _ in 0..argc {
        if idx >= size {
            break;
        }
        let start = idx;
        while idx < size && buf[idx] != 0 {
            idx += 1;
        }
        if idx > start {
            if let Ok(s) = std::str::from_utf8(&buf[start..idx]) {
                args.push(s.to_string());
            }
        }
        while idx < size && buf[idx] == 0 {
            idx += 1;
        }
    }

    args
}

const SYSTEM_PREFIXES: &[&str] = &[
    "/System/",
    "/usr/libexec/",
    "/usr/sbin/",
    "/usr/bin/",
    "/sbin/",
    "/Library/Apple/",
];

pub fn is_system_process(info: &ProcessInfo) -> bool {
    match &info.path {
        Some(path) => SYSTEM_PREFIXES.iter().any(|p| path.starts_with(p)),
        None => false,
    }
}
