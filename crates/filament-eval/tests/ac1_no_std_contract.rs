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
    let dependency_names: BTreeSet<&str> = dependencies
        .iter()
        .map(|dependency| dependency.name.as_str())
        .collect();
    assert_eq!(
        dependency_names,
        BTreeSet::from(["filament-exact", "filament-semantic-value", "thiserror"]),
        "only the leaf's directly used support crates are normal dependencies"
    );
    for dependency in dependencies {
        assert!(
            dependency.default_features_disabled,
            "normal dependency {} must disable default features",
            dependency.name
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

struct NormalDependency {
    name: String,
    default_features_disabled: bool,
}

#[derive(Clone, Copy)]
enum DependencyTable<'a> {
    Other,
    Entries,
    Nested(&'a str),
}

fn normal_dependency_entries(manifest: &str) -> Vec<NormalDependency> {
    let mut table = DependencyTable::Other;
    let mut entries = Vec::new();
    let mut nested_default_features_disabled = false;

    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            if let DependencyTable::Nested(name) = table {
                entries.push(NormalDependency {
                    name: name.to_owned(),
                    default_features_disabled: nested_default_features_disabled,
                });
            }
            nested_default_features_disabled = false;
            let header = line
                .strip_prefix('[')
                .and_then(|header| header.strip_suffix(']'))
                .expect("TOML table header is bracketed");
            table = dependency_table(header);
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        match table {
            DependencyTable::Other => {}
            DependencyTable::Entries => {
                let (name, declaration) = line
                    .split_once('=')
                    .expect("dependency table entries have a key and declaration");
                entries.push(NormalDependency {
                    name: unquote(name.trim()).to_owned(),
                    default_features_disabled: declaration.contains("default-features = false"),
                });
            }
            DependencyTable::Nested(_) => {
                let (key, value) = line
                    .split_once('=')
                    .expect("nested dependency fields have a key and value");
                if key.trim() == "default-features" && value.trim() == "false" {
                    nested_default_features_disabled = true;
                }
            }
        }
    }
    if let DependencyTable::Nested(name) = table {
        entries.push(NormalDependency {
            name: name.to_owned(),
            default_features_disabled: nested_default_features_disabled,
        });
    }

    entries
}

fn dependency_table(header: &str) -> DependencyTable<'_> {
    if header == "dependencies" {
        return DependencyTable::Entries;
    }
    if let Some(name) = header.strip_prefix("dependencies.") {
        return DependencyTable::Nested(unquote(name));
    }
    if let Some(name) = target_dependency_suffix(header) {
        return if name.is_empty() {
            DependencyTable::Entries
        } else {
            DependencyTable::Nested(unquote(name))
        };
    }
    DependencyTable::Other
}

fn target_dependency_suffix(header: &str) -> Option<&str> {
    if !header.starts_with("target.") {
        return None;
    }
    let (_, suffix) = header.rsplit_once(".dependencies")?;
    if suffix.is_empty() {
        Some("")
    } else {
        suffix.strip_prefix('.')
    }
}

fn unquote(name: &str) -> &str {
    name.trim_matches(|character| character == '\'' || character == '"')
}

#[test]
fn target_specific_normal_dependencies_are_included_in_the_manifest_scan() {
    let manifest = "[dependencies]\ninline-crate = { version = \"1\", default-features = false }\n\
        [dependencies.filament-exact]\nversion = \"1\"\ndefault-features = false\n\
        [target.'cfg(target_arch = \"arm\")'.dependencies]\n\
        filament-semantic-value = { version = \"1\", default-features = false }\n\
        [target.'cfg(target_arch = \"arm\")'.dependencies.fsl-forbidden]\n\
        version = \"1\"\ndefault-features = false\n\
        [dev-dependencies]\nexample = \"1\"\n";

    let dependencies = normal_dependency_entries(manifest);
    let names: BTreeSet<&str> = dependencies
        .iter()
        .map(|dependency| dependency.name.as_str())
        .collect();

    assert_eq!(
        names,
        BTreeSet::from([
            "filament-exact",
            "filament-semantic-value",
            "fsl-forbidden",
            "inline-crate",
        ])
    );
    assert!(dependencies
        .iter()
        .all(|dependency| dependency.default_features_disabled));
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
