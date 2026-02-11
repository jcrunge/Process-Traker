use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Default)]
pub struct Allowlist {
    pub names: HashSet<String>,
    pub paths: HashSet<String>,
    pub hashes: HashSet<String>,
    pub uids: HashSet<u32>,
    pub ppids: HashSet<u32>,
    pub args: Vec<String>,
}

impl Allowlist {
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
            && self.paths.is_empty()
            && self.hashes.is_empty()
            && self.uids.is_empty()
            && self.ppids.is_empty()
            && self.args.is_empty()
    }
}

pub fn load_allowlist(path: &Path) -> Result<Allowlist, String> {
    let content = fs::read_to_string(path)
        .map_err(|err| format!("failed to read allowlist {}: {}", path.display(), err))?;

    let mut allowlist = Allowlist::default();
    for (idx, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| format!("invalid allowlist line {}: {}", idx + 1, line))?;
        let key = key.trim().to_lowercase();
        let value = value.trim();
        if value.is_empty() {
            return Err(format!("empty value on line {}", idx + 1));
        }

        match key.as_str() {
            "name" => {
                allowlist.names.insert(value.to_string());
            }
            "path" => {
                allowlist.paths.insert(value.to_string());
            }
            "hash" => {
                allowlist.hashes.insert(value.to_lowercase());
            }
            "uid" => {
                let uid = value
                    .parse::<u32>()
                    .map_err(|_| format!("invalid uid on line {}", idx + 1))?;
                allowlist.uids.insert(uid);
            }
            "ppid" => {
                let ppid = value
                    .parse::<u32>()
                    .map_err(|_| format!("invalid ppid on line {}", idx + 1))?;
                allowlist.ppids.insert(ppid);
            }
            "arg" => {
                allowlist.args.push(value.to_string());
            }
            _ => return Err(format!("unknown key on line {}: {}", idx + 1, key)),
        }
    }

    if allowlist.is_empty() {
        return Err("allowlist is empty".to_string());
    }

    Ok(allowlist)
}
