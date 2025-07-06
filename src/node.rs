use std::{cell::RefCell, rc::Rc};

pub type Atom = char;
pub type SDIS = u64;

// Most common structure is as follows:
// A node has one children, a mininode, that holds
// an atom (a character) and has a determinable PosID (infix walk distance)

// Major node
#[derive(Debug, Clone)]
pub struct Node {
    /// Must be kept sorted
    pub children: RefCell<Vec<Rc<Mininode>>>,
    pub left: Option<Box<Node>>,  // Rc<RefCell<Node>>
    pub right: Option<Box<Node>>, // Rc<RefCell<Node>>
}

#[derive(Debug, Clone)]
pub struct Mininode {
    pub disambiguator: SDIS, // SDIS
    pub atom: Atom,
    pub tombstone: bool,

    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}
