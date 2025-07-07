use std::{cmp::Ordering, usize};

use crate::node::SDIS;

// Keeping track of the SDIS on each "turn" also helps determining which mininode
// under a major node has to be selected -> now the mininodes don't have to have
// internal turns inside a major node

/*
    Paths (PosIDs) include a disambiguator only when necessary, i.e., (i) at the last element of the path; or (ii) whenever
    the path follows a child of a mini-node explicitly. A path
    element without a disambiguator refers to the children of
    the corresponding major node.
*/

#[derive(Debug, PartialEq, Clone)]
pub struct PathComponent(pub usize, pub Option<SDIS>);

#[derive(Debug, Clone)]
pub struct PosID(pub Vec<PathComponent>);

impl PosID {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn new_empty_start() -> Self {
        let mut pos = PosID::new();
        pos.0.push(PathComponent(0, None));
        pos
    }

    pub fn new_empty_end() -> Self {
        let mut pos = PosID::new();
        pos.0.push(PathComponent(usize::MAX, Some(u64::MAX)));
        pos
    }
}

impl PartialOrd for PathComponent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (PathComponent(a, None), PathComponent(b, None)) => a.partial_cmp(b),
            (PathComponent(a, Some(dis_a)), PathComponent(b, Some(dis_b))) => a
                .partial_cmp(b)
                .filter(|o| o.is_ne())
                .or_else(|| dis_a.partial_cmp(dis_b)),
            (PathComponent(0, None), PathComponent(_, Some(_))) => Some(Ordering::Less),
            (PathComponent(_, Some(_)), PathComponent(0, None)) => Some(Ordering::Greater),
            (PathComponent(_, Some(_)), PathComponent(1, None)) => Some(Ordering::Less),
            (PathComponent(1, None), PathComponent(_, Some(_))) => Some(Ordering::Greater),
            (PathComponent(a, _), PathComponent(b, _)) => a.partial_cmp(b), // Def
        }
    }
}

impl PartialOrd for PosID {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl PartialEq for PosID {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
