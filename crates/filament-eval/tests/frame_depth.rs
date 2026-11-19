// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-012 (TC-004): the call depth an evaluation reports.

mod support;

use filament_eval::{Admission, Evaluation, Registry, SessionLimits};
use ix_trace_rs::trace;
use support::{
    boom, completed_depth, count, host_a, host_b, int, int_signature, meter, recursive_undefined,
    Counts, Finish, Relay, Toy,
};

fn reports_depth(evaluation: &Evaluation, depth: u32) -> bool {
    evaluation.frame_depth == depth
}

#[trace("TC-004", "FR-012-AC-1")]
#[test]
fn a_completed_call_reports_depth_zero() {
    let mut registry = Registry::new(Toy::default());
    let mut fuel = meter(u64::MAX);
    let done = registry
        .begin_call(
            count(),
            vec![int(3)],
            SessionLimits::default(),
            &(),
            &mut fuel,
        )
        .expect("admitted call")
        .run()
        .expect("application");
    assert!(reports_depth(&done, 0), "{done:?}");
}

#[trace("TC-004", "FR-012-AC-1")]
#[test]
fn a_root_completed_outcome_is_normalized_to_zero() {
    let mut registry = Registry::new(Toy::default());
    let mut fuel = meter(u64::MAX);
    let done = registry
        .begin_call(
            completed_depth(),
            vec![int(1)],
            SessionLimits::default(),
            &(),
            &mut fuel,
        )
        .expect("admitted call")
        .run()
        .expect("application");
    assert!(reports_depth(&done, 0), "{done:?}");
}

#[trace("TC-004", "FR-012-AC-2")]
#[test]
fn a_relayed_undefined_reports_the_frames_above_the_selected_call() {
    let mut registry = Registry::new(Toy::default());
    for (callable, target) in [(host_a(), host_b()), (host_b(), boom())] {
        assert_eq!(
            registry.register(
                callable,
                &int_signature(),
                Relay {
                    target,
                    finish: Finish::Direct,
                    counts: Counts::default(),
                },
            ),
            Admission::Accepted
        );
    }
    for (callable, depth) in [(boom(), 0), (host_b(), 1), (host_a(), 2)] {
        let mut fuel = meter(u64::MAX);
        let stopped = registry
            .begin_call(
                callable,
                vec![int(1)],
                SessionLimits::default(),
                &(),
                &mut fuel,
            )
            .expect("admitted call")
            .run()
            .expect("an undefined outcome is an evaluation");
        assert!(reports_depth(&stopped, depth), "{stopped:?}");
    }
}

#[trace("TC-004", "FR-012-AC-3")]
#[test]
fn a_recursive_undefined_reports_its_depth_at_one_location() {
    let mut registry = Registry::new(Toy::default());
    for (argument, depth) in [(1, 0), (2, 1), (3, 2)] {
        let mut fuel = meter(u64::MAX);
        let evaluation = registry
            .begin_call(
                recursive_undefined(),
                vec![int(argument)],
                SessionLimits::default(),
                &(),
                &mut fuel,
            )
            .expect("admitted call")
            .run()
            .expect("undefined is an evaluation");
        assert!(reports_depth(&evaluation, depth), "{evaluation:?}");
        assert_eq!(evaluation.location, Some(support::site()));
    }

    let mut fuel = meter(u64::MAX);
    let expression = registry
        .begin_expression(
            support::Expression::Apply {
                callee: recursive_undefined(),
                argument: 1,
            },
            Vec::new(),
            SessionLimits::default(),
            &(),
            &mut fuel,
        )
        .expect("admitted expression")
        .run()
        .expect("undefined is an evaluation");
    assert!(reports_depth(&expression, 1), "{expression:?}");
}

#[trace("TC-004", "FR-012-AC-4")]
#[test]
fn a_denied_call_reports_the_calling_frames_depth() {
    let mut registry = Registry::new(Toy::default());
    let mut root_fuel = meter(0);
    let root = registry
        .begin_call(
            count(),
            vec![int(3)],
            SessionLimits::default(),
            &(),
            &mut root_fuel,
        )
        .expect("admitted call")
        .run()
        .expect("denial is an incomplete evaluation");
    assert!(reports_depth(&root, 0), "{root:?}");

    let mut nested_fuel = meter(3);
    let nested = registry
        .begin_call(
            recursive_undefined(),
            vec![int(5)],
            SessionLimits::default(),
            &(),
            &mut nested_fuel,
        )
        .expect("admitted call")
        .run()
        .expect("denial is an incomplete evaluation");
    assert!(reports_depth(&nested, 2), "{nested:?}");
}
