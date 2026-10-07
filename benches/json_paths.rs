#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use json_traits::{DiffJson, JsonPaths, PathPattern};
use serde_json::{Value, json};

fn sample_document() -> Value {
    let contacts: Vec<_> = (0..200)
        .map(|index| {
            json!({
                "info": {
                    "name": format!("person-{index}"),
                    "number": index,
                    "isAwesome": index % 2 == 0,
                }
            })
        })
        .collect();
    json!({
        "prop1": { "prop2": "value" },
        "contacts": contacts,
    })
}

fn flatten(c: &mut Criterion) {
    let value = sample_document();
    let leaves = value.paths_and_values().len() as u64;
    let mut group = c.benchmark_group("flatten");
    group.throughput(Throughput::Elements(leaves));
    group.bench_function("paths_and_values", |bencher| {
        bencher.iter(|| black_box(&value).paths_and_values());
    });
    group.finish();
}

fn diff(c: &mut Criterion) {
    let left = sample_document();
    let mut right = left.clone();
    right["contacts"][10]["info"]["number"] = json!(999);
    right["prop1"]["prop2"] = json!("other");
    c.bench_function("diff_with", |bencher| {
        bencher.iter(|| black_box(&left).diff_with(black_box(&right)));
    });
}

fn patterns(c: &mut Criterion) {
    let paths: Vec<String> = (0..500)
        .map(|index| format!("contacts.{index}.info.name"))
        .chain((0..500).map(|index| format!("contacts.{index}.info.extra")))
        .collect();
    let supported = [
        "prop1.prop2",
        "contacts.*.info.name",
        "contacts.*.info.number",
    ];
    c.bench_function("is_supported_by", |bencher| {
        bencher.iter(|| {
            black_box(&paths)
                .iter()
                .filter(|path| path.as_str().is_supported_by(supported))
                .count()
        });
    });
}

criterion_group!(benches, flatten, diff, patterns);
criterion_main!(benches);
