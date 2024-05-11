use typeset::{entry::Entry, TypeSet};
#[test]
fn smoke() {
    let mut set = TypeSet::new();
    assert_eq!(set.len(), 0);
    assert!(set.is_empty());
    assert!(!set.contains::<bool>());
    set.insert(true);
    assert!(set.contains::<bool>());
    assert!(!set.is_empty());
    assert_eq!(set.len(), 1);
    assert!(set.get::<bool>().unwrap());
    set.insert(false);
    assert_eq!(set.len(), 1);
    assert!(!set.get::<bool>().unwrap());

    assert_eq!(*set.entry().or_insert("hello"), "hello");
    set.insert(String::from("hello"));
    assert_eq!(
        *set.entry()
            .and_modify(|h: &mut String| h.push_str(" world"))
            .or_default(),
        "hello world"
    );

    set.get_mut::<String>().unwrap().make_ascii_uppercase();
    assert_eq!(*set.get_or_insert(String::from("unused")), "HELLO WORLD");
    assert_eq!(
        *set.get_or_insert_with(|| String::from("unused")),
        "HELLO WORLD"
    );
    assert_eq!(*set.get_or_insert_default::<String>(), "HELLO WORLD");
    assert_eq!(set.remove::<String>().unwrap(), "HELLO WORLD");
    assert_eq!(set.remove::<String>(), None);
}

#[test]
fn merge() {
    let mut set_a = TypeSet::new().with(8u8).with("hello");
    let set_b = TypeSet::new().with(32u32).with("world");
    set_a.merge(set_b);
    assert_eq!(set_a.get::<u8>(), Some(&8));
    assert_eq!(set_a.get::<u32>(), Some(&32));
    assert_eq!(set_a.get::<&'static str>(), Some(&"world"));
    assert_eq!(set_a.len(), 3);
}

#[test]
fn entry() {
    let mut set = TypeSet::new();
    let entry = set.entry::<String>();
    let Entry::Vacant(vacant_entry) = entry else {
        panic!()
    };
    vacant_entry.insert("hello".into());

    let mut entry = set.entry::<String>();
    let Entry::Occupied(occupied_entry) = &mut entry else {
        panic!()
    };
    assert_eq!(&**occupied_entry, "hello"); //deref
    assert_eq!(occupied_entry.get(), "hello");
    occupied_entry.get_mut().push_str(" world");
    occupied_entry.make_ascii_uppercase(); //deref mut

    let Entry::Occupied(occupied_entry) = entry else {
        panic!()
    };
    assert_eq!(occupied_entry.remove(), "HELLO WORLD");

    assert_eq!(*set.entry::<usize>().or_insert(10), 10);
    assert_eq!(
        *set.entry()
            .and_modify(|x: &mut usize| *x += 10)
            .or_default(),
        20
    );

    assert_eq!(
        *set.entry::<String>()
            .and_modify(|_| panic!("never called"))
            .or_insert_with(|| String::from("hello")),
        "hello"
    );
}