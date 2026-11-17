// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-012 (TC-004): the call depth an evaluation reports.

mod support;

use filament_eval::{Admission, Evaluation, Registry};
use ix_trace_rs::trace;
use support::{boom, count, host_a, host_b, int, int_signature, meter, Counts, Finish, Relay, Toy};

/// `frame_depth` is not a field yet, so the stubs read it from `Debug`.
fn reports_depth(evaluation: &Evaluation, depth: u32) -> bool {
    format!("{evaluation:?}").contains(&format!("frame_depth: {depth}"))
}

#[trace("TC-004", "FR-012-AC-1")]
#[test]
#[ignore = "QSL-2353: Evaluation.frame_depth not implemented yet"]
fn a_completed_call_reports_depth_zero() {
    let mut registry = Registry::new(Toy::default());
    let mut fuel = meter(u64::MAX);
    let done = registry
        .begin_call(count(), vec![int(3)], &(), &mut fuel)
        .expect("admitted call")
        .run()
        .expect("application");
    assert!(reports_depth(&done, 0), "{done:?}");
}

#[trace("TC-004", "FR-012-AC-2")]
#[test]
#[ignore = "QSL-2353: Evaluation.frame_depth not implemented yet"]
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
            .begin_call(callable, vec![int(1)], &(), &mut fuel)
            .expect("admitted call")
            .run()
            .expect("an undefined outcome is an evaluation");
        assert!(reports_depth(&stopped, depth), "{stopped:?}");
    }
}

#[trace("TC-004", "FR-012-AC-3")]
#[test]
#[ignore = "QSL-2353: Evaluation.frame_depth for recursive calls not implemented yet"]
fn a_recursive_undefined_reports_its_depth_at_one_location() {
    // d(1), d(2), d(3) undefined at one location with frame_depth 0, 1, 2;
    // d(0) completes with 0; an expression root calling d(1) reports 1.
    panic!("Evaluation has no frame_depth and the toy closure has no recursive undefined");
}

#[trace("TC-004", "FR-012-AC-4")]
#[test]
#[ignore = "QSL-2353: Evaluation.frame_depth for session-produced outcomes not implemented yet"]
fn a_denied_call_reports_the_calling_frames_depth() {
    // A denied root function.call reports 0; one denied at the call made by
    // the third nested frame reports 2; a live-occurrence stop likewise.
    panic!("Evaluation has no frame_depth");
}
