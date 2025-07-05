use std::{cell::RefCell, rc::Rc};

type Atom = char; 

// Major node
#[derive(Debug)]
pub struct Node {
    pub children: RefCell<Vec<Rc<Mininode>>>, // ??
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>
}

#[derive(Debug)] // PartialOrd
pub struct Mininode {
    disambiguator: u64, // SDIS
    atom: Atom,
    tombstone: bool,

    left: Option<Box<Node>>,
    right: Option<Box<Node>>
}

