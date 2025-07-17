use std::{
    cell::RefCell,
    collections::VecDeque,
    io::{Error, Result},
    rc::Rc,
    usize,
};

use crate::{
    node::{AtPosition, Atom, Mininode, Node, SDIS},
    pos_id::{PathComponent, PosID},
};

struct TreedocIter {
    paths: VecDeque<PosID>,
}

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
    // For concurrency
    _unique_disambiguator: SDIS,
}

#[derive(Debug)]
pub enum Signal {
    Insert(InsertSignal),
    Delete(DeleteSignal),
}

#[derive(Debug)]
pub struct Treedoc {
    pub root: Option<Rc<RefCell<Node>>>,
    pub unique_disambiguator: SDIS,
    pub doc_length: usize,
}

impl Iterator for TreedocIter {
    type Item = PosID;
    fn next(&mut self) -> Option<Self::Item> {
        self.paths.pop_front()
    }
}

impl Treedoc {
    pub fn new(ch: char) -> Self {
        let root = Node::new_with_mini(ch, 1u64);
        Treedoc {
            root: Some(Rc::new(RefCell::new(root))),
            doc_length: 1,
            unique_disambiguator: 1u64,
        }
    }
    pub fn apply(&mut self, sig: Signal) -> Result<()> {
        match sig {
            Signal::Insert(op) => {
                if let Some((last, rest)) = op.pos_id.0.split_last() {
                    let vd: Vec<PathComponent> = rest.iter().cloned().collect();
                    match Self::traverse_node_at_pos_id(AtPosition::Major(self.root.clone()), &vd) {
                        AtPosition::Major(parent_node) => match (last.0, last.1) {
                            (_, Some(dis)) => {
                                // push a child node, do not assign
                                parent_node
                                    .unwrap()
                                    .borrow_mut()
                                    .add_mini(Mininode::new_with_atom(op.atom, dis));
                            }
                            (0, None) => {
                                let new_node =
                                    Node::new_with_mini(op.atom, op.unique_disambiguator);
                                parent_node.unwrap().borrow_mut().add_left(new_node);
                            }
                            (1, None) => {
                                let new_node =
                                    Node::new_with_mini(op.atom, op.unique_disambiguator);
                                parent_node.unwrap().borrow_mut().add_right(new_node);
                            }
                            _ => {
                                unreachable!()
                            }
                        },
                        AtPosition::Mini(parent_node) => match (last.0, last.1) {
                            (0, None) => {
                                let new_node =
                                    Node::new_with_mini(op.atom, op.unique_disambiguator);
                                parent_node.unwrap().borrow_mut().add_left(new_node);
                            }
                            (1, None) => {
                                let new_node =
                                    Node::new_with_mini(op.atom, op.unique_disambiguator);
                                parent_node.unwrap().borrow_mut().add_right(new_node);
                            }
                            _ => {
                                unreachable!()
                            }
                        },
                    }
                    self.doc_length += 1;
                    return Ok(());
                }
                Err(Error::from(std::io::ErrorKind::InvalidData))
            }
            Signal::Delete(op) => {
                if let AtPosition::Mini(Some(node)) = Self::traverse_node_at_pos_id(
                    AtPosition::Major(self.root.clone()),
                    &op.pos_id.0,
                ) {
                    let mut node_mut = node.borrow_mut();
                    if node_mut.left.is_some() || node_mut.right.is_some() {
                        node_mut.tombstone = true;
                        self.doc_length -= 1;
                        return Ok(());
                    }
                }
                if let Some((last, rest)) = op.pos_id.0.split_last() {
                    let vd: Vec<PathComponent> = rest.iter().cloned().collect();
                    if let AtPosition::Major(Some(node)) =
                        Self::traverse_node_at_pos_id(AtPosition::Major(self.root.clone()), &vd)
                    {
                        let node_mut = node.borrow_mut();
                        node_mut.remove_mini(last.1)?;
                        self.doc_length -= 1;
                        return Ok(());
                    }
                }
                return Err(Error::from(std::io::ErrorKind::InvalidInput));
            }
        }
    }

    // 0-index characters -> as supposed to indices pointing to "gaps" in the insertion
    pub fn delete(&self, pos: usize) -> Result<DeleteSignal> {
        let pos = if pos == 0 {
            pos
        } else if pos > self.doc_length {
            self.doc_length - 1
        } else {
            pos - 1
        };
        Ok(DeleteSignal {
            pos_id: self.find_path_to_char(pos).unwrap_or(PosID::new()),
            _unique_disambiguator: self.unique_disambiguator,
        })
    }

