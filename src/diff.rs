use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde_json::Value;

use crate::{JsonPaths, JsonScalar};

/// Compares JSON values, or maps of paths, by leaf value.
///
/// This is the Rust form of `DiffWith`. The pair is `(self, other)`.
/// [`JsonScalar::Undefined`] fills the side where a path is absent.
pub trait DiffJson {
    /// Paths whose leaf values differ.
    ///
    /// Paths that are equal on both sides are omitted. JSON `null` equals
    /// JSON `null`.
    ///
    /// # Examples
    ///
    /// ```
    /// use json_traits::{DiffJson, JsonScalar};
    /// use serde_json::json;
    ///
    /// let left = json!({ "name": "Stewie", "number": 12 });
    /// let right = json!({ "name": "Stewie", "number": 13, "isAwesome": true });
    ///
    /// let diff = left.diff_with(&right);
    /// assert_eq!(
    ///     diff.get("number"),
    ///     Some(&(JsonScalar::from_integer(12), JsonScalar::from_integer(13)))
    /// );
    /// assert_eq!(
    ///     diff.get("isAwesome"),
    ///     Some(&(JsonScalar::Undefined, JsonScalar::from(true)))
    /// );
    /// assert!(!diff.contains_key("name"));
    /// ```
    #[doc(alias = "DiffWith")]
    #[must_use]
    fn diff_with(&self, other: &Self) -> BTreeMap<String, (JsonScalar, JsonScalar)>;
}

impl DiffJson for Value {
    fn diff_with(&self, other: &Self) -> BTreeMap<String, (JsonScalar, JsonScalar)> {
        self.paths_and_values().diff_with(&other.paths_and_values())
    }
}

impl DiffJson for BTreeMap<String, JsonScalar> {
    fn diff_with(&self, other: &Self) -> BTreeMap<String, (JsonScalar, JsonScalar)> {
        // Both maps are already ordered, so walk them together. Equal leaves
        // are skipped without cloning their paths.
        let mut diff = BTreeMap::new();
        let mut left_keys = self.iter();
        let mut right_keys = other.iter();
        let mut left = left_keys.next();
        let mut right = right_keys.next();
        loop {
            match (left, right) {
                (None, None) => break,
                (Some((key, value)), None) => {
                    diff.insert(key.clone(), (value.clone(), JsonScalar::Undefined));
                    left = left_keys.next();
                }
                (None, Some((key, value))) => {
                    diff.insert(key.clone(), (JsonScalar::Undefined, value.clone()));
                    right = right_keys.next();
                }
                (Some((left_key, left_value)), Some((right_key, right_value))) => {
                    match left_key.cmp(right_key) {
                        Ordering::Less => {
                            diff.insert(
                                left_key.clone(),
                                (left_value.clone(), JsonScalar::Undefined),
                            );
                            left = left_keys.next();
                        }
                        Ordering::Greater => {
                            diff.insert(
                                right_key.clone(),
                                (JsonScalar::Undefined, right_value.clone()),
                            );
                            right = right_keys.next();
                        }
                        Ordering::Equal => {
                            if left_value != right_value {
                                diff.insert(
                                    left_key.clone(),
                                    (left_value.clone(), right_value.clone()),
                                );
                            }
                            left = left_keys.next();
                            right = right_keys.next();
                        }
                    }
                }
            }
        }
        diff
    }
}
