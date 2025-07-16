use std::{cell::RefCell, io::Error, rc::Rc};

pub type Atom = char;

// SDIS only on mininodes -> this is why PathComponents with
// disambiguators are mininodes and ones without are major nodes
pub type SDIS = u64;

// pub trait TreeNode {
//     fn add_left(&mut self, node: Node);
//     fn add_right(&mut self, node: Node);
//     fn add_mini(&self, mini: Mininode);
// }

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

    pub fn add_mini(&self, mini: Mininode) {
        self.children.borrow_mut().push(Rc::new(RefCell::new(mini)));
        self.children
            .borrow_mut()
            .sort_by_key(|m| m.borrow().disambiguator);
    }

    pub fn remove_mini(&self, dis: Option<SDIS>) -> Result<(), Error> {
        let init_len = self.children.borrow().len();
        self.children
            .borrow_mut()
            .retain(|mn| mn.borrow().disambiguator != dis.unwrap());
        if self.children.borrow().len() == init_len {
            return Err(Error::from(std::io::ErrorKind::NotFound));
        }
        Ok(())
    }

    pub fn add_left(&mut self, node: Node) {
        self.left = Some(Rc::new(RefCell::new(node)))
    }

    pub fn add_right(&mut self, node: Node) {
        self.right = Some(Rc::new(RefCell::new(node)))
    }
}

impl Mininode {
    pub fn new_with_atom(atom: Atom, dis: SDIS) -> Self {
        Mininode {
            atom: atom,
            disambiguator: dis,
            left: None,
            right: None,
            tombstone: false,
        }
    }

    pub fn add_left(&mut self, node: Node) {
        self.left = Some(Rc::new(RefCell::new(node)))
    }

    pub fn add_right(&mut self, node: Node) {
        self.right = Some(Rc::new(RefCell::new(node)))
    }

    pub fn add_mini(&self, mini: Mininode) {}
}
