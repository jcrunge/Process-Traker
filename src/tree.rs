use std::collections::HashMap;

use crate::platform::ProcessInfo;

pub struct ProcNode {
    pub info: ProcessInfo,
    pub children: Vec<u32>,
}

pub struct ProcTree {
    pub nodes: HashMap<u32, ProcNode>,
    pub roots: Vec<u32>,
}

impl ProcTree {
    pub fn from_processes(processes: Vec<ProcessInfo>) -> ProcTree {
        let mut nodes: HashMap<u32, ProcNode> = HashMap::new();
        for proc in processes {
            nodes.insert(
                proc.pid,
                ProcNode {
                    info: proc,
                    children: Vec::new(),
                },
            );
        }

        let mut roots = Vec::new();
        let pids: Vec<u32> = nodes.keys().copied().collect();
        for pid in pids {
            let ppid = nodes.get(&pid).map(|n| n.info.ppid).unwrap_or(0);
            if ppid != 0 && nodes.contains_key(&ppid) {
                if let Some(parent) = nodes.get_mut(&ppid) {
                    parent.children.push(pid);
                }
            } else {
                roots.push(pid);
            }
        }

        ProcTree { nodes, roots }
    }

    pub fn walk(&self) -> Vec<&ProcessInfo> {
        let mut out = Vec::new();
        for root in &self.roots {
            self.walk_from(*root, &mut out);
        }
        out
    }

    fn walk_from<'a>(&'a self, pid: u32, out: &mut Vec<&'a ProcessInfo>) {
        let Some(node) = self.nodes.get(&pid) else { return };
        out.push(&node.info);
        for child in &node.children {
            self.walk_from(*child, out);
        }
    }
}
