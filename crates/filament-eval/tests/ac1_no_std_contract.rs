// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-401-AC-1: the evaluation leaf stays no_std and depends only on its
//! three used, no-default-feature support crates.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use ix_trace_rs::trace;
use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::{Meta, Token};

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

    let mut sources = Vec::new();
    append_rust_sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut sources,
    );
    for (dependency, item_prefix) in [
        ("filament-exact", "filament_exact"),
        ("filament-semantic-value", "filament_semantic_value"),
        ("thiserror", "thiserror"),
    ] {
        assert!(
            source_uses_dependency_item(&sources, item_prefix),
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
                entries.push(NormalDependency {
                    name: unquote(name.trim()).to_owned(),
                    default_features_disabled: inline_default_features_disabled(declaration),
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

fn inline_default_features_disabled(declaration: &str) -> bool {
    let declaration = strip_toml_comment(declaration).trim();
    let Some(fields) = declaration
        .strip_prefix('{')
        .and_then(|table| table.strip_suffix('}'))
    else {
        return false;
    };

    split_inline_table_fields(fields).into_iter().any(|field| {
        field
            .split_once('=')
            .is_some_and(|(key, value)| key.trim() == "default-features" && value.trim() == "false")
    })
}

fn split_inline_table_fields(fields: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    let mut nesting = 0_usize;
    let mut start = 0;

    for (index, character) in fields.char_indices() {
        if let Some(delimiter) = quote {
            if delimiter == '"' && escaped {
                escaped = false;
            } else if delimiter == '"' && character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }
        } else {
            match character {
                '"' | '\'' => quote = Some(character),
                '[' | '{' => nesting += 1,
                ']' | '}' => nesting = nesting.saturating_sub(1),
                ',' if nesting == 0 => {
                    result.push(fields[start..index].trim());
                    start = index + character.len_utf8();
                }
                _ => {}
            }
        }
    }
    result.push(fields[start..].trim());
    result
}

#[test]
fn target_specific_normal_dependencies_are_included_in_the_manifest_scan() {
    let manifest = "[dependencies]\ninline-crate = { version = \"1\", path = \"default-features = false\", marker = \"value#part\" } # default-features = false\n\
        inline-off = { version = \"1\", default-features = false }\n\
        [dependencies.filament-exact]\nversion = \"1\"\ndefault-features = false\n\
        [dependencies.path-nested]\nversion = \"1\"\npath = \"default-features = false\"\n\
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
            "inline-off",
            "path-nested",
        ])
    );
    assert!(dependencies
        .iter()
        .filter(|dependency| {
            !matches!(dependency.name.as_str(), "inline-crate" | "path-nested")
        })
        .all(|dependency| dependency.default_features_disabled));
    assert!(dependencies
        .iter()
        .find(|dependency| dependency.name == "inline-crate")
        .is_some_and(|dependency| !dependency.default_features_disabled));
    assert!(dependencies
        .iter()
        .find(|dependency| dependency.name == "path-nested")
        .is_some_and(|dependency| !dependency.default_features_disabled));
}

#[test]
fn rust_dependency_scan_ignores_comments_literals_and_local_names() {
    let false_source = r#"
        // filament_exact::LineComment
        /* filament_semantic_value::Outer /* thiserror::Nested */ */
        const URL: &str = "filament_exact::StringLiteral";
        mod filament_exact {
            pub struct LocalItem;
        }
        use filament_exact::LocalItem;
    "#;
    let actual_source = "use filament_exact::ActualItem;";
    let local_alias = "use local::Module as filament_exact; use filament_exact::FakeItem;";
    let block_local_alias =
        "fn nested() { use local::Module as filament_exact; use filament_exact::FakeItem; }";
    let unrelated_local = "mod child { mod filament_exact {} }";
    let unrelated_glob = "use other::*; use filament_exact::ActualItem;";

    assert!(!source_uses_dependency_item(
        &[false_source.to_owned()],
        "filament_exact"
    ));
    assert!(!source_uses_dependency_item(
        &[local_alias.to_owned()],
        "filament_exact"
    ));
    assert!(!source_uses_dependency_item(
        &[block_local_alias.to_owned()],
        "filament_exact"
    ));
    assert!(source_uses_dependency_item(
        &[
            actual_source.to_owned(),
            unrelated_local.to_owned(),
            unrelated_glob.to_owned(),
        ],
        "filament_exact"
    ));
}

fn append_rust_sources(directory: &Path, sources: &mut Vec<String>) {
    for entry in fs::read_dir(directory).expect("read filament-eval source directory") {
        let path = entry.expect("read filament-eval source entry").path();
        if path.is_dir() {
            append_rust_sources(&path, sources);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            sources.push(fs::read_to_string(path).expect("read filament-eval Rust source"));
        }
    }
}

