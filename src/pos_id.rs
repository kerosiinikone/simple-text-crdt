use std::{cmp::Ordering, usize};

use crate::node::SDIS;

// PathComponent(1, None) -> to major on the right
// PathComponent(0, None) -> to major on the left
// PathComponent(0, Some(dis)) -> get the mininode from within (distinct step)
#[derive(Debug, PartialEq, Clone)]
pub struct PathComponent(pub usize, pub Option<SDIS>);

#[derive(Debug, Clone)]
pub struct PosID(pub Vec<PathComponent>);

impl PosID {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn new_empty_end() -> Self {
        let mut pos = Self::new();
        pos.0.push(PathComponent(usize::MAX, None));
        pos
    }

    pub fn strip_to_major(&self) -> Self {
        let mut temp = self.clone();
        while let Some(last_component) = temp.0.last() {
            if last_component.1.is_some() {
                temp.0.pop();
            } else {
                break;
            }
        }
        temp
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
