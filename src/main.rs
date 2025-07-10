use std::{cell::RefCell, io::Error, rc::Rc};

use crate::node::Node;

mod node;
mod pos_id;
mod treedoc;

fn main() -> Result<(), Error> {
    let root = Node::new_with_mini('c', 1u64);

    let mut td: treedoc::Treedoc = treedoc::Treedoc {
        root: Some(Rc::new(RefCell::new(root))),
        doc_length: 1,
        unique_disambiguator: 1u64,
    };

    let sig = td.insert(0, 'b')?;
    td.apply(treedoc::Signal::Insert(sig))?;

    let sig = td.insert(0, 'a')?;
    td.apply(treedoc::Signal::Insert(sig))?;

    let sig = td.insert(3, 'e')?;
    td.apply(treedoc::Signal::Insert(sig))?;

    let sig = td.insert(3, 'd')?;
    td.apply(treedoc::Signal::Insert(sig))?;

    let sig = td.insert(5, 'f')?;
    td.apply(treedoc::Signal::Insert(sig))?;

    let mut nodes = Vec::new();
    treedoc::Treedoc::traverse_in_and_collect(&td.root, &mut nodes);
    println!("{:?}", nodes);

    Ok(())
}

// a b c
