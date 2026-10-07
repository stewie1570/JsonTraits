# json-traits

Rust helpers for JSON documents addressed by dotted leaf paths. Flatten a `serde_json::Value` into those paths, diff two documents leaf by leaf, and test a path against an allow list. A server uses that to accept a PATCH of individual leaves, such as the body produced by [leaf-validator](https://www.npmjs.com/package/leaf-validator)'s `leafDiff`.

Version 0.1.0 is on [crates.io](https://crates.io/crates/json-traits). API docs are on [docs.rs](https://docs.rs/json-traits).

## Leaf paths

A leaf is a JSON string, number, boolean, or `null`. Objects and arrays are containers. A path names one leaf by joining object keys and array indexes with `.`:

```text
person.contact.phoneNumber
contacts.0.info.name
contacts.1.info.number
```

[leaf-validator](https://github.com/stewie1570/leaf-validator) is a React library that binds a control to one of these locations. Its `leafDiff` turns an edit into one entry per leaf:

```json
[
  { "location": "person.contact.phoneNumber", "updatedValue": "111-222-3333" }
]
```

`location` is the dotted path. `updatedValue` is the new leaf. Sending that list as a PATCH means two people editing the same document overwrite only the leaves each of them changed. [MongoDB calls the same addressing dot notation](https://www.mongodb.com/docs/manual/core/document/#dot-notation): `"contacts.2"` is the third array element, and `"person.contact.phoneNumber"` is a field inside an embedded document. [mongo-leaf-validator-example](https://github.com/stewie1570/mongo-leaf-validator-example) is a server that receives this list and applies each location with a MongoDB update.

leaf-validator also has a `diff` that can put a whole new object in one entry. `leafDiff`, and this crate, expand that object into one entry per leaf inside it. The paths line up with a [normalized document](https://redux.js.org/usage/structuring-reducers/normalizing-state-shape): each value lives at one location, and an update names that location.

[JsonElementExtensions](https://github.com/stewie1570/JsonElementExtensions) is the .NET library for the same three operations. The methods here are traits on `serde_json::Value` and on `&str`. Import a trait, or the prelude, and the methods are in scope.

| JsonElementExtensions | json-traits |
| --- | --- |
| `PathsAndValuesDictionary` | `JsonPaths::paths_and_values` |
| `DiffWith` | `DiffJson::diff_with` |
| `IsSupportedBy` | `PathPattern::is_supported_by` |
| `IsAPathMatchWith` | `PathPattern::is_a_path_match_with` |

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

Bring every method into scope with `use json_traits::prelude::*;`, or import `JsonPaths`, `DiffJson`, and `PathPattern` on their own. `JsonScalar` is the leaf type and is exported from the crate root.

## Check a PATCH, then diff the document

`is_supported_by` is called on the path from the client. The argument is the list of patterns the server allows. `*` matches one path segment, so an allow list containing `contacts.*.info.name` accepts every contact name.

```rust
use json_traits::prelude::*;
use serde_json::json;

let stored = json!({
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

let allowed = [
    "person.contact.phoneNumber",
    "person.contact.email",
    "contacts.*.info.name",
];

assert!("person.contact.phoneNumber".is_supported_by(allowed));
assert!("contacts.1.info.name".is_supported_by(allowed));
assert!(!"person.firstName".is_supported_by(allowed));

let edited = json!({
    "person": {
        "firstName": "Ada",
        "contact": {
            "email": "ada@example.com",
            "phoneNumber": "111-222-3333"
        }
    },
    "contacts": [
        { "info": { "name": "Ada" } },
        { "info": { "name": "Grace" } }
    ]
});

let diff = stored.diff_with(&edited);
assert!(diff.contains_key("person.contact.phoneNumber"));
assert_eq!(diff.len(), 1);
```

Reject any `location` for which `is_supported_by` is false before writing `updatedValue`. This crate stops at the path check and the leaf diff. Writing the accepted leaves is the storage layer: the MongoDB example turns each location into a `$set` or an `$unset` on that dotted path.

`diff_with` also works on the maps from `paths_and_values`, so a caller can filter the leaves and then diff:

```rust
use std::collections::BTreeMap;

use json_traits::{DiffJson, JsonPaths, JsonScalar, PathPattern};
use serde_json::json;

let allowed = ["contacts.*.info.name"];
let before = json!({"contacts": [{"info": {"name": "Ada"}}]}).paths_and_values();
let after = json!({"contacts": [{"info": {"name": "Grace"}}]}).paths_and_values();

let kept = |map: BTreeMap<String, JsonScalar>| {
    map.into_iter()
        .filter(|(path, _)| path.is_supported_by(allowed))
        .collect::<BTreeMap<_, _>>()
};

let diff = kept(before).diff_with(&kept(after));
assert!(diff.contains_key("contacts.0.info.name"));
```

## `paths_and_values`

`JsonPaths::paths_and_values` walks a value and returns a `BTreeMap<String, JsonScalar>`. Object keys and array indexes become path segments. The map is sorted by path.

```text
person.contact.email
person.contact.phoneNumber
person.firstName
contacts.0.info.name
contacts.1.info.name
```

A document that is itself a string, number, boolean, or `null` is stored at the empty path `""`. An empty object or an empty array has no leaves, so it contributes no paths. Replacing a leaf with `{}` or `[]` shows up as that leaf disappearing. Array indexes are decimal and unpadded: the eleventh element is `10`.

## `diff_with`

`DiffJson::diff_with` compares leaves. Paths that are equal on both sides are omitted. Each difference is a pair `(left, right)`.

`JsonScalar::Undefined` fills the side where the path is absent. A removed leaf is `(value, Undefined)`. An added leaf is `(Undefined, value)`. JSON `null` equals JSON `null`. A stored `null` compared with a missing path is a difference: `(Null, Undefined)`.

In JsonElementExtensions, `DiffWith` calls `.Equals` on the boxed value, and that call throws when the value is null. Here `null` is a leaf.

## Path patterns

Call `is_supported_by` on the path. Call `is_a_path_match_with` on the pattern:

```rust
use json_traits::PathPattern;

assert!("contacts.*.info.name".is_a_path_match_with("contacts.0.info.name"));
```

`*` matches one whole segment. `contacts.*.info.name` matches `contacts.0.info.name`. It does not match `contacts.0.info.name.last`, and `*` does not match `person.contact`. A pattern with no `*` matches only that exact path. An empty pattern list supports nothing.

The same trait is implemented for `str`, so a `String` path works through deref.

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

Integer `1` and decimal `1.0` compare equal. Two integers compare exactly, so a pair of large integers that would collapse to the same floating-point number stay different. A JSON string and a JSON number are different even when their text looks the same.

## Behavior worth knowing

- Paths come out sorted, because the map is a `BTreeMap`.
- A dot inside an object key is copied into the path as written. The key `"a.b"` and the nested object `{"a": {"b": ...}}` both become the path `a.b`. JsonElementExtensions and MongoDB dot notation have the same overlap, so keep dots out of keys that you address this way.
- Empty containers contribute no leaves. A PATCH that needs to record "this field is now an empty object" has to be represented by the leaves that disappeared.

## From a clone of this repository

```bash
cargo test
cargo run --example list_paths < document.json
cargo doc --open
```

`list_paths` reads a JSON document from standard input and prints every leaf path. `rust-toolchain.toml` selects the stable toolchain, including rustfmt and clippy, for commands run inside the repository.

## License

MIT. See [LICENSE](LICENSE).
