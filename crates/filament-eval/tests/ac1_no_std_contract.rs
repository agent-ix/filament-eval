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
    let sources = strip_rust_comments_and_literals(&sources);
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

    for line in manifest.lines().map(strip_toml_comment).map(str::trim) {
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
                let declaration = strip_toml_comment(declaration).trim();
                entries.push(NormalDependency {
                    name: unquote(name.trim()).to_owned(),
                    default_features_disabled: declaration.contains("default-features = false"),
                });
            }
            DependencyTable::Nested(_) => {
                let (key, value) = line
                    .split_once('=')
                    .expect("nested dependency fields have a key and value");
                let value = strip_toml_comment(value).trim();
                if key.trim() == "default-features" && value == "false" {
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

fn strip_toml_comment(line: &str) -> &str {
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in line.char_indices() {
        if let Some(delimiter) = quote {
            if delimiter == '"' && escaped {
                escaped = false;
            } else if delimiter == '"' && character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
        } else if character == '"' || character == '\'' {
            quote = Some(character);
        } else if character == '#' {
            return &line[..index];
        }
    }
    line
}

#[test]
fn target_specific_normal_dependencies_are_included_in_the_manifest_scan() {
    let manifest = "[dependencies]\ninline-crate = { version = \"1\", marker = \"value#part\", default-features = true } # default-features = false\n\
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
        .filter(|dependency| dependency.name != "inline-crate")
        .all(|dependency| dependency.default_features_disabled));
    assert!(dependencies
        .iter()
        .find(|dependency| dependency.name == "inline-crate")
        .is_some_and(|dependency| !dependency.default_features_disabled));
}

#[test]
fn rust_dependency_scan_ignores_line_nested_block_and_string_comments() {
    let source = r#"
        // filament_exact::LineComment
        /* filament_semantic_value::Outer /* thiserror::Nested */ */
        const URL: &str = "filament_exact::StringLiteral";
        use filament_exact::ActualItem;
    "#;
    let stripped = strip_rust_comments_and_literals(source);

    assert!(stripped.contains("filament_exact::ActualItem"));
    assert!(!stripped.contains("filament_exact::LineComment"));
    assert!(!stripped.contains("filament_semantic_value::Outer"));
    assert!(!stripped.contains("thiserror::Nested"));
    assert!(!stripped.contains("filament_exact::StringLiteral"));
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

fn strip_rust_comments_and_literals(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut stripped = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if let Some(end) = raw_string_end(bytes, index) {
            stripped.push(b' ');
            index = end;
        } else if bytes[index..].starts_with(b"//") {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
        } else if bytes[index..].starts_with(b"/*") {
            index = skip_block_comment(bytes, index);
            stripped.push(b' ');
        } else if bytes[index] == b'"' {
            index = skip_quoted(bytes, index, b'"');
            stripped.push(b' ');
        } else if bytes[index] == b'\'' && char_literal_end(bytes, index).is_some() {
            index = char_literal_end(bytes, index).expect("checked character literal");
            stripped.push(b' ');
        } else {
            stripped.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(stripped).expect("comment stripping preserves UTF-8")
}

fn raw_string_end(bytes: &[u8], start: usize) -> Option<usize> {
    let raw_start = if bytes[start..].starts_with(b"br") {
        start + 1
    } else if bytes[start..].starts_with(b"r") {
        start
    } else {
        return None;
    };
    let mut quote = raw_start + 1;
    while quote < bytes.len() && bytes[quote] == b'#' {
        quote += 1;
    }
    if quote >= bytes.len() || bytes[quote] != b'"' {
        return None;
    }
    let hashes = quote - raw_start - 1;
    let mut cursor = quote + 1;
    while cursor < bytes.len() {
        if bytes[cursor] == b'"'
            && bytes
                .get(cursor + 1..cursor + 1 + hashes)
                .is_some_and(|suffix| suffix.iter().all(|byte| *byte == b'#'))
        {
            return Some(cursor + 1 + hashes);
        }
        cursor += 1;
    }
    Some(bytes.len())
}

fn skip_block_comment(bytes: &[u8], start: usize) -> usize {
    let mut depth = 1_usize;
    let mut cursor = start + 2;
    while cursor < bytes.len() && depth > 0 {
        if bytes[cursor..].starts_with(b"/*") {
            depth += 1;
            cursor += 2;
        } else if bytes[cursor..].starts_with(b"*/") {
            depth -= 1;
            cursor += 2;
        } else {
            cursor += 1;
        }
    }
    cursor
}

fn skip_quoted(bytes: &[u8], start: usize, delimiter: u8) -> usize {
    let mut cursor = start + 1;
    let mut escaped = false;
    while cursor < bytes.len() {
        let byte = bytes[cursor];
        cursor += 1;
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == delimiter {
            break;
        }
    }
    cursor
}

fn char_literal_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut cursor = start + 1;
    if cursor >= bytes.len() || bytes[cursor] == b'\n' {
        return None;
    }
    if bytes[cursor] == b'\\' {
        cursor += 1;
        while cursor < bytes.len() && bytes[cursor] != b'\'' && bytes[cursor] != b'\n' {
            cursor += 1;
        }
    } else {
        cursor += 1;
        while cursor < bytes.len() && (bytes[cursor] & 0b1100_0000) == 0b1000_0000 {
            cursor += 1;
        }
    }
    (cursor < bytes.len() && bytes[cursor] == b'\'').then_some(cursor + 1)
}
