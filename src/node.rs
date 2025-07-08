use std::{cell::RefCell, rc::Rc};

pub type Atom = char;

// SDIS only on mininodes -> this is why PathComponents with
// disambiguators are mininodes and ones without are major nodes
pub type SDIS = u64;

// Major node
#[derive(Debug, Clone)]
pub struct Node {
    /// Must be kept sorted
    pub children: RefCell<Vec<Rc<RefCell<Mininode>>>>,
    pub left: Option<Rc<RefCell<Node>>>,
    pub right: Option<Rc<RefCell<Node>>>,
}

#[derive(Debug, Clone)]
pub struct Mininode {
    pub disambiguator: SDIS, // SDIS
    pub atom: Atom,
    pub tombstone: bool,

    pub left: Option<Rc<RefCell<Node>>>,
    pub right: Option<Rc<RefCell<Node>>>,
}

#[derive(Debug, Clone)]
pub enum AtPosition {
    Major(Option<Rc<RefCell<Node>>>),
    Mini(Option<Rc<RefCell<Mininode>>>),
}

impl Node {
    pub fn new() -> Self {
        Self {
            children: RefCell::new(Vec::new()),
            left: None,
            right: None,
        }
    }
    pub fn new_with_mini(atom: Atom, dis: SDIS) -> Self {
        let mini = Rc::new(RefCell::new(Mininode {
            atom: atom,
            disambiguator: dis,
            left: None,
            right: None,
            tombstone: false,
        }));
        Self {
            children: RefCell::new(vec![mini]),
            left: None,
            right: None,
        }
    }
}
