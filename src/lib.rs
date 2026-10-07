//! Server-side helper for clients that address JSON the way
//! [leaf-validator](https://www.npmjs.com/package/leaf-validator) does and
//! PATCH individual leaves.
//!
//! Flatten a document into dotted paths, diff two documents, and match paths
//! against patterns. This is the Rust port of
//! [JsonElementExtensions](https://github.com/stewie1570/JsonElementExtensions).
//! C# extension methods are traits here. Import a trait before calling its
//! methods. That import is what adds `paths_and_values` to
//! [`serde_json::Value`], in the same way a `using` brings an extension method
//! into scope.
//!
//! ```
//! use json_traits::{DiffJson, JsonPaths, JsonScalar, PathPattern};
//! use serde_json::json;
//!
//! let document = json!({
//!     "prop1": { "prop2": "value" },
//!     "contacts": [{ "info": { "name": "Stewie" } }]
//! });
//!
//! let paths = document.paths_and_values();
//! assert_eq!(paths.get("prop1.prop2"), Some(&JsonScalar::from("value")));
//! assert!("contacts.0.info.name".is_supported_by(["contacts.*.info.name"]));
//! ```
//!
//! Dots inside an object key are not escaped. The key `"a.b"` and the nested
//! object `{"a": {"b": ...}}` produce the same path.

mod diff;
mod flatten;
mod path_pattern;
mod scalar;

pub use diff::DiffJson;
pub use flatten::JsonPaths;
pub use path_pattern::PathPattern;
pub use scalar::JsonScalar;

/// The traits that add path, diff, and pattern methods to existing types.
pub mod prelude {
    pub use crate::{DiffJson, JsonPaths, PathPattern};
}
