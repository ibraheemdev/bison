use bison::http::Headers;

#[test]
fn headers() {
    let headers = Headers::new();
    assert!(headers.is_empty());

    headers.add(("A", "A"));
    headers.add(("B", ["A", "B"]));
    headers.add(("C", vec!["A".into(), "B".into(), "C".into()]));

    assert_eq!(headers.len(), 3);

    let a = vec![RcStr::from("A")];
    let ab = vec![RcStr::from("A"), RcStr::from("B")];
    let abc = vec![RcStr::from("A"), RcStr::from("B"), RcStr::from("C")];

    assert_eq!(headers.first("C").as_deref(), Some("A"));

    for (name, value) in ["A", "B", "C"]
        .into_iter()
        .zip([a.clone(), ab.clone(), abc.clone()])
    {
        assert!(headers.get(name).eq(value.clone()));
        assert!(headers.remove(name).eq(value));
        assert_eq!(headers.get(name).count(), 0);
        assert_eq!(headers.remove(name).count(), 0);
    }

    assert!(headers.is_empty());

    headers.add(("A", "A"));
    headers.add(("A", "B"));
    headers.add(("A", "C"));

    assert!(headers.get("A").eq(abc.clone()));
    assert!(headers.remove("A").eq(abc));

    headers.replace(("A", "A"));
    headers.replace(("A", "B"));
    headers.replace(("A", "C"));

    assert!(headers.get("A").eq([RcStr::from("C")]));
    assert!(headers.remove("A").eq([RcStr::from("C")]));
}

#[test]
fn case_insensitive() {
    // TODO
}
