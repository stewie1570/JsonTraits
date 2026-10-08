# json-traits

Flatten a `serde_json::Value` into dotted leaf paths, diff two JSON values by those leaves, and match a path against patterns.

These paths are the same locations [leaf-validator](https://www.npmjs.com/package/leaf-validator) uses. That React library binds a control to a path such as `person.contact.phoneNumber`. Its `leafDiff` reports one entry per leaf:

```json
{ "location": "person.contact.phoneNumber", "updatedValue": "111-222-3333" }
```

`location` is the dotted path and `updatedValue` is the new leaf. `leafDiff` expands an object into one entry per leaf inside it. leaf-validator's `diff` can place a whole object in a single entry. This crate follows the leaf form.

[MongoDB calls the same addressing dot notation](https://www.mongodb.com/docs/manual/core/document/#dot-notation): `"contacts.2"` is the third array element, and `"person.contact.phoneNumber"` is a field of an embedded document. [JsonElementExtensions](https://github.com/stewie1570/JsonElementExtensions) is the .NET library for the same operations.

Version 0.1.0 is on [crates.io](https://crates.io/crates/json-traits). API docs are on [docs.rs](https://docs.rs/json-traits).

## Add it to a project

The crate requires Rust 1.85 or newer. [rustup](https://rustup.rs/) installs the toolchain.

```toml
[dependencies]
json-traits = "0.1"
serde_json = "1"
```

To depend on the repository instead of the published crate:

```toml
[dependencies]
json-traits = { git = "https://github.com/stewie1570/JsonTraits" }
serde_json = "1"
```

Bring the methods into scope with `use json_traits::prelude::*;`, or import `JsonPaths`, `DiffJson`, and `PathPattern` on their own. `JsonScalar` is the leaf type and is exported from the crate root.

| JsonElementExtensions | json-traits |
| --- | --- |
| `PathsAndValuesDictionary` | `JsonPaths::paths_and_values` |
| `DiffWith` | `DiffJson::diff_with` |
| `IsSupportedBy` | `PathPattern::is_supported_by` |
| `IsAPathMatchWith` | `PathPattern::is_a_path_match_with` |

## `paths_and_values`

`JsonPaths::paths_and_values` walks a value and returns a `BTreeMap<String, JsonScalar>`. Object keys and array indexes become path segments. The map is sorted by path.

A leaf is a JSON string, number, boolean, or `null`. Objects and arrays are containers, so the walk continues through them.

```rust
use json_traits::JsonPaths;
use serde_json::json;

let document = json!({
    "person": {
        "firstName": "Ada",
        "contact": {
            "email": "ada@example.com",
            "phoneNumber": "0123456789"
        }
    },
    "contacts": [
        { "info": { "name": "Ada" } },
        { "info": { "name": "Grace" } }
    ]
});

let paths = document.paths_and_values();
assert!(paths.contains_key("person.contact.phoneNumber"));
assert!(paths.contains_key("contacts.0.info.name"));
assert!(paths.contains_key("contacts.1.info.name"));
```

```text
contacts.0.info.name
contacts.1.info.name
person.contact.email
person.contact.phoneNumber
person.firstName
```

A value that is itself a string, number, boolean, or `null` is stored at the empty path `""`. An empty object or an empty array has no leaves, so it contributes no paths. Array indexes are decimal and unpadded: index 10 is the segment `10`.

## `diff_with`

`DiffJson::diff_with` compares leaves. Paths that are equal on both sides are omitted. Each difference is a pair `(left, right)`.

```rust
use json_traits::DiffJson;
use serde_json::json;

let left = json!({
    "person": { "contact": { "phoneNumber": "0123456789" } }
});
let right = json!({
    "person": { "contact": { "phoneNumber": "111-222-3333" } }
});

let diff = left.diff_with(&right);
assert!(diff.contains_key("person.contact.phoneNumber"));
assert_eq!(diff.len(), 1);
```

`JsonScalar::Undefined` fills the side where the path is absent. A removed leaf is `(value, Undefined)`. An added leaf is `(Undefined, value)`. JSON `null` equals JSON `null`. A `null` leaf compared with a missing path is `(Null, Undefined)`.

In JsonElementExtensions, `DiffWith` calls `.Equals` on the boxed value, and that call throws when the value is null. Here `null` is a leaf.

`diff_with` is also implemented for `BTreeMap<String, JsonScalar>`, so two path maps can be compared directly. Filtering the maps first limits the comparison to the paths you keep:

```rust
use std::collections::BTreeMap;

use json_traits::{DiffJson, JsonPaths, JsonScalar, PathPattern};
use serde_json::json;

let patterns = ["contacts.*.info.name"];
let before = json!({"contacts": [{"info": {"name": "Ada"}}]}).paths_and_values();
let after = json!({"contacts": [{"info": {"name": "Grace"}}]}).paths_and_values();

let kept = |map: BTreeMap<String, JsonScalar>| {
    map.into_iter()
        .filter(|(path, _)| path.is_supported_by(patterns))
        .collect::<BTreeMap<_, _>>()
};

let diff = kept(before).diff_with(&kept(after));
assert!(diff.contains_key("contacts.0.info.name"));
```

## Path patterns

`is_supported_by` is called on the path and takes the patterns to test. `is_a_path_match_with` is called on the pattern and takes the path:

```rust
use json_traits::PathPattern;

let patterns = [
    "person.contact.phoneNumber",
    "person.contact.email",
    "contacts.*.info.name",
];

assert!("person.contact.phoneNumber".is_supported_by(patterns));
assert!("contacts.1.info.name".is_supported_by(patterns));
assert!(!"person.firstName".is_supported_by(patterns));

assert!("contacts.*.info.name".is_a_path_match_with("contacts.0.info.name"));
```

`*` matches one whole segment. `contacts.*.info.name` matches `contacts.0.info.name`. It leaves `contacts.0.info.name.last` unmatched, and `*` leaves `person.contact` unmatched. A pattern with no `*` matches that exact path. An empty pattern list matches nothing.

The trait is implemented for `str`, so a `String` path works through deref.

## `JsonScalar`

`JsonScalar` is one leaf:

| Variant | JSON |
| --- | --- |
| `Null` | `null` |
| `Bool` | `true` or `false` |
| `Number` | a JSON number |
| `String` | a JSON string |
| `Undefined` | the path is missing on this side of a diff |

`JsonScalar::from_integer` builds a number from an `i64`. `JsonScalar::from_float` builds one from a finite `f64` and returns `None` for NaN and infinity. `From` is implemented for `bool`, `&str`, and `String`.

Integer `1` and decimal `1.0` compare equal. Two integers compare exactly, so a pair of large integers that would collapse to the same floating-point number stay different. A JSON string and a JSON number stay different when their text looks the same.

## Behavior worth knowing

- Paths come out sorted, because the map is a `BTreeMap`.
- A dot inside an object key is copied into the path as written. The key `"a.b"` and the nested object `{"a": {"b": ...}}` both become the path `a.b`. JsonElementExtensions and MongoDB dot notation share that overlap.
- Empty containers contribute no leaves. Replacing a leaf with `{}` or `[]` shows up as that leaf disappearing.

## From a clone of this repository

```bash
cargo test
cargo run --example list_paths < document.json
cargo doc --open
```

`list_paths` reads a JSON document from standard input and prints every leaf path. `rust-toolchain.toml` selects the stable toolchain, including rustfmt and clippy, for commands run inside the repository.

## License

MIT. See [LICENSE](LICENSE).
