use std::{
    cell::RefCell,
    collections::VecDeque,
    io::{Error, Result},
    rc::Rc,
    usize,
};

use crate::{
    node::{Atom, Mininode, Node, SDIS},
    pos_id::{PathComponent, PosID},
};

// Could also be implemented as a buffer on Treedoc??
// -> depends on the sync strat later
struct InsertSignal {
    atom: Atom,
    pos_id: PosID,
    unique_disambiguator: SDIS,
}

struct DeleteSignal {
    pos_id: PosID,
    unique_disambiguator: SDIS,
}

// -> for applying locally and syncing
pub enum Signal {
    Insert(InsertSignal),
    Delete(DeleteSignal),
}

// This is the document that is copied to all the peers
// -> aka the document state / atom buffer
#[derive(Debug)]
pub struct Treedoc {
    pub root: Option<Rc<RefCell<Node>>>,
    pub unique_disambiguator: SDIS,
    // Inc/dec on apply -> O(1)
    pub doc_length: usize,
}

impl Treedoc {
    pub fn apply(&mut self, sig: Signal) -> Result<()> {
        match sig {
            Signal::Insert(op) => {
                if let Some((last, rest)) = op.pos_id.0.split_last() {
                    let vd: Vec<PathComponent> = rest.iter().cloned().collect();
                    if let Some(parent_node) = Self::traverse_node_at_pos_id(&mut self.root, &vd) {
                        match (last.0, last.1) {
                            (_, Some(dis)) => {
                                parent_node.borrow().children.borrow_mut().push(Rc::new(
                                    Mininode {
                                        atom: op.atom,
                                        disambiguator: dis,
                                        left: None,
                                        right: None,
                                        tombstone: false,
                                    },
                                ));
                                parent_node
                                    .borrow()
                                    .children
                                    .borrow_mut()
                                    .sort_by_key(|m| m.disambiguator);
                            }
                            (0, None) => {
                                let new_node =
                                    Node::new_with_mini(op.atom, op.unique_disambiguator);
                                parent_node.borrow_mut().left =
                                    Some(Rc::new(RefCell::new(new_node)))
                            }
                            (1, None) => {
                                let new_node =
                                    Node::new_with_mini(op.atom, op.unique_disambiguator);
                                parent_node.borrow_mut().right =
                                    Some(Rc::new(RefCell::new(new_node)))
                            }
                            (_, None) => {
                                unimplemented!() // Shouldn't happen
                            }
                        }
                        self.doc_length += 1;
                        return Ok(());
                    }
                }
                // pos_id epmty?
                Err(Error::from(std::io::ErrorKind::InvalidData))
            }
            Signal::Delete(_) => Ok(()),
        }
    }

    pub fn delete(&mut self, pos: usize) -> Result<DeleteSignal> {
        Ok(DeleteSignal {
            pos_id: Self::find_path_at_index(&self.root, pos, &mut 0, &mut PosID::new())
                .unwrap_or_else(PosID::new_empty_end),
            unique_disambiguator: self.unique_disambiguator,
        })
    }

    pub fn insert(&mut self, pos: usize, ch: char) -> Result<InsertSignal> {
        let mut prev_pos_id = PosID::new();
        let mut next_pos_id = PosID::new();

        let prev = if pos == 0 {
            PosID::new_empty_start()
        } else {
            Self::find_path_at_index(&self.root, pos - 1, &mut 0, &mut prev_pos_id)
                .unwrap_or_else(PosID::new_empty_end)
        };
        let next = if pos == self.doc_length {
            PosID::new_empty_end()
        } else {
            Self::find_path_at_index(&self.root, pos, &mut 0, &mut next_pos_id)
                .unwrap_or_else(PosID::new_empty_end)
        };
        let new_pos_id = self.new_pos_id(&prev, &next);

        Ok(InsertSignal {
            atom: ch,
            pos_id: new_pos_id,
            unique_disambiguator: self.unique_disambiguator,
        })
    }
    /*
    "A major node is ordered by infix-order
    walk: the major node’s left child is before any mini-node;
    mini-nodes are ordered by disambiguator; and mini-nodes
    are before the major node’s right child."
    */
    pub fn traverse_in_and_collect(node: &Option<Rc<RefCell<Node>>>, vec: &mut Vec<Mininode>) {
        if let Some(node) = node {
            Self::traverse_in_and_collect(&node.borrow().left, vec);
            // Order mininodes by their disambiguators
            // Keep a sorted list vs. sort here? -> assume the children are sorted (u64)
            // Check the children and if they have left or/and right subtrees
            for mininode in node.borrow().children.borrow().iter() {
                Self::traverse_in_and_collect(&mininode.left, vec);
                if !mininode.tombstone {
                    vec.push(mininode.as_ref().clone());
                }
                Self::traverse_in_and_collect(&mininode.right, vec);
            }
            Self::traverse_in_and_collect(&node.borrow().right, vec);
        }
    }
    /*
    "When inserting between mini-siblings of a major node, a direct
    descendant of the mini-node is created. Otherwise, a child
    of a major node is created."
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

    fn traverse_node_at_pos_id(
        node: &Option<Rc<RefCell<Node>>>,
        curr_pos_id: &Vec<PathComponent>,
    ) -> Option<Rc<RefCell<Node>>> {
        let mut ref_point = node.clone();
        for path_comp in curr_pos_id {
            match (path_comp.0, path_comp.1) {
                (0, None) => ref_point = ref_point.unwrap().borrow().left.clone(),
                (1, None) => ref_point = ref_point.unwrap().borrow().right.clone(),
                (t, Some(dis)) => {
                    let target = ref_point
                        .unwrap()
                        .borrow()
                        .children
                        .borrow()
                        .iter()
                        .find(|mn| mn.disambiguator == dis)
                        .cloned();
                    if t == 0 {
                        ref_point = target.unwrap().left.clone();
                    } else {
                        ref_point = target.unwrap().right.clone();
                    }
                }
                _ => return None,
            };
        }
        ref_point
    }

    fn find_path_at_index(
        node: &Option<Rc<RefCell<Node>>>,
        target_index: usize,
        curr_index: &mut usize,
        curr_path: &mut PosID,
    ) -> Option<PosID> {
        if let Some(node) = node {
            curr_path.0.push(PathComponent(0, None));
            match Self::find_path_at_index(&node.borrow().left, target_index, curr_index, curr_path)
            {
                Some(path) => return Some(path),
                None => {
                    curr_path.0.pop();
                }
            }
            for mininode in node.borrow().children.borrow().iter() {
                curr_path
                    .0
                    .push(PathComponent(0, Some(mininode.disambiguator)));
                curr_path.0.push(PathComponent(0, None));
                match Self::find_path_at_index(&mininode.left, target_index, curr_index, curr_path)
                {
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
                match Self::find_path_at_index(&mininode.right, target_index, curr_index, curr_path)
                {
                    Some(path) => return Some(path),
                    None => {
                        curr_path.0.pop();
                        curr_path.0.pop();
                    }
                }
            }
            curr_path.0.push(PathComponent(1, None));
            match Self::find_path_at_index(
                &node.borrow().right,
                target_index,
                curr_index,
                curr_path,
            ) {
                Some(path) => return Some(path),
                None => {
                    curr_path.0.pop();
                }
            }
        }
        None
    }
}
