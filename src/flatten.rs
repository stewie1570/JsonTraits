use std::collections::BTreeMap;

use serde_json::Value;

use crate::JsonScalar;

/// Flattens a JSON value into dotted paths and leaf values.
///
/// This is the Rust form of `PathsAndValuesDictionary`.
pub trait JsonPaths {
    /// Walk this JSON value and return every leaf as `path -> scalar`.
    ///
    /// Object keys and array indexes are joined with `.`. A scalar root
    /// (string, number, bool, or null) is stored under the empty path `""`.
    /// Empty objects and empty arrays contribute no paths, because they have
    /// no leaves.
    ///
    /// # Examples
    ///
    /// ```
    /// use json_traits::{JsonPaths, JsonScalar};
    /// use serde_json::json;
    ///
    /// let document = json!({
    ///     "prop1": { "prop2": "value" },
    ///     "contacts": [
    ///         { "info": { "name": "Stewie" } },
    ///         { "info": { "number": 12 } },
    ///         { "info": { "isAwesome": true } }
    ///     ]
    /// });
    ///
    /// let paths = document.paths_and_values();
    /// assert_eq!(paths.get("prop1.prop2"), Some(&JsonScalar::from("value")));
    /// assert_eq!(
    ///     paths.get("contacts.1.info.number"),
    ///     Some(&JsonScalar::from_integer(12))
    /// );
    /// assert_eq!(
    ///     paths.get("contacts.2.info.isAwesome"),
    ///     Some(&JsonScalar::from(true))
    /// );
    /// ```
    #[doc(alias = "PathsAndValuesDictionary")]
    #[must_use]
    fn paths_and_values(&self) -> BTreeMap<String, JsonScalar>;
}

impl JsonPaths for Value {
    fn paths_and_values(&self) -> BTreeMap<String, JsonScalar> {
        let mut paths = BTreeMap::new();
        // One buffer is reused for every path. Before descending, remember
        // the length and truncate back to it so siblings do not keep the
        // previous segment.
        walk(self, &mut String::new(), &mut paths);
        paths
    }
}

fn walk(value: &Value, path: &mut String, paths: &mut BTreeMap<String, JsonScalar>) {
    match value {
        Value::Object(properties) => {
            for (key, child) in properties {
                let length = path.len();
                push_segment(path, key);
                walk(child, path, paths);
                path.truncate(length);
            }
        }
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                let length = path.len();
                push_segment(path, &index.to_string());
                walk(child, path, paths);
                path.truncate(length);
            }
        }
        Value::Null => {
            paths.insert(path.clone(), JsonScalar::Null);
        }
        Value::Bool(value) => {
            paths.insert(path.clone(), JsonScalar::Bool(*value));
        }
        Value::Number(value) => {
            paths.insert(path.clone(), JsonScalar::Number(value.clone()));
        }
        Value::String(value) => {
            paths.insert(path.clone(), JsonScalar::String(value.clone()));
        }
    }
}

fn push_segment(path: &mut String, segment: &str) {
    if !path.is_empty() {
        path.push('.');
    }
    path.push_str(segment);
}
