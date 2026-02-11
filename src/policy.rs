use std::collections::HashMap;
use std::fs;
use std::os::raw::c_void;

use crate::config::Allowlist;
use crate::platform::ProcessInfo;

pub fn is_allowed(
    info: &ProcessInfo,
    allowlist: &Allowlist,
    hash_cache: &mut HashMap<String, String>,
) -> bool {
    if allowlist.names.contains(&info.name) {
        return true;
    }

    if let Some(path) = &info.path {
        if allowlist.paths.contains(path) {
            return true;
        }
    }

    if allowlist.uids.contains(&info.uid) {
        return true;
    }

    if allowlist.ppids.contains(&info.ppid) {
        return true;
    }

    if !allowlist.args.is_empty() && !info.args.is_empty() {
        let joined = info.args.join(" ");
        for arg in &allowlist.args {
            if joined.contains(arg) {
                return true;
            }
        }
    }

    if !allowlist.hashes.is_empty() {
        if let Some(path) = &info.path {
            let entry = hash_cache.entry(path.clone()).or_insert_with(|| {
                compute_sha256_hex(path).unwrap_or_default()
            });
            if !entry.is_empty() && allowlist.hashes.contains(&entry.to_lowercase()) {
                return true;
            }
        }
    }

    false
}

#[cfg(target_os = "macos")]
extern "C" {
    fn CC_SHA256(data: *const c_void, len: u32, md: *mut u8) -> *mut u8;
}

#[cfg(target_os = "macos")]
fn compute_sha256_hex(path: &str) -> Result<String, String> {
    let data = fs::read(path).map_err(|err| format!("hash read failed: {}", err))?;
    let mut digest = [0u8; 32];
    unsafe {
        CC_SHA256(data.as_ptr() as *const c_void, data.len() as u32, digest.as_mut_ptr());
    }
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{:02x}", byte));
    }
    Ok(out)
}

#[cfg(not(target_os = "macos"))]
fn compute_sha256_hex(_path: &str) -> Result<String, String> {
    Err("hashing not supported on this platform".to_string())
}
