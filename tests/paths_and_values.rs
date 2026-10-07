#![allow(missing_docs)]

use std::collections::BTreeMap;

use json_traits::{DiffJson, JsonPaths, JsonScalar};
use serde_json::json;

fn scalar_paths(value: serde_json::Value) -> BTreeMap<String, JsonScalar> {
    value.paths_and_values()
}

#[test]
fn finds_no_paths_for_a_root_scalar() {
    let paths = scalar_paths(json!("value"));
    assert_eq!(
        paths,
        BTreeMap::from([("".to_owned(), JsonScalar::from("value"))])
    );
}

#[test]
fn finds_shallow_paths() {
    let paths = scalar_paths(json!({ "prop1": "value1", "prop2": "value2" }));
    assert_eq!(
        paths,
        BTreeMap::from([
            ("prop1".to_owned(), JsonScalar::from("value1")),
            ("prop2".to_owned(), JsonScalar::from("value2")),
        ])
    );
}

#[test]
fn finds_deep_object_paths() {
    let paths = scalar_paths(json!({
        "prop1": { "prop2": "value" },
        "contacts": { "info": { "name": "Stewie" } }
    }));
    assert_eq!(
        paths,
        BTreeMap::from([
            ("prop1.prop2".to_owned(), JsonScalar::from("value")),
            ("contacts.info.name".to_owned(), JsonScalar::from("Stewie")),
        ])
    );
}

#[test]
fn finds_deep_object_and_array_paths() {
    let paths = scalar_paths(json!({
        "prop1": { "prop2": "value" },
        "contacts": [
            { "info": { "name": "Stewie" } },
            { "info": { "number": 12 } },
            { "info": { "isAwesome": true } }
        ]
    }));
    assert_eq!(
        paths,
        BTreeMap::from([
            ("prop1.prop2".to_owned(), JsonScalar::from("value")),
            (
                "contacts.0.info.name".to_owned(),
                JsonScalar::from("Stewie"),
            ),
            (
                "contacts.1.info.number".to_owned(),
                JsonScalar::from_integer(12),
            ),
            (
                "contacts.2.info.isAwesome".to_owned(),
                JsonScalar::from(true),
            ),
        ])
    );
}

#[test]
fn records_null_false_and_a_root_array() {
    assert_eq!(
        scalar_paths(json!(null)),
        BTreeMap::from([("".to_owned(), JsonScalar::Null)])
    );
    assert_eq!(
        scalar_paths(json!(false)),
        BTreeMap::from([("".to_owned(), JsonScalar::from(false))])
    );
    assert_eq!(
        scalar_paths(json!([{"a": true}, "x"])),
        BTreeMap::from([
            ("0.a".to_owned(), JsonScalar::from(true)),
            ("1".to_owned(), JsonScalar::from("x")),
        ])
    );
}

#[test]
fn empty_containers_have_no_leaves() {
    assert!(scalar_paths(json!({})).is_empty());
    assert!(scalar_paths(json!([])).is_empty());
    assert!(scalar_paths(json!({"a": {}})).is_empty());
    assert!(scalar_paths(json!({"a": []})).is_empty());
}

#[test]
fn array_indexes_are_decimal_and_not_padded() {
    let mut items = vec![json!(null); 10];
    items.push(json!("last"));
    let paths = serde_json::Value::Array(items).paths_and_values();
    assert_eq!(paths.get("10"), Some(&JsonScalar::from("last")));
}

#[test]
fn a_dot_in_an_object_key_is_not_escaped() {
    let flat = json!({ "a.b": 1 }).paths_and_values();
    let nested = json!({ "a": { "b": 1 } }).paths_and_values();
    assert_eq!(flat, nested);
    assert_eq!(flat.get("a.b"), Some(&JsonScalar::from_integer(1)));
}

#[test]
fn diffs_two_json_documents() {
    let left = json!({
        "prop1": { "prop2": 1 },
        "contacts": [
            { "info": { "name": "Stewie" } },
            { "info": { "number": 12 } },
            { "info": { "isAwesome": true } }
        ]
    });
    let right = json!({
        "prop1": { "prop2": "value2" },
        "contacts": [
            { "info": { "name": "Stewie" } },
            { "info": { "number": 13 } },
            { "info": { "isAwesome": false } },
            { "info": { "isSomething": true } }
        ]
    });

    assert_eq!(
        left.diff_with(&right),
        BTreeMap::from([
            (
                "prop1.prop2".to_owned(),
                (JsonScalar::from_integer(1), JsonScalar::from("value2")),
            ),
            (
                "contacts.1.info.number".to_owned(),
                (JsonScalar::from_integer(12), JsonScalar::from_integer(13)),
            ),
            (
                "contacts.2.info.isAwesome".to_owned(),
                (JsonScalar::from(true), JsonScalar::from(false)),
            ),
            (
                "contacts.3.info.isSomething".to_owned(),
                (JsonScalar::Undefined, JsonScalar::from(true)),
            ),
        ])
    );
}

#[test]
fn identical_documents_and_equal_nulls_have_no_diff() {
    let document = json!({"a": null, "b": [1, {"c": false}]});
    assert!(document.diff_with(&document).is_empty());
}

#[test]
fn reports_added_and_removed_paths() {
    let left = json!({"a": 1, "b": 2});
    let right = json!({"b": 2, "c": 3});
    assert_eq!(
        left.diff_with(&right),
        BTreeMap::from([
            (
                "a".to_owned(),
                (JsonScalar::from_integer(1), JsonScalar::Undefined),
            ),
            (
                "c".to_owned(),
                (JsonScalar::Undefined, JsonScalar::from_integer(3)),
            ),
        ])
    );
}

#[test]
fn null_against_a_missing_path_is_a_difference() {
    let diff = json!({"a": null}).diff_with(&json!({}));
    assert_eq!(
        diff.get("a"),
        Some(&(JsonScalar::Null, JsonScalar::Undefined))
    );
}

#[test]
fn integer_and_matching_decimal_are_the_same_value() {
    assert!(json!({"n": 1}).diff_with(&json!({"n": 1.0})).is_empty());
    assert!(!json!({"n": "1"}).diff_with(&json!({"n": 1})).is_empty());
    assert_eq!(
        JsonScalar::from_float(1.0),
        Some(JsonScalar::from_integer(1))
    );
    assert!(JsonScalar::from_float(f64::NAN).is_none());
    assert!(JsonScalar::from_float(f64::INFINITY).is_none());
}

#[test]
fn distinct_large_integers_stay_different() {
    let left = json!({"n": 9007199254740993_u64});
    let right = json!({"n": 9007199254740992_u64});
    assert!(!left.diff_with(&right).is_empty());
}

#[test]
fn diff_of_path_maps_matches_diff_of_documents() {
    let left = json!({"a": 1});
    let right = json!({"a": 2});
    assert_eq!(
        left.diff_with(&right),
        left.paths_and_values().diff_with(&right.paths_and_values())
    );
}
