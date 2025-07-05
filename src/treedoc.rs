// This is the document that is copied to all the peers 
// -> aka the document state / atom buffer

use crate::node;

/// Essentially a binary tree (BT)
#[derive(Debug)]
pub struct Treedoc {
    root: node::Node
}