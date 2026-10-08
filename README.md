# json-traits

Flatten a `serde_json::Value` into dotted leaf paths, diff two JSON values by those leaves, and match a path against patterns.

These paths are the same locations [leaf-validator](https://www.npmjs.com/package/leaf-validator) uses. That React library binds a control to a path such as `person.contact.phoneNumber`. Its `leafDiff` reports one entry per leaf:

```json
{ "location": "person.contact.phoneNumber", "updatedValue": "111-222-3333" }
```

`location` is the dotted path and `updatedValue` is the new leaf. `leafDiff` expands an object into one entry per leaf inside it. leaf-validator's `diff` can place a whole object in a single entry. This crate follows the leaf form.

[MongoDB calls the same addressing dot notation](https://www.mongodb.com/docs/manual/core/document/#dot-notation): `"contacts.2"` is the third array element, and `"person.contact.phoneNumber"` is a field of an embedded document. [JsonElementExtensions](https://github.com/stewie1570/JsonElementExtensions) is the .NET library for the same operations.

Releases are published to [crates.io](https://crates.io/crates/json-traits). API docs are on [docs.rs](https://docs.rs/json-traits).

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

`JsonPaths::paths_and_values` walks a value and returns a `BTreeMap<String, JsonScalar>`. A leaf is a JSON string, number, boolean, or `null`. Objects and arrays are containers, so the walk continues through them. Object keys and array indexes become path segments. The map is sorted by path.

```rust
use std::collections::BTreeMap;

use json_traits::{JsonPaths, JsonScalar};
use serde_json::json;

let document = json!({
    "prop1": { "prop2": "value" },
    "contacts": [
        { "info": { "name": "Stewie" } },
        { "info": { "number": 12 } },
        { "info": { "isAwesome": true } }
    ]
});

assert_eq!(
    document.paths_and_values(),
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
```

A value that is itself a string, number, boolean, or `null` is stored at the empty path `""`. An empty object or an empty array has no leaves, so it contributes no paths. Array indexes are decimal and unpadded: index 10 is the segment `10`.

## `diff_with`

`DiffJson::diff_with` compares those leaves. Paths that are equal on both sides are omitted. Each difference is a pair `(left, right)`. `JsonScalar::Undefined` fills the side where the path is absent.

```rust
use std::collections::BTreeMap;

use json_traits::{DiffJson, JsonScalar};
use serde_json::json;

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
```

`contacts.0.info.name` is absent from the result because both documents have `"Stewie"` there. The new leaf `contacts.3.info.isSomething` is `(Undefined, true)`. A removed leaf is `(value, Undefined)`. JSON `null` equals JSON `null`. A `null` leaf compared with a missing path is `(Null, Undefined)`.

In JsonElementExtensions, `DiffWith` calls `.Equals` on the boxed value, and that call throws when the value is null. Here `null` is a leaf.

The same method is implemented for `BTreeMap<String, JsonScalar>`. `left.paths_and_values().diff_with(&right.paths_and_values())` returns the map above.

## Path patterns

`is_supported_by` is called on a path and takes the patterns to test. `*` matches one whole segment. `contacts.*.info.name` matches `contacts.0.info.name`. It leaves `contacts.0.info.name.last` unmatched, because that path has one more segment.

```rust
use json_traits::PathPattern;

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
```

`is_a_path_match_with` is the same test called on the pattern:

```rust
use json_traits::PathPattern;

assert!("contacts.*.info.name".is_a_path_match_with("contacts.0.info.name"));
```

A pattern with no `*` matches that exact path. An empty pattern list matches nothing. The trait is implemented for `str`, so a `String` path works through deref.

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
