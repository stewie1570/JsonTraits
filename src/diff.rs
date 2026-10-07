use std::collections::{BTreeMap, BTreeSet};

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
        let mut keys = BTreeSet::new();
        keys.extend(self.keys().cloned());
        keys.extend(other.keys().cloned());

        keys.into_iter()
            .filter_map(|key| {
                let left = self.get(&key);
                let right = other.get(&key);
                match (left, right) {
                    (Some(left), Some(right)) if left == right => None,
                    _ => Some((
                        key,
                        (
                            left.cloned().unwrap_or(JsonScalar::Undefined),
                            right.cloned().unwrap_or(JsonScalar::Undefined),
                        ),
                    )),
                }
            })
            .collect()
    }
}
