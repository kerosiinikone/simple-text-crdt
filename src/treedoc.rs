use std::{
    cell::RefCell,
    io::{Error, Result},
    rc::Rc,
    usize,
};

use crate::{
    node::{AtPosition, Atom, Mininode, Node, SDIS},
    pos_id::{PathComponent, PosID},
};

// Could also be implemented as a buffer on Treedoc??
// -> depends on the sync strat later
#[derive(Debug)]
pub struct InsertSignal {
    atom: Atom,
    pos_id: PosID,
    unique_disambiguator: SDIS,
}

#[derive(Debug)]
pub struct DeleteSignal {
    pos_id: PosID,
    unique_disambiguator: SDIS,
}

// -> for applying locally and syncing
#[derive(Debug)]
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
                    match Self::traverse_node_at_pos_id(AtPosition::Major(self.root.clone()), &vd) {
                        AtPosition::Major(parent_node) => {
                            match (last.0, last.1) {
                                (_, Some(dis)) => {
                                    let parent_mut = parent_node.clone().unwrap();
                                    parent_mut.borrow().children.borrow_mut().push(Rc::new(
                                        RefCell::new(Mininode {
                                            atom: op.atom,
                                            disambiguator: dis,
                                            left: None,
                                            right: None,
                                            tombstone: false,
                                        }),
                                    ));
                                    parent_mut
                                        .borrow()
                                        .children
                                        .borrow_mut()
                                        .sort_by_key(|m| m.borrow().disambiguator);
                                }
                                (0, None) => {
                                    let new_node =
                                        Node::new_with_mini(op.atom, op.unique_disambiguator);
                                    parent_node.unwrap().borrow_mut().left =
                                        Some(Rc::new(RefCell::new(new_node)))
                                }
                                (1, None) => {
                                    let new_node =
                                        Node::new_with_mini(op.atom, op.unique_disambiguator);
                                    parent_node.unwrap().borrow_mut().right =
                                        Some(Rc::new(RefCell::new(new_node)))
                                }
                                _ => {
                                    unimplemented!() // Shouldn't happen
                                }
                            }
                        }
                        AtPosition::Mini(parent_node) => {
                            match (last.0, last.1) {
                                (0, None) => {
                                    let parent_mut = parent_node.clone().unwrap();
                                    let new_node =
                                        Node::new_with_mini(op.atom, op.unique_disambiguator);
                                    parent_mut.borrow_mut().left =
                                        Some(Rc::new(RefCell::new(new_node)))
                                }
                                (1, None) => {
                                    let new_node =
                                        Node::new_with_mini(op.atom, op.unique_disambiguator);
                                    parent_node.unwrap().borrow_mut().right =
                                        Some(Rc::new(RefCell::new(new_node)))
                                }
                                _ => {
                                    unimplemented!() // Shouldn't happen
                                }
                            }
                        }
                    }
                    self.doc_length += 1;
                    return Ok(());
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
    pub fn traverse_in_and_collect(node: &Option<Rc<RefCell<Node>>>, vec: &mut Vec<Atom>) {
        if let Some(node) = node {
            Self::traverse_in_and_collect(&node.borrow().left, vec);
            // Order mininodes by their disambiguators
            // Keep a sorted list vs. sort here? -> assume the children are sorted (u64)
            // Check the children and if they have left or/and right subtrees
            for mininode in node.borrow().children.borrow().iter() {
                Self::traverse_in_and_collect(&mininode.borrow().left, vec);
                if !mininode.borrow().tombstone {
                    vec.push(mininode.borrow().atom);
                }
                Self::traverse_in_and_collect(&mininode.borrow().right, vec);
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
            f_prev.0.push(PathComponent(0, None));
            return f_prev;
        } else if next.0.len() < prev.0.len() && prev.0.starts_with(&next.0) {
            let mut p_prev = prev.clone();
            p_prev.0.pop();
            p_prev.0.push(PathComponent(1, None));
            return p_prev;
        } else if let Some(p) = prev.0.split_last() {
            if let Some(f) = next.0.split_last() {
                // Minisiblings
                if p.1 == f.1 && p.0 != f.0 {
                    let mut p_prev = prev.clone();
                    p_prev.0.push(PathComponent(1, None));
                    return p_prev;
                }
            }
        }
        let mut p_prev = prev.clone();
        p_prev.0.pop();
        p_prev.0.push(PathComponent(1, None));
        return p_prev;
    }

    fn traverse_node_at_pos_id(node: AtPosition, curr_pos_id: &Vec<PathComponent>) -> AtPosition {
        let mut ref_point = node.clone();
        for path_comp in curr_pos_id {
            match ref_point {
                AtPosition::Major(major) => {
                    match (path_comp.0, path_comp.1) {
                        (0, None) => {
                            ref_point = AtPosition::Major(major.unwrap().borrow().left.clone())
                        }
                        (1, None) => {
                            ref_point = AtPosition::Major(major.unwrap().borrow().right.clone())
                        }
                        (0, Some(dis)) => {
                            ref_point = AtPosition::Mini(
                                major
                                    .unwrap()
                                    .borrow()
                                    .children
                                    .borrow()
                                    .iter()
                                    .find(|mn| mn.borrow().disambiguator == dis)
                                    .cloned(),
                            );
                        }
                        _ => unimplemented!(),
                    };
                }
                AtPosition::Mini(mini) => {
                    match (path_comp.0, path_comp.1) {
                        (0, None) => {
                            ref_point = AtPosition::Major(mini.unwrap().borrow().left.clone())
                        }
                        (1, None) => {
                            ref_point = AtPosition::Major(mini.unwrap().borrow().right.clone())
                        }
                        _ => unimplemented!(), // Impossible
                    };
                }
            }
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
                    .push(PathComponent(0, Some(mininode.borrow().disambiguator)));
                curr_path.0.push(PathComponent(0, None));
                match Self::find_path_at_index(
                    &mininode.borrow().left,
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
                // To the mininode itself -> PathComponent(0, Some(dis))
                if !mininode.borrow().tombstone {
                    if *curr_index == target_index {
                        curr_path
                            .0
                            .push(PathComponent(0, Some(mininode.borrow().disambiguator)));
                        return Some(curr_path.to_owned());
                    }
                    *curr_index += 1;
                }
                curr_path
                    .0
                    .push(PathComponent(0, Some(mininode.borrow().disambiguator))); // Current mini
                curr_path.0.push(PathComponent(1, None)); // Turn to major
                match Self::find_path_at_index(
                    &mininode.borrow().right,
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
