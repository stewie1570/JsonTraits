#![allow(missing_docs)]

use json_traits::PathPattern;

#[test]
fn filtering_to_known_paths() {
    let paths = [
        "prop1.prop2",
        "contacts.0.info.name",
        "contacts.0.info.name.last",
        "contacts.1.info.number",
        "contacts.2.info.isAwesome",
    ];
    let patterns = [
        "prop1.prop2",
        "contacts.*.info.name",
        "contacts.*.info.number",
    ];

    let matched: Vec<_> = paths
        .into_iter()
        .filter(|path| path.is_supported_by(patterns))
        .collect();

    assert_eq!(
        matched,
        [
            "prop1.prop2",
            "contacts.0.info.name",
            "contacts.1.info.number",
        ]
    );
}

#[test]
fn star_matches_one_segment_only() {
    assert!("contacts.*.info.name".is_a_path_match_with("contacts.0.info.name"));
    assert!(!"contacts.*.info.name".is_a_path_match_with("contacts.0.info.name.last"));
    assert!("*".is_a_path_match_with("prop1"));
    assert!(!"*".is_a_path_match_with("prop1.prop2"));
    assert!("*.*".is_a_path_match_with("a.b"));
    assert!(!"*.*".is_a_path_match_with("a"));
}

#[test]
fn exact_segments_must_agree() {
    assert!("prop1.prop2".is_a_path_match_with("prop1.prop2"));
    assert!(!"prop1.prop2".is_a_path_match_with("prop1.prop3"));
    assert!(!"prop1.prop2".is_a_path_match_with("prop1"));
    assert!(!"prop1".is_a_path_match_with("prop1.prop2"));
}

#[test]
fn an_empty_pattern_list_supports_nothing() {
    assert!(!"prop1".is_supported_by(Vec::<&str>::new()));
}

#[test]
fn string_paths_use_the_same_trait() {
    let path = String::from("contacts.1.info.number");
    assert!(path.is_supported_by(["contacts.*.info.number"]));
}