struct DependencyPathVisitor<'a> {
    dependency: &'a str,
    path_found: bool,
}

impl<'ast> Visit<'ast> for DependencyPathVisitor<'_> {
    fn visit_use_path(&mut self, path: &'ast syn::UsePath) {
        if path.ident == self.dependency {
            self.path_found = true;
        }
        syn::visit::visit_use_path(self, path);
    }

    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        if let Meta::List(list) = &attribute.meta {
            if list.path.is_ident("derive") {
                if let Ok(DerivePaths(derives)) = syn::parse2(list.tokens.clone()) {
                    if derives.iter().any(|path| {
                        path.segments
                            .first()
                            .is_some_and(|segment| segment.ident == self.dependency)
                    }) {
                        self.path_found = true;
                    }
                }
            }
        }
        syn::visit::visit_attribute(self, attribute);
    }

    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path.segments.len() > 1
            && path
                .segments
                .first()
                .is_some_and(|segment| segment.ident == self.dependency)
        {
            self.path_found = true;
        }
        syn::visit::visit_path(self, path);
    }

    fn visit_block(&mut self, block: &'ast syn::Block) {
        if block_uses_dependency_item(block, self.dependency) {
            self.path_found = true;
        }
    }
}

struct DerivePaths(Punctuated<syn::Path, Token![,]>);

impl syn::parse::Parse for DerivePaths {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        Punctuated::parse_terminated(input).map(Self)
    }
}

fn source_uses_dependency_item(sources: &[String], dependency: &str) -> bool {
    sources.iter().any(|source| {
        let file = syn::parse_file(source).expect("parse filament-eval Rust source");
        scope_uses_dependency_item(&file.items, dependency)
    })
}

fn scope_uses_dependency_item(items: &[syn::Item], dependency: &str) -> bool {
    if scope_is_shadowed(items, dependency) {
        return false;
    }

    let mut visitor = DependencyPathVisitor {
        dependency,
        path_found: false,
    };
    for item in items {
        if let syn::Item::Mod(module) = item {
            if let Some((_, nested_items)) = &module.content {
                if scope_uses_dependency_item(nested_items, dependency) {
                    return true;
                }
            }
        } else {
            visitor.visit_item(item);
        }
    }
    visitor.path_found
}

fn block_uses_dependency_item(block: &syn::Block, dependency: &str) -> bool {
    let shadowed = block.stmts.iter().any(|statement| match statement {
        syn::Stmt::Item(syn::Item::Mod(item)) => item.ident == dependency,
        syn::Stmt::Item(syn::Item::Use(item)) => {
            !use_tree_is_rooted_at(&item.tree, dependency)
                && use_tree_binds_name(&item.tree, dependency, None)
        }
        _ => false,
    });
    if shadowed {
        return false;
    }

    let mut visitor = DependencyPathVisitor {
        dependency,
        path_found: false,
    };
    for statement in &block.stmts {
        if let syn::Stmt::Item(syn::Item::Mod(module)) = statement {
            if let Some((_, nested_items)) = &module.content {
                if scope_uses_dependency_item(nested_items, dependency) {
                    return true;
                }
            }
        } else {
            visitor.visit_stmt(statement);
        }
    }
    visitor.path_found
}

fn scope_is_shadowed(items: &[syn::Item], dependency: &str) -> bool {
    items.iter().any(|item| match item {
        syn::Item::Mod(item) => item.ident == dependency,
        syn::Item::Use(item) => {
            !use_tree_is_rooted_at(&item.tree, dependency)
                && use_tree_binds_name(&item.tree, dependency, None)
        }
        _ => false,
    })
}

fn use_tree_is_rooted_at(tree: &syn::UseTree, dependency: &str) -> bool {
    matches!(tree, syn::UseTree::Path(path) if path.ident == dependency)
}

fn use_tree_binds_name(tree: &syn::UseTree, dependency: &str, parent: Option<&str>) -> bool {
    match tree {
        syn::UseTree::Path(path) => {
            use_tree_binds_name(&path.tree, dependency, Some(&path.ident.to_string()))
        }
        syn::UseTree::Name(name) => {
            name.ident == dependency || (name.ident == "self" && parent == Some(dependency))
        }
        syn::UseTree::Rename(rename) => rename.rename == dependency,
        syn::UseTree::Glob(_) => false,
        syn::UseTree::Group(group) => group
            .items
            .iter()
            .any(|item| use_tree_binds_name(item, dependency, parent)),
    }
}
