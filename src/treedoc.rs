use std::{io::Result, usize};

use crate::{
    node::{Atom, Mininode, Node, SDIS},
    pos_id::{PathComponent, PosID},
};

// Could also be implemented as a buffer on Treedoc?
// -> depends on the sync strat later
struct InsertSignal {
    atom: Atom,
    pos_id: PosID,
}

// This is the document that is copied to all the peers
// -> aka the document state / atom buffer
#[derive(Debug)]
pub struct Treedoc {
    pub root: Option<Box<Node>>,
    pub unique_disambiguator: SDIS,
    // Inc/dec on apply
    pub doc_length: usize,
}

impl Treedoc {
    // Public facing
    // Check and write helpers for the deletion caveats (tombstones / active deletion, inheritance)
    pub fn delete(&mut self, pos: usize) -> Result<()> {
        Ok(())
    }
    // Public facing
    pub fn insert(&mut self, pos: usize, ch: char) -> Result<InsertSignal> {
        // Somehow get the prev and next PosID from deisred _pos_
        // -> traverse and "build" a path until pos - 1 and pos + 1 (add left and right and diambig., etc)
        let mut prev_pos_id = PosID::new();
        let mut next_pos_id = PosID::new();

        let prev = if pos == 0 {
            PosID::new_empty_start()
        } else {
            Treedoc::find_path_at_index(&self.root, pos - 1, &mut 0, &mut prev_pos_id)
                .unwrap_or_else(PosID::new_empty_end)
        };
        let next = if pos == self.doc_length {
            PosID::new_empty_end()
        } else {
            Treedoc::find_path_at_index(&self.root, pos, &mut 0, &mut next_pos_id)
                .unwrap_or_else(PosID::new_empty_end)
        };
        // Get the *new* PosID
        let new_pos_id = self.new_pos_id(&prev, &next);
        // Return the operation signal with the PosID and Atom; Insert??
        Ok(InsertSignal {
            atom: ch,
            pos_id: new_pos_id,
        })
    }
    /*
        A major node is ordered by infix-order
        walk: the major node’s left child is before any mini-node;
        mini-nodes are ordered by disambiguator; and mini-nodes
        are before the major node’s right child.
    */
    pub fn traverse_in_and_collect(node: &Option<Box<Node>>, vec: &mut Vec<Mininode>) {
        if let Some(node) = node {
            Treedoc::traverse_in_and_collect(&node.left, vec);
            // Order mininodes by their disambiguators
            // Keep a sorted list vs. sort here? -> assume the children are sorted (u64)
            // Check the children and if they have left or/and right subtrees
            for mininode in node.children.borrow().iter() {
                Treedoc::traverse_in_and_collect(&mininode.left, vec);
                if !mininode.tombstone {
                    vec.push(mininode.as_ref().clone());
                }
                Treedoc::traverse_in_and_collect(&mininode.right, vec);
            }
            Treedoc::traverse_in_and_collect(&node.right, vec);
        }
    }
    /*
        When inserting between mini-siblings of a major node, a direct
        descendant of the mini-node is created. Otherwise, a child
        of a major node is created.
    */
    fn new_pos_id(&mut self, prev: &PosID, next: &PosID) -> PosID {
        if prev.0.len() < next.0.len() && next.0.starts_with(&prev.0) {
            let mut f_prev = next.clone();
            f_prev.0.pop();
            f_prev
                .0
                .push(PathComponent(0, Some(self.unique_disambiguator)));
            return f_prev;
        } else if next.0.len() < prev.0.len() && prev.0.starts_with(&next.0) {
            let mut p_prev = prev.clone();
            p_prev.0.pop();
            p_prev
                .0
                .push(PathComponent(1, Some(self.unique_disambiguator)));
            return p_prev;
        } else if let Some(p) = prev.0.split_last() {
            if let Some(f) = next.0.split_last() {
                if p.1 == f.1 && p.0 != f.0 {
                    let mut p_prev = prev.clone();
                    p_prev
                        .0
                        .push(PathComponent(1, Some(self.unique_disambiguator)));
                    return p_prev;
                }
            }
        }
        let mut p_prev = prev.clone();
        p_prev.0.pop();
        p_prev
            .0
            .push(PathComponent(1, Some(self.unique_disambiguator)));
        return p_prev;
    }

    fn find_path_at_index(
        node: &Option<Box<Node>>,
        target_index: usize,
        curr_index: &mut usize,
        curr_path: &mut PosID,
    ) -> Option<PosID> {
        if let Some(node) = node {
            curr_path.0.push(PathComponent(0, None));
            match Treedoc::find_path_at_index(&node.left, target_index, curr_index, curr_path) {
                Some(path) => return Some(path),
                None => {
                    curr_path.0.pop();
                }
            }
            for mininode in node.children.borrow().iter() {
                curr_path
                    .0
                    .push(PathComponent(0, Some(mininode.disambiguator)));
                curr_path.0.push(PathComponent(0, None));
                match Treedoc::find_path_at_index(
                    &mininode.left,
                    target_index,
                    curr_index,
                    curr_path,
                ) {
                    Some(path) => return Some(path),
                    None => {
                        curr_path.0.pop();
                        curr_path.0.pop();
                    }
                }
                // Mininode itself
                if !mininode.tombstone {
                    if *curr_index == target_index {
                        curr_path
                            .0
                            .push(PathComponent(0, Some(mininode.disambiguator)));
                        return Some(curr_path.to_owned());
                    }
                    *curr_index += 1;
                }
                curr_path
                    .0
                    .push(PathComponent(0, Some(mininode.disambiguator))); // Current mini
                curr_path.0.push(PathComponent(1, None)); // Turn to major
                match Treedoc::find_path_at_index(
                    &mininode.right,
                    target_index,
                    curr_index,
                    curr_path,
                ) {
                    Some(path) => return Some(path),
                    None => {
                        curr_path.0.pop();
                        curr_path.0.pop();
                    }
                }
            }
            curr_path.0.push(PathComponent(1, None));
            match Treedoc::find_path_at_index(&node.right, target_index, curr_index, curr_path) {
                Some(path) => return Some(path),
                None => {
                    curr_path.0.pop();
                }
            }
        }
        None
    }
}
