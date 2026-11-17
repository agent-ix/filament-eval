// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-011 (TC-003): the session's live-occurrence limit.

mod support;

use filament_eval::{FamilyOutcome, Registry};
use filament_exact::{ChargePoint, Outcome};
use ix_trace_rs::trace;
use support::{completed_int, count, int, meter, spent, Toy};

/// `count(DEEP)` makes DEEP + 1 = 600,001 calls; their frames (two
/// occurrences each) pass the default limit of 1,048,576 at the 524,289th call.
const DEEP: u64 = 600_000;

#[trace("TC-003", "FR-011-AC-1")]
#[test]
#[ignore = "QSL-2326: the session live-occurrence limit (LimitKind::LiveOccurrences) not implemented yet"]
fn a_deep_recursion_stops_at_the_default_live_occurrence_limit() {
    let mut registry = Registry::new(Toy::default());
    let mut unlimited = meter(u64::MAX);
    let stopped = registry
        .begin_call(count(), vec![int(DEEP)], &(), &mut unlimited)
        .expect("admitted call")
        .run()
        .expect("a reached limit is an evaluation, not a failure");
    match &stopped.outcome {
        FamilyOutcome::Evaluated(Outcome::Incomplete(denied)) => {
            assert_eq!(format!("{:?}", denied.limit_kind), "LiveOccurrences");
            assert_eq!(denied.limit, 1_048_576);
            assert_eq!(denied.consumed, 1_048_576);
            assert_eq!(denied.charge_point, ChargePoint::FunctionCall);
        }
        other => panic!("expected the live-occurrence incomplete, got {other:?}"),
    }
    assert_eq!(spent(&unlimited), 524_289, "the stopped call's charge stays spent");

    let mut shallow = meter(u64::MAX);
    let done = registry
        .begin_call(count(), vec![int(1_000)], &(), &mut shallow)
        .expect("admitted call")
        .run()
        .expect("application");
    assert_eq!(completed_int(&done), Some(1_000));
}

#[trace("TC-003", "FR-011-AC-2")]
#[test]
#[ignore = "QSL-2326: SessionLimits on begin_call and Evaluation.peak_live_occurrences not implemented yet"]
fn a_raised_limit_completes_and_zero_stops_the_root_call() {
    // begin_call(.., SessionLimits { live_occurrences: 4_000_000 }) of DEEP
    // completes with peak_live_occurrences 1_200_002; live_occurrences 0
    // stops the root call with consumed 0 and next_charge 2.
    panic!("begin_call takes no SessionLimits");
}

#[trace("TC-003", "FR-011-AC-3")]
#[test]
#[ignore = "QSL-2326: frames leaving the live-occurrence count not implemented yet"]
fn returned_frames_leave_the_count() {
    // Ten sequential one-argument calls under live_occurrences 4 complete
    // with peak_live_occurrences 4, for interpreted and host-body callees.
    panic!("the session keeps no live-occurrence count");
}

#[trace("TC-003", "FR-011-AC-4")]
#[test]
#[ignore = "QSL-2326: occ-weighted frame counting and expression-root frames not implemented yet"]
fn argument_occurrences_and_expression_roots_count() {
    // An argument of occ 5 counts 6; an expression root with parameters of
    // occ 3 calling a one-argument callee under 5 stops with consumed 4 and
    // next_charge 2.
    panic!("the session keeps no live-occurrence count");
}

#[trace("TC-003", "FR-011-AC-5")]
#[test]
#[ignore = "QSL-2326: the live-occurrence stop as incomplete only, identical without std, not implemented yet"]
fn the_limit_is_never_a_failure_and_agrees_without_std() {
    // The stop is an Evaluation (never CallFailure, refusal or fault), the
    // stopped continuation never resumes, and the no-std build agrees.
    panic!("the session keeps no live-occurrence count");
}
