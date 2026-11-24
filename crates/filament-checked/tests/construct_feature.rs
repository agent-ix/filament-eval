// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-400-AC-4's default-feature constructor boundary.

use ix_trace_rs::trace;

#[test]
#[trace("FR-400-AC-4")]
fn default_features_leave_checked_constructors_disabled() {
    assert!(
        !cfg!(feature = "construct"),
        "default features must not enable checked-term construction"
    );
}
