use std::io::Result;

use crate::{
    node::{self, Mininode, Node, SDIS},
    pos_id::{PathComponent, PosID},
};

enum AtPosID {
    Node(Option<Box<Node>>),
    Mininode(Option<Box<Mininode>>),
}

// This is the document that is copied to all the peers
// -> aka the document state / atom buffer
#[derive(Debug)]
pub struct Treedoc {
    root: Option<Box<Node>>,
    unique_disambiguator: SDIS,
}

impl Treedoc {
    /*
        A major node is ordered by infix-order
        walk: the major node’s left child is before any mini-node;
        mini-nodes are ordered by disambiguator; and mini-nodes
        are before the major node’s right child.
    */
    fn traverse_in_and_collect(node: &Option<Box<Node>>, vec: &mut Vec<Mininode>) {
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

    fn get_by_pos_id(&self, pid: PosID) -> AtPosID {
        unimplemented!()
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

    // Public facing
    fn insert(&mut self, pos: usize, atom: node::Atom) -> Result<()> {
        Ok(())
    }

    // Public facing
    // Check and write helpers for the deletion caveats (tombstones / active deletion, inheritance)
    fn delete(&mut self, pos: usize) -> Result<()> {
        Ok(())
    }
}
