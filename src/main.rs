use simple_text_crdt::{node::Node, treedoc::Signal, treedoc::Treedoc};
use std::{cell::RefCell, io::Error, rc::Rc};

// To deal with on conc:
// Simulate simultaneous insertions / deletions -> diff disambiq., same path
// Interface for syncing ACTUAL concurrent use (simple server?)
// ...

fn main() -> Result<(), Error> {
    let root = Node::new_with_mini('c', 1u64);

    let mut td = Treedoc {
        root: Some(Rc::new(RefCell::new(root))),
        doc_length: 1,
        unique_disambiguator: 1u64,
    };

    let sig_f = td.insert(0, 'a')?;
    let sig_s = td.insert(0, 'b')?;

    td.apply(Signal::Insert(sig_f))?;
    td.apply(Signal::Insert(sig_s))?;

    let mut nodes = Vec::new();
    Treedoc::traverse_in_and_collect(&td.root, &mut nodes);
    println!("{:?}", nodes);

    Ok(())
}
