use simple_text_crdt::treedoc::{Signal, Treedoc};

#[test]
fn test_insert_start() {
    let corr_string = "abc";
    let mut td = Treedoc::new('c');

    let sig = td.insert(0, 'b');
    let res = td.apply(Signal::Insert(sig.unwrap()));
    assert!(res.is_ok());

    let sig = td.insert(0, 'a');
    let res = td.apply(Signal::Insert(sig.unwrap()));
    assert!(res.is_ok());

    let mut nodes = Vec::new();
    Treedoc::traverse_in_and_collect(&td.root, &mut nodes);
    let res_string: String = nodes.iter().collect();
    assert_eq!(res_string, corr_string)
}

#[test]
fn test_insert_end() {
    let corr_string = "abc";
    let mut td = Treedoc::new('a');

    let sig = td.insert(td.doc_length, 'b');
    let res = td.apply(Signal::Insert(sig.unwrap()));
    assert!(res.is_ok());

    let sig = td.insert(td.doc_length, 'c');
    let res = td.apply(Signal::Insert(sig.unwrap()));
    assert!(res.is_ok());

    let mut nodes = Vec::new();
    Treedoc::traverse_in_and_collect(&td.root, &mut nodes);
    let res_string: String = nodes.iter().collect();
    assert_eq!(res_string, corr_string)
}

#[test]
fn test_insert_between() {
    let corr_string = "abc";
    let mut td = Treedoc::new('a');

    let sig = td.insert(td.doc_length, 'c');
    let res = td.apply(Signal::Insert(sig.unwrap()));
    assert!(res.is_ok());

    let sig = td.insert(1, 'b');
    let res = td.apply(Signal::Insert(sig.unwrap()));
    assert!(res.is_ok());

    let mut nodes = Vec::new();
    Treedoc::traverse_in_and_collect(&td.root, &mut nodes);
    let res_string: String = nodes.iter().collect();
    assert_eq!(res_string, corr_string)
}

#[test]
fn test_delete() {
    let corr_string = "ab";
    let mut td = Treedoc::new('a');

    let sig = td.insert(td.doc_length, 'c');
    let res = td.apply(Signal::Insert(sig.unwrap()));
    assert!(res.is_ok());

    let sig = td.insert(1, 'b');
    let res = td.apply(Signal::Insert(sig.unwrap()));
    assert!(res.is_ok());

    let sig = td.delete(td.doc_length);
    let res = td.apply(Signal::Delete(sig.unwrap()));
    assert!(res.is_ok());

    let mut nodes = Vec::new();
    Treedoc::traverse_in_and_collect(&td.root, &mut nodes);
    let res_string: String = nodes.iter().collect();
    assert_eq!(res_string, corr_string);

    let sig = td.delete(0);
    let res = td.apply(Signal::Delete(sig.unwrap()));
    assert!(res.is_ok());

    let mut nodes = Vec::new();
    Treedoc::traverse_in_and_collect(&td.root, &mut nodes);
    let res_string: String = nodes.iter().collect();
    assert_eq!(res_string, "b");
}
