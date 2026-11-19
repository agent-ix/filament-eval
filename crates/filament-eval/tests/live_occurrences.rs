// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-011 (TC-003): the session's live-occurrence limit.

mod support;

use filament_eval::{
    Admission, Evaluation, FamilyOutcome, Progress, Registry, SessionLimits,
    DEFAULT_LIVE_OCCURRENCES,
};
use filament_exact::{
    form_collection, ChargePoint, CollectionKind, CollectionType, Incomplete, Integer, LimitKind,
    Outcome, Value, ValueType,
};
use ix_trace_rs::trace;
use support::{
    completed_int, count, count_site, host_a, host_b, int, int_signature, meter, repeat, root_site,
    site, spent, Counts, Echo, Expression, Finish, Relay, Sequence, Toy,
};

/// `count(DEEP)` makes DEEP + 1 = 600,001 calls; their frames (two
/// occurrences each) pass the default limit of 1,048,576 at the 524,289th call.
const DEEP: u64 = 600_000;

fn limit(live_occurrences: u64) -> SessionLimits {
    SessionLimits {
        live_occurrences,
        held: 0,
    }
}

/// The live-occurrence incomplete record an evaluation holds, if any.
fn stopped(evaluation: &Evaluation) -> Option<&Incomplete> {
    match &evaluation.outcome {
        FamilyOutcome::Evaluated(Outcome::Incomplete(record))
            if record.limit_kind == LimitKind::LiveOccurrences =>
        {
            Some(record)
        }
        _ => None,
    }
}

/// Assert `evaluation` stopped at the live-occurrence limit with these fields.
fn assert_stopped(evaluation: &Evaluation, limit: u64, consumed: u64, next_charge: u64) {
    let record = stopped(evaluation)
        .unwrap_or_else(|| panic!("expected the live-occurrence incomplete, got {evaluation:?}"));
    assert_eq!(record.limit, limit);
    assert_eq!(record.consumed, consumed);
    assert_eq!(record.next_charge, Integer::from(next_charge));
    assert_eq!(record.charge_point, ChargePoint::FunctionCall);
    assert!(evaluation.losses.is_empty());
}

/// A sequence of four Booleans: `occ` 1 + 4 = 5.
fn occ_five() -> Value {
    form_collection(
        &CollectionType::new(CollectionKind::Sequence, ValueType::Boolean, None),
        vec![Value::Boolean(true); 4],
        &mut meter(10_000),
    )
    .expect("typed sequence")
    .completed()
    .expect("sequence formation")
}

#[trace("TC-003", "FR-011-AC-1")]
#[test]
fn a_deep_recursion_stops_at_the_default_live_occurrence_limit() {
    assert_eq!(DEFAULT_LIVE_OCCURRENCES, 1_048_576);
    assert_eq!(SessionLimits::default(), limit(1_048_576));

    let mut registry = Registry::new(Toy::default());
    let mut unlimited = meter(u64::MAX);
    let evaluation = registry
        .begin_call(
            count(),
            vec![int(DEEP)],
            SessionLimits::default(),
            &(),
            &mut unlimited,
        )
        .expect("admitted call")
        .run()
        .expect("a reached limit is an evaluation, not a failure");
    assert_stopped(&evaluation, 1_048_576, 1_048_576, 2);
    // The 524,289th call applies COUNT to DEEP - 524,288 at its request site.
    assert_eq!(evaluation.location, Some(count_site(DEEP - 524_288)));
    assert_eq!(evaluation.peak_live_occurrences, 1_048_576);
    assert_eq!(
        spent(&unlimited),
        524_289,
        "the stopped call's charge stays spent"
    );

    let mut shallow = meter(u64::MAX);
    let done = registry
        .begin_call(
            count(),
            vec![int(1_000)],
            SessionLimits::default(),
            &(),
            &mut shallow,
        )
        .expect("admitted call")
        .run()
        .expect("application");
    assert_eq!(completed_int(&done), Some(1_000));
    assert_eq!(done.peak_live_occurrences, 2_002);
}

#[trace("TC-003", "FR-011-AC-2")]
#[test]
fn a_raised_limit_completes_and_zero_stops_the_root_call() {
    let mut registry = Registry::new(Toy::default());
    let mut raised = meter(u64::MAX);
    let done = registry
        .begin_call(count(), vec![int(DEEP)], limit(4_000_000), &(), &mut raised)
        .expect("admitted call")
        .run()
        .expect("application");
    assert_eq!(completed_int(&done), Some(DEEP));
    assert_eq!(done.peak_live_occurrences, 1_200_002);

    let toy = Toy::default();
    let binds = toy.binds();
    let mut registry = Registry::new(toy);
    let mut fuel = meter(u64::MAX);
    let evaluation = registry
        .begin_call(count(), vec![int(1)], limit(0), &(), &mut fuel)
        .expect("admitted call")
        .run()
        .expect("a reached limit is an evaluation");
    assert_stopped(&evaluation, 0, 0, 2);
    assert_eq!(evaluation.location, Some(root_site(&count())));
    assert_eq!(evaluation.peak_live_occurrences, 0);
    assert_eq!(spent(&fuel), 1, "the root call's charge stays spent");
    assert!(
        binds.lock().expect("bind log").is_empty(),
        "nothing is bound"
    );
}

