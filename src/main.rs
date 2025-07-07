mod node;
mod pos_id;
mod treedoc;

fn main() {
    let td = treedoc::Treedoc {
        root: None,
        doc_length: 1,
        unique_disambiguator: 45u64,
    };

    let mut nodes = Vec::new();
    treedoc::Treedoc::traverse_in_and_collect(&td.root, &mut nodes);
    println!("{:?}", nodes)
}
