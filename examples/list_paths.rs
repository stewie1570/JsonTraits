#![allow(missing_docs)]

//! Print every leaf path in a JSON document read from standard input.
//!
//! Paths are sorted. Pipe a file, or paste JSON and end the input:
//!
//! ```text
//! cargo run --example list_paths < document.json
//! ```

use std::io::{self, Read};

use json_traits::JsonPaths;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let value: serde_json::Value = serde_json::from_str(&input)?;
    for path in value.paths_and_values().keys() {
        println!("{path}");
    }
    Ok(())
}
