use std::io::Result;

use crate::{
    node::{self, Mininode, Node},
    pos_id::PosID,
};

// This is the document that is copied to all the peers
// -> aka the document state / atom buffer

/// Essentially a binary tree (BT)
#[derive(Debug)]
pub struct Treedoc {
    root: Option<Box<Node>>,
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

    fn find_by_pos_id(&self, pid: PosID) -> Option<Box<Mininode>> {
        unimplemented!()
    }

    // Public facing
    fn insert(&mut self, pos: usize, atom: node::Atom) -> Result<()> {
        Ok(())
    }

    // Public facing
    fn delete(&mut self, pos: usize) -> Result<()> {
        Ok(())
    }
}