#[trace("TC-003", "FR-011-AC-3")]
#[test]
fn returned_frames_leave_the_count() {
    let mut registry = Registry::new(Toy::default());
    let mut fuel = meter(u64::MAX);
    let interpreted = registry
        .begin_call(repeat(), vec![int(10)], limit(4), &(), &mut fuel)
        .expect("admitted call")
        .run()
        .expect("application");
    assert_eq!(completed_int(&interpreted), Some(10));
    assert_eq!(interpreted.peak_live_occurrences, 4);
    assert_eq!(spent(&fuel), 11, "the root and ten sequential calls");

    // A host-body caller and host-body callees count as interpreted ones do.
    let mut registry = Registry::new(Toy::default());
    assert_eq!(
        registry.register(
            host_a(),
            &int_signature(),
            Sequence {
                target: host_b(),
                times: 10,
            },
        ),
        Admission::Accepted
    );
    assert_eq!(
        registry.register(host_b(), &int_signature(), Echo),
        Admission::Accepted
    );
    let mut fuel = meter(u64::MAX);
    let hosted = registry
        .begin_call(host_a(), vec![int(0)], limit(4), &(), &mut fuel)
        .expect("admitted call")
        .run()
        .expect("application");
    assert_eq!(completed_int(&hosted), Some(10));
    assert_eq!(hosted.peak_live_occurrences, 4);

    let mut fuel = meter(u64::MAX);
    let host_root = registry
        .begin_call(host_b(), vec![int(7)], limit(1), &(), &mut fuel)
        .expect("admitted call")
        .run()
        .expect("a reached limit is an evaluation");
    assert_stopped(&host_root, 1, 0, 2);
}

#[trace("TC-003", "FR-011-AC-4")]
#[test]
fn argument_occurrences_and_expression_roots_count() {
    let argument = occ_five();
    assert_eq!(argument.occ(), Integer::from(5_u64));

    let mut registry = Registry::new(Toy::default());
    let mut fuel = meter(u64::MAX);
    let fits = registry
        .begin_call(count(), vec![argument.clone()], limit(6), &(), &mut fuel)
        .expect("admitted call")
        .run()
        .expect("application");
    assert_eq!(fits.peak_live_occurrences, 6);

    let mut fuel = meter(u64::MAX);
    let over = registry
        .begin_call(count(), vec![argument], limit(5), &(), &mut fuel)
        .expect("admitted call")
        .run()
        .expect("a reached limit is an evaluation");
    assert_stopped(&over, 5, 0, 6);

    // Three Integer parameters: the root frame counts 1 + 3 = 4 without a
    // test of its own; its call to COUNT(0) (2 more) exceeds 5.
    let mut fuel = meter(u64::MAX);
    let expression = registry
        .begin_expression(
            Expression::ApplyWith {
                callee: count(),
                argument: 0,
            },
            vec![int(1), int(2), int(3)],
            limit(5),
            &(),
            &mut fuel,
        )
        .expect("admitted expression")
        .run()
        .expect("a reached limit is an evaluation");
    assert_stopped(&expression, 5, 4, 2);
    assert_eq!(expression.location, Some(site()));
    assert_eq!(expression.peak_live_occurrences, 4);
    assert_eq!(spent(&fuel), 1, "the stopped call's charge stays spent");
}

/// `filament-eval` has a single build, `no_std` with `alloc` and no `std`
/// feature (FR-401), so this host test runs the same build the
/// `thumbv7em-none-eabi` target compiles; the count is `u64` and exact
/// `Integer` arithmetic, which is platform independent.
#[trace("TC-003", "FR-011-AC-5")]
#[test]
fn the_limit_is_never_a_failure_and_agrees_without_std() {
    let mut registry = Registry::new(Toy::default());
    let counts = Counts::default();
    assert_eq!(
        registry.register(
            host_a(),
            &int_signature(),
            Relay {
                target: count(),
                finish: Finish::Direct,
                counts: counts.clone(),
            },
        ),
        Admission::Accepted
    );
    // HOST_A's frame (2) fits under 3; its request for COUNT (2 more) does not.
    let mut fuel = meter(u64::MAX);
    let mut session = registry
        .begin_call(host_a(), vec![int(4)], limit(3), &(), &mut fuel)
        .expect("admitted call");
    let terminal = loop {
        match session.step() {
            Progress::Runnable(next) => session = next,
            Progress::Terminal(result) => break result,
        }
    };
    let evaluation = terminal.expect("never a CallFailure");
    assert!(
        !matches!(
            evaluation.outcome,
            FamilyOutcome::Evaluated(Outcome::Refused(_)) | FamilyOutcome::FamilyEvaluated(_)
        ),
        "never a refusal"
    );
    assert_stopped(&evaluation, 3, 2, 2);
    assert_eq!(evaluation.peak_live_occurrences, 2);
    assert_eq!(counts.begun(), 1);
    assert_eq!(
        counts.resumed(),
        1,
        "the stopped call's continuation is never resumed"
    );
}