    pub fn insert(&mut self, pos: usize, ch: char) -> Result<InsertSignal> {
        if pos > self.doc_length {
            return Err(Error::from(std::io::ErrorKind::InvalidInput));
        }
        let prev = if pos == 0 {
            PosID::new()
        } else {
            self.find_path_to_char(pos - 1)
                .unwrap_or_else(PosID::new_empty_end)
        };
        let next = if pos == self.doc_length {
            PosID::new_empty_end()
        } else {
            self.find_path_to_char(pos)
                .unwrap_or_else(PosID::new_empty_end)
        };
        let new_pos_id = self.new_pos_id(&prev, &next);

        Ok(InsertSignal {
            atom: ch,
            pos_id: new_pos_id,
            unique_disambiguator: self.unique_disambiguator,
        })
    }

    fn find_path_to_char(&self, target_index: usize) -> Option<PosID> {
        self.iter().nth(target_index)
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
                if p.1 == f.1 && p.0 != f.0 && !p.1.is_empty() {
                    let mut p_prev = prev.clone();
                    p_prev
                        .0
                        .push(PathComponent(0, Some(self.unique_disambiguator)));
                    return p_prev;
                }
            }
        }
        let mut p_prev = prev.strip_to_major();
        let mut p_next = next.strip_to_major();
        p_prev.0.push(PathComponent(1, None));
        if p_prev.0 < p_next.0 {
            return p_prev;
        }
        drop(p_prev);
        p_next.0.push(PathComponent(0, None));
        return p_next;
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
                        _ => unreachable!(),
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
                        _ => unreachable!(),
                    };
                }
            }
        }
        ref_point
    }

    fn iterate_pos_id(
        node: &Option<Rc<RefCell<Node>>>,
        curr_path: &mut PosID,
        iterated_vec: &mut VecDeque<PosID>,
    ) {
        if let Some(node) = node {
            curr_path.0.push(PathComponent(0, None));
            Self::iterate_pos_id(&node.borrow().left, curr_path, iterated_vec);
            curr_path.0.pop();
            for mininode in node.borrow().children.borrow().iter() {
                curr_path
                    .0
                    .push(PathComponent(0, Some(mininode.borrow().disambiguator)));
                curr_path.0.push(PathComponent(0, None));
                Self::iterate_pos_id(&mininode.borrow().left, curr_path, iterated_vec);
                curr_path.0.pop();
                curr_path.0.pop();
                if !mininode.borrow().tombstone {
                    let mut final_path = curr_path.clone();
                    final_path
                        .0
                        .push(PathComponent(0, Some(mininode.borrow().disambiguator)));
                    iterated_vec.push_back(final_path);
                }
                curr_path
                    .0
                    .push(PathComponent(0, Some(mininode.borrow().disambiguator)));
                curr_path.0.push(PathComponent(1, None));
                Self::iterate_pos_id(&mininode.borrow().right, curr_path, iterated_vec);
                curr_path.0.pop();
                curr_path.0.pop();
            }
            curr_path.0.push(PathComponent(1, None));
            Self::iterate_pos_id(&node.borrow().right, curr_path, iterated_vec);
            curr_path.0.pop();
        }
    }

    fn iter(&self) -> TreedocIter {
        let mut iterated_vec: VecDeque<PosID> = VecDeque::new();

        Self::iterate_pos_id(&self.root, &mut PosID::new(), &mut iterated_vec);
        TreedocIter {
            paths: iterated_vec,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_node_at_pos_id() {
        let mut td = Treedoc::new('a');
        let sig = td.insert(1, 'b');
        let res = td.apply(Signal::Insert(sig.unwrap()));
        assert!(res.is_ok());

        let mut pos_id_root = PosID::new();
        pos_id_root.0.push(PathComponent(1, None));
        pos_id_root
            .0
            .push(PathComponent(0, Some(td.unique_disambiguator)));
        let a_node =
            Treedoc::traverse_node_at_pos_id(AtPosition::Major(td.root.clone()), &pos_id_root.0);
        if let AtPosition::Mini(mn) = a_node {
            assert!(mn.is_some());
            assert!(mn.unwrap().borrow().atom == 'b');
        } else {
            panic!("Wrong node type iterated")
        }
    }
}
