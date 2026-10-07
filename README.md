# json-traits

Server-side helper for frontends that address JSON the way [leaf-validator](https://www.npmjs.com/package/leaf-validator) does, and that send a PATCH of the leaves that changed.

[leaf-validator](https://github.com/stewie1570/leaf-validator) is the client library this crate was written to match. A screen binds a control to a dotted location such as `person.contact.phoneNumber`. Its `leafDiff` turns an edit into one entry per leaf, `{ location, updatedValue }`, instead of one blob for a whole new object. The client sends that list as a PATCH. Concurrent editors then overwrite only the leaves they changed. Those locations are the same dotted paths [MongoDB calls dot notation](https://www.mongodb.com/docs/manual/core/document/#dot-notation). [mongo-leaf-validator-example](https://github.com/stewie1570/mongo-leaf-validator-example) shows a server applying them. leaf-validator encourages the [normalized state shape](https://redux.js.org/usage/structuring-reducers/normalizing-state-shape): each value lives in one place, and an update names that place.

This crate is the Rust server helper for that PATCH. [JsonElementExtensions](https://github.com/stewie1570/JsonElementExtensions) is the .NET one. Both flatten a stored document into the same leaf paths the client uses, diff two documents, and match paths against patterns so the server can accept only the leaves a caller is allowed to change. `*` matches one path segment.

`serde_json::Value` stands in for .NET's `JsonElement`. The methods are traits. Import a trait and its methods are available on `Value`, the same way a `using` brings an extension method into scope.

```rust
use json_traits::prelude::*;
use serde_json::json;

let left = json!({
    "prop1": { "prop2": 1 },
    "contacts": [
        { "info": { "name": "Stewie" } },
        { "info": { "number": 12 } }
    ]
});
let right = json!({
    "prop1": { "prop2": "value2" },
    "contacts": [
        { "info": { "name": "Stewie" } },
        { "info": { "number": 13 } }
    ]
});

// "prop1.prop2", "contacts.0.info.name", "contacts.1.info.number"
let _paths = left.paths_and_values();

// prop1.prop2 and contacts.1.info.number differ
let _diff = left.diff_with(&right);

let patterns = ["prop1.prop2", "contacts.*.info.name"];
assert!("contacts.0.info.name".is_supported_by(patterns));
```

## Rust on this machine

Stable Rust is installed with [rustup](https://rustup.rs/). New terminals pick it up from your shell profile. In a terminal that was already open, run:

```bash
. "$HOME/.cargo/env"
rustc --version
```

On another machine:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

This repository pins the stable toolchain in `rust-toolchain.toml`, including rustfmt and clippy. The first `cargo` command inside the directory installs that toolchain if it is missing.

Useful commands while learning the layout:

| Command | What it does |
| --- | --- |
| `cargo test` | Runs the unit, integration, and documentation tests |
| `cargo bench` | Runs the Criterion benchmarks |
| `cargo doc --open` | Builds the API docs and opens them |
| `cargo run --example list_paths < document.json` | Prints every leaf path in a file |
| `cargo fmt --all -- --check` | Checks formatting |
| `cargo clippy --all-targets --all-features -- -D warnings` | Lints the crate |
| `cargo package --all-features` | Builds the crate archive |
| `cargo publish --dry-run` | Checks that crates.io would accept the archive |

## Add it to another project

Once version 0.1.0 is on crates.io:

```toml
[dependencies]
json-traits = "0.1"
serde_json = "1"
```

Until then, depend on the git repository:

```toml
[dependencies]
json-traits = { git = "https://github.com/stewie1570/JsonTraits" }
serde_json = "1"
```

Bring the methods into scope with `use json_traits::prelude::*;`, or import `JsonPaths`, `DiffJson`, and `PathPattern` individually.

## What the methods do

A leaf-validator PATCH is a list of leaf locations. These methods are how the server speaks that same shape.

`paths_and_values` walks a value and returns a `BTreeMap<String, JsonScalar>` of leaves. Object keys and array indexes become path segments:

```text
prop1.prop2
contacts.0.info.name
contacts.1.info.number
contacts.2.info.isAwesome
```

A root scalar such as `"value"` is stored at the empty path `""`. Empty objects and empty arrays have no leaves, so they produce no paths.

`diff_with` compares those leaves. Each difference is `(left, right)`. A path that exists on only one side uses `JsonScalar::Undefined` for the other side. That is the server's view of the same leaf diff the client would send. The method also works on the maps returned by `paths_and_values`, which is how you diff after filtering.

`is_supported_by` reports whether a PATCH path is one the server allows. `is_a_path_match_with` is called on the pattern, not the path:

```rust
"contacts.*.info.name".is_a_path_match_with("contacts.0.info.name");
```

`*` matches one whole segment. It does not match across dots, so `contacts.*.info.name` does not match `contacts.0.info.name.last`.

## Behavior worth knowing

- Paths are sorted, because the map is a `BTreeMap`.
- Dots inside an object key are not escaped. `"a.b"` and `{"a":{"b": ...}}` become the same path. That matches the C# library.
- JSON `null` equals JSON `null`. The C# `DiffWith` calls `.Equals` on the boxed value and throws when that value is null; this port treats null as a real leaf.
- Integer `1` and decimal `1.0` are the same value. Two integers still compare exactly.

## Checks before a release

[`.github/workflows/ci.yml`](.github/workflows/ci.yml) gives each check its own job. Format, Clippy, tests, the `list_paths` example, and benchmarks start together. Packaging waits until format, Clippy, tests, and the example have succeeded. The packaged crate is tested in a later job. The publish dry run waits until that packaged test and the benchmarks have both succeeded.

[`.github/workflows/publish.yml`](.github/workflows/publish.yml) runs when a pull request is merged into `master`. It reads `version` from `Cargo.toml`. If the tag `v` plus that version is not already on the repository, the version is new: the same checks run, then `cargo publish`, then the workflow pushes that tag. A later merge that leaves the version unchanged finds the tag and does not publish again.

## Publish the crate

1. Create the GitHub repository `stewie1570/JsonTraits` and push this project, including the workflow files.
2. Create a [crates.io](https://crates.io/) account and verify your email.
3. Configure publishing in one of these ways:
   - **Trusted publishing** (no long-lived token). On crates.io, add a trusted publisher for repository `stewie1570/JsonTraits`, workflow `publish.yml`, and environment `crates-io`. Create a GitHub environment named `crates-io` on the repository. The workflow already requests `id-token: write`.
   - **API token.** Create a crates.io token and save it as the `CARGO_REGISTRY_TOKEN` repository secret.
4. Merge a pull request into `master`. The workflow publishes when `v` plus the `Cargo.toml` version is not already a tag. The first such merge publishes `0.1.0` and pushes `v0.1.0`. To release `0.2.0` later, change `version` in `Cargo.toml` and merge that pull request.

A crates.io version cannot be overwritten. If a published version is broken, `cargo yank --version 0.1.0` hides it from new dependents without deleting it.

## Project map

```text
src/lib.rs                 crate root
src/scalar.rs              JsonScalar, one JSON leaf
src/flatten.rs             JsonPaths::paths_and_values
src/diff.rs                DiffJson::diff_with
src/path_pattern.rs        PathPattern
tests/                     the C# cases, ported, plus edge cases
benches/json_paths.rs      Criterion benchmarks
examples/list_paths.rs     read JSON from stdin, print paths
.github/workflows/         CI jobs and crates.io publish
```
