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

#[derive(Debug, PartialEq, PartialOrd)]
struct PathComponent(usize, Option<SDIS>);

#[derive(Debug)]
pub struct PosID(Vec<PathComponent>);

impl PartialOrd for PosID {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        unimplemented!()
    }
}

impl PartialEq for PosID {
    fn eq(&self, other: &Self) -> bool {
        for (idx, turn) in self.0.iter().enumerate() {
            let pos_id_oth = &other.0[idx];
            if turn.1 != pos_id_oth.1 || turn.0 != pos_id_oth.0 {
                return false;
            }
        }
        true
    }
}

fn newPosID(prev: PosID, next: PosID) -> PosID {
    unimplemented!()
}
