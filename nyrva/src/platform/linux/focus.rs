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
