// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-400-AC-4's default-feature constructor boundary.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use ix_trace_rs::trace;

#[test]
#[trace("FR-400-AC-4")]
fn default_features_leave_checked_constructors_disabled() {
    assert!(
        !cfg!(feature = "construct"),
        "default features must not enable checked-term construction"
    );
}

/// The checked crate must not expose the checker-owned derivation vocabulary,
/// derivation methods on its declaration key, or the model refusal cause.
#[trace(
    "FR-400-AC-18",
    "FR-400-AC-19",
    "FR-400-AC-20",
    "FR-400-AC-21",
    "FR-400-AC-22",
    "FR-400-AC-23"
)]
#[test]
fn checker_owned_derivation_api_is_not_public_from_filament_checked() {
    let absent_types = [
        (
            "FR-400-AC-18",
            "EffectiveDeclarationPreimage",
            &["", "key", "node"][..],
        ),
        ("FR-400-AC-19", "Fact", &["", "key", "node"][..]),
        ("FR-400-AC-20", "RuleRef", &["", "key", "node"][..]),
        (
            "FR-400-AC-23",
            "ModelRefusalCause",
            &["", "key", "node"][..],
        ),
    ];

    for (criterion, item, modules) in absent_types {
        for module in modules {
            let path = if module.is_empty() {
                format!("filament_checked::{item}")
            } else {
                format!("filament_checked::{module}::{item}")
            };
            let source = format!("use {path};\nfn main() {{ let _ = {item}; }}\n");
            let probe = format!("{criterion}-{module}");
            assert_unavailable_from_external_crate(&probe, &source, item);
        }
    }

    let absent_methods = [
        (
            "FR-400-AC-21",
            "identity",
            "use filament_checked::key::DeclarationKey;\nfn main() { let key = DeclarationKey { package: String::new(), node: String::new() }; let _ = key.identity(); }\n",
        ),
        (
            "FR-400-AC-22",
            "validate_derivation",
            "use filament_checked::key::DeclarationKey;\nfn main() { let key = DeclarationKey { package: String::new(), node: String::new() }; let _ = key.validate_derivation(); }\n",
        ),
    ];

    for (criterion, method, source) in absent_methods {
        assert_unavailable_from_external_crate(criterion, source, method);
    }
    assert_no_model_refusal_cause_item_in_crate_source();
}

fn assert_unavailable_from_external_crate(criterion: &str, source: &str, absent_item: &str) {
    let test_executable = std::env::current_exe().expect("integration test executable path");
    let dependencies = test_executable
        .parent()
        .expect("integration test executable has a deps directory");
    let checked_library = fs::read_dir(dependencies)
        .expect("read Cargo deps directory")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with("libfilament_checked-") && name.ends_with(".rlib")
                })
        })
        .expect("Cargo built filament-checked as an rlib");

    let source_path = compile_fail_source_path(criterion);
    let output_path = source_path.with_extension("output");
    fs::write(&source_path, source).expect("write compile-fail source");
    let output = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-type=bin")
        .arg(&source_path)
        .arg("-L")
        .arg(format!("dependency={}", dependencies.display()))
        .arg("--extern")
        .arg(format!("filament_checked={}", checked_library.display()))
        .arg("-o")
        .arg(&output_path)
        .output()
        .expect("run rustc for public-surface probe");
    let _ = fs::remove_file(&source_path);
    let _ = fs::remove_file(&output_path);

    let diagnostics = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "{criterion}: {absent_item} unexpectedly compiled from an external crate"
    );
    assert!(
        diagnostics.contains(absent_item),
        "{criterion}: compilation failed for a reason unrelated to {absent_item}: {diagnostics}"
    );
}

fn compile_fail_source_path(criterion: &str) -> PathBuf {
    let safe_criterion = criterion.replace('-', "_");
    std::env::temp_dir().join(format!(
        "filament_checked_{safe_criterion}_{}.rs",
        std::process::id()
    ))
}

fn assert_no_model_refusal_cause_item_in_crate_source() {
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut pending = vec![source_root];

    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(path).expect("read filament-checked source directory") {
            let entry = entry.expect("read filament-checked source entry");
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let source = fs::read_to_string(&path).expect("read Rust source");
                assert!(
                    !source.contains("ModelRefusalCause"),
                    "FR-400-AC-23: public crate source names ModelRefusalCause in {}",
                    path.display()
                );
            }
        }
    }
}
