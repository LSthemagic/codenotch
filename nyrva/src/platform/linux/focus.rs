use std::collections::HashMap;

pub struct ProcMaps {
    pub ppid: HashMap<u32, u32>,
    pub name: HashMap<u32, String>,
}

pub fn proc_maps() -> ProcMaps {
    ProcMaps {
        ppid: HashMap::new(),
        name: HashMap::new(),
    }
}

pub fn fg_pid() -> u32 {
    0
}

pub fn chain_of(pid: u32, ppid: &HashMap<u32, u32>) -> Vec<u32> {
    let mut chain = vec![pid];
    let mut cur = pid;
    for _ in 0..8 {
        match ppid.get(&cur) {
            Some(&p) if p != 0 && !chain.contains(&p) => {
                chain.push(p);
                cur = p;
            }
            _ => break,
        }
    }
    chain
}

pub fn pid_hits_chain(pid: u32, chain: &[u32], maps: &ProcMaps) -> bool {
    chain.contains(&pid)
        || maps
            .ppid
            .get(&pid)
            .map(|p| chain.contains(p))
            .unwrap_or(false)
}

pub fn focus_terminal(_pid: u32) -> bool {
    false
}

pub fn focus_claude_desktop() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::{chain_of, pid_hits_chain, ProcMaps};
    use std::collections::HashMap;

    #[test]
    fn chain_of_walks_parents_and_stops_at_zero() {
        let ppid = HashMap::from([(40, 30), (30, 20), (20, 0)]);
        assert_eq!(chain_of(40, &ppid), vec![40, 30, 20]);
    }

    #[test]
    fn chain_of_stops_on_cycles() {
        let ppid = HashMap::from([(40, 30), (30, 40)]);
        assert_eq!(chain_of(40, &ppid), vec![40, 30]);
    }

    #[test]
    fn pid_hits_chain_accepts_direct_and_parent_matches() {
        let maps = ProcMaps {
            ppid: HashMap::from([(99, 30)]),
            name: HashMap::new(),
        };
        let chain = vec![40, 30, 20];
        assert!(pid_hits_chain(40, &chain, &maps));
        assert!(pid_hits_chain(99, &chain, &maps));
        assert!(!pid_hits_chain(100, &chain, &maps));
    }
}
