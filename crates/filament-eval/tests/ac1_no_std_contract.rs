// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-401-AC-1: the evaluation leaf stays no_std and depends only on its
//! three used, no-default-feature support crates.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use ix_trace_rs::trace;

const PACKAGE_MANIFEST: &str = include_str!("../Cargo.toml");
const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
const CRATE_ROOT: &str = include_str!("../src/lib.rs");

#[trace("FR-401-AC-1")]
#[test]
fn evaluation_leaf_manifest_and_root_keep_the_no_std_boundary() {
    assert!(
        CRATE_ROOT.lines().any(|line| line.trim() == "#![no_std]"),
        "filament-eval crate root must declare no_std"
    );
    assert!(
        WORKSPACE_MANIFEST.contains("[workspace.lints.rust]")
            && WORKSPACE_MANIFEST.contains("unsafe_code = \"forbid\"")
            && PACKAGE_MANIFEST.contains("[lints]")
            && PACKAGE_MANIFEST.contains("workspace = true"),
        "the manifest must forbid unsafe Rust"
    );

    let dependencies = normal_dependency_entries(PACKAGE_MANIFEST);
    let dependency_names: BTreeSet<&str> = dependencies.iter().map(|(name, _)| *name).collect();
    assert_eq!(
        dependency_names,
        BTreeSet::from(["filament-exact", "filament-semantic-value", "thiserror"]),
        "only the leaf's directly used support crates are normal dependencies"
    );
    for (name, declaration) in dependencies {
        assert!(
            declaration.contains("default-features = false"),
            "normal dependency {name} must disable default features: {declaration}"
        );
    }
    assert!(
        !PACKAGE_MANIFEST.contains("fsl-")
            && !PACKAGE_MANIFEST.contains("fsl_")
            && !WORKSPACE_MANIFEST.contains("fsl-")
            && !WORKSPACE_MANIFEST.contains("fsl_"),
        "filament-eval manifests must not name an FSL crate"
    );

    let mut sources = String::new();
    append_rust_sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut sources,
    );
    for (dependency, item_prefix) in [
        ("filament-exact", "filament_exact::"),
        ("filament-semantic-value", "filament_semantic_value::"),
        ("thiserror", "thiserror::"),
    ] {
        assert!(
            sources.contains(item_prefix),
            "normal dependency {dependency} must provide an item named by source"
        );
    }
}

fn normal_dependency_entries(manifest: &str) -> Vec<(&str, &str)> {
    let mut in_table = false;
    let mut entries = Vec::new();

    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            let table = line
                .strip_prefix('[')
                .and_then(|table| table.strip_suffix(']'))
                .expect("TOML table header is bracketed");
            in_table = table == "dependencies"
                || (table.starts_with("target.") && table.ends_with(".dependencies"));
            continue;
        }
        if !in_table || line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (name, declaration) = line
            .split_once('=')
            .expect("dependency table entries have a key and declaration");
        entries.push((name.trim(), declaration.trim()));
    }

    entries
}

#[test]
fn target_specific_normal_dependencies_are_included_in_the_manifest_scan() {
    let manifest =
        "[dependencies]\nfilament-exact = { version = \"1\", default-features = false }\n\
        [target.'cfg(target_arch = \"arm\")'.dependencies]\n\
        filament-semantic-value = { version = \"1\", default-features = false }\n\
        [dev-dependencies]\nexample = \"1\"\n";

    let dependencies = normal_dependency_entries(manifest);
    let names: BTreeSet<&str> = dependencies.iter().map(|(name, _)| *name).collect();

    assert_eq!(
        names,
        BTreeSet::from(["filament-exact", "filament-semantic-value"])
    );
    assert!(dependencies
        .iter()
        .all(|(_, declaration)| declaration.contains("default-features = false")));
}

fn append_rust_sources(directory: &Path, sources: &mut String) {
    for entry in fs::read_dir(directory).expect("read filament-eval source directory") {
        let path = entry.expect("read filament-eval source entry").path();
        if path.is_dir() {
            append_rust_sources(&path, sources);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push_str(&fs::read_to_string(path).expect("read filament-eval Rust source"));
        }
    }
}
