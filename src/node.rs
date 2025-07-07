use std::{cell::RefCell, rc::Rc};

pub type Atom = char;

// SDIS only on mininodes -> this is why PathComponents with
// disambiguators are mininodes and ones without are major nodes
pub type SDIS = u64;

// Major node
#[derive(Debug, Clone)]
pub struct Node {
    /// Must be kept sorted
    pub children: RefCell<Vec<Rc<Mininode>>>,
    pub left: Option<Box<Node>>,  // Rc<RefCell<Node>>?
    pub right: Option<Box<Node>>, // Rc<RefCell<Node>>?
}

#[derive(Debug, Clone)]
pub struct Mininode {
    pub disambiguator: SDIS, // SDIS
    pub atom: Atom,
    pub tombstone: bool,

    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
}
