// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-013 (TC-005): one live-occurrence count across transitions and sessions.

mod support;

use std::sync::{Arc, Mutex};

use filament_eval::{
    Admission, Body, BodyFactory, BodyFailure, Evaluation, FamilyOutcome, LiveContext, Registry,
    Resume, SessionLimits, Transition,
};
use filament_exact::{Charge, ChargePoint, Incomplete, Integer, LimitKind, Meter, Outcome, Value};
use ix_trace_rs::trace;
use support::{
    add, completed_int, count, int, live_child_three, live_child_two, live_done_three,
    live_done_two, meter, native_nested_fit, native_nested_over, nested_fit_child,
    nested_over_child, signature_of_arity, site, spent, Toy,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct ContextRead {
    live: u64,
    limit: u64,
    peak: u64,
    nested_held: u64,
}

enum HostAction {
    Call {
        callable: filament_exact::NodeKey,
        arguments: Vec<Value>,
    },
    CallWithCarries {
        callable: filament_exact::NodeKey,
        arguments: Vec<Value>,
    },
    Carry,
    Nested {
        callable: filament_exact::NodeKey,
        arguments: Vec<Value>,
    },
    NestedAfterOpen {
        occurrences: u64,
        callable: filament_exact::NodeKey,
        arguments: Vec<Value>,
    },
}

struct TestBodyFactory {
    action: HostAction,
    reads: Arc<Mutex<Vec<ContextRead>>>,
    resumes: Arc<Mutex<Vec<&'static str>>>,
}

impl BodyFactory for TestBodyFactory {
    fn begin(&self) -> Box<dyn Body> {
        Box::new(TestBody {
            action: match &self.action {
                HostAction::Call {
                    callable,
                    arguments,
                } => HostAction::Call {
                    callable: callable.clone(),
                    arguments: arguments.clone(),
                },
                HostAction::CallWithCarries {
                    callable,
                    arguments,
                } => HostAction::CallWithCarries {
                    callable: callable.clone(),
                    arguments: arguments.clone(),
                },
                HostAction::Carry => HostAction::Carry,
                HostAction::Nested {
                    callable,
                    arguments,
                } => HostAction::Nested {
                    callable: callable.clone(),
                    arguments: arguments.clone(),
                },
                HostAction::NestedAfterOpen {
                    occurrences,
                    callable,
                    arguments,
                } => HostAction::NestedAfterOpen {
                    occurrences: *occurrences,
                    callable: callable.clone(),
                    arguments: arguments.clone(),
                },
            },
            reads: Arc::clone(&self.reads),
            resumes: Arc::clone(&self.resumes),
            carried: None,
            carried_many: Vec::new(),
        })
    }
}

struct TestBody {
    action: HostAction,
    reads: Arc<Mutex<Vec<ContextRead>>>,
    resumes: Arc<Mutex<Vec<&'static str>>>,
    carried: Option<u64>,
    carried_many: Vec<u64>,
}

impl TestBody {
    fn record(&self, context: &LiveContext) {
        self.reads.lock().expect("context reads").push(ContextRead {
            live: context.live(),
            limit: context.limit(),
            peak: context.peak(),
            nested_held: context.nested().held,
        });
    }
}

impl Body for TestBody {
    fn resume(
        &mut self,
        input: Resume,
        context: &mut LiveContext,
        meter: &mut Meter,
    ) -> Result<Transition, BodyFailure> {
        self.record(context);
        match (&self.action, input) {
            (
                HostAction::Call {
                    callable,
                    arguments,
                },
                Resume::Start(_),
            ) => {
                self.resumes.lock().expect("resume log").push("start");
                Ok(Transition::Call(filament_eval::ChildRequest {
                    callee: callable.clone(),
                    arguments: arguments.clone(),
                    site: site(),
                }))
            }
            (HostAction::Call { .. }, Resume::Child(child)) => {
                self.resumes.lock().expect("resume log").push("child");
                Ok(Transition::Done(*child))
            }
            (
                HostAction::CallWithCarries {
                    callable,
                    arguments,
                },
                Resume::Start(_),
            ) => {
                self.resumes.lock().expect("resume log").push("start");
                for _ in 0..2 {
                    meter
                        .charge(Charge::new(ChargePoint::FunctionCall))
                        .expect("the test meter admits the in-transition call");
                    self.carried_many.push(
                        context
                            .open(&Integer::from(4_u64))
                            .expect("each carried frame fits"),
                    );
                }
                self.record(context);
                Ok(Transition::Call(filament_eval::ChildRequest {
                    callee: callable.clone(),
                    arguments: arguments.clone(),
                    site: site(),
                }))
            }
            (HostAction::CallWithCarries { .. }, Resume::Child(child)) => {
                self.resumes.lock().expect("resume log").push("child");
                Ok(Transition::Done(*child))
            }
            (HostAction::Carry, Resume::Start(_)) => {
                self.resumes.lock().expect("resume log").push("start");
                self.carried = Some(
                    context
                        .open(&Integer::from(4_u64))
                        .expect("the carried frame fits"),
                );
                Ok(Transition::Continue)
            }
            (HostAction::Carry, Resume::Again) => {
                self.resumes.lock().expect("resume log").push("again");
                context.close(self.carried.take().expect("the frame was opened"));
                Ok(Transition::Done(Evaluation::completed(int(0))))
            }
            (
                HostAction::Nested {
                    callable,
                    arguments,
                },
                Resume::Start(_),
            ) => {
                self.resumes.lock().expect("resume log").push("start");
                let limits = context.nested();
                let mut nested_registry = Registry::new(Toy::default());
                let nested = nested_registry
                    .begin_call(callable.clone(), arguments.clone(), limits, &(), meter)
                    .expect("nested call is admitted")
                    .run()
                    .expect("nested application");
                context.reach(nested.peak_live_occurrences);
                Ok(Transition::Done(nested))
            }
            (
                HostAction::NestedAfterOpen {
                    occurrences,
                    callable,
                    arguments,
                },
                Resume::Start(_),
            ) => {
                self.resumes.lock().expect("resume log").push("start");
                meter
                    .charge(Charge::new(ChargePoint::FunctionCall))
                    .expect("the test meter admits the in-transition call");
                let opened = context
                    .open(&Integer::from(*occurrences))
                    .expect("the held in-transition frame fits");
                self.record(context);
                let mut nested_registry = Registry::new(Toy::default());
                let nested = nested_registry
                    .begin_call(
                        callable.clone(),
                        arguments.clone(),
                        context.nested(),
                        &(),
                        meter,
                    )
                    .expect("nested call is admitted")
                    .run()
                    .expect("nested application");
                context.reach(nested.peak_live_occurrences);
                context.close(opened);
                Ok(Transition::Done(nested))
            }
            _ => Err(BodyFailure::Fault(
                filament_semantic_value::catalog::InternalFault::new(
                    "filament-eval-test",
                    "unexpected-test-body-resume",
                ),
            )),
        }
    }
}

fn host_factory(
    action: HostAction,
) -> (
    TestBodyFactory,
    Arc<Mutex<Vec<ContextRead>>>,
    Arc<Mutex<Vec<&'static str>>>,
) {
    let reads = Arc::new(Mutex::new(Vec::new()));
    let resumes = Arc::new(Mutex::new(Vec::new()));
    (
        TestBodyFactory {
            action,
            reads: Arc::clone(&reads),
            resumes: Arc::clone(&resumes),
        },
        reads,
        resumes,
    )
}

fn limit(live_occurrences: u64, held: u64) -> SessionLimits {
    SessionLimits {
        live_occurrences,
        held,
    }
}

fn incomplete(evaluation: &Evaluation) -> &Incomplete {
    match &evaluation.outcome {
        FamilyOutcome::Evaluated(Outcome::Incomplete(incomplete)) => incomplete,
        other => panic!("expected an incomplete evaluation, got {other:?}"),
    }
}

#[trace("TC-005", "FR-013-AC-8")]
#[test]
fn direct_context_opens_closes_and_reports_nested_peaks() {
    let mut context = LiveContext::new(10);
    assert_eq!(SessionLimits::default().held, 0);
    assert_eq!(context.live(), 0);
    assert_eq!(context.limit(), 10);
    assert_eq!(context.peak(), 0);

    let first = context
        .open(&Integer::from(4_u64))
        .expect("first frame fits");
    let second = context
        .open(&Integer::from(4_u64))
        .expect("second frame fits");
    assert_eq!((context.live(), context.peak()), (8, 8));
    assert_eq!(context.nested(), limit(10, 8));

    let refused = context
        .open(&Integer::from(4_u64))
        .expect_err("the third frame exceeds the limit");
    assert_eq!(refused.limit_kind, LimitKind::LiveOccurrences);
    assert_eq!(refused.limit, 10);
    assert_eq!(refused.consumed, 8);
    assert_eq!(refused.next_charge, Integer::from(4_u64));
    assert_eq!(refused.charge_point, ChargePoint::FunctionCall);
    assert_eq!((context.live(), context.peak()), (8, 8));

    context.close(second);
    context.close(first);
    assert_eq!(context.live(), 0);
    context.reach(9);
    assert_eq!(context.peak(), 9);
}

#[trace("TC-005", "FR-013-AC-1")]
#[test]
fn a_stop_inside_an_interpreted_transition_ends_the_registered_application() {
    let toy = Toy::default();
    let binds = toy.binds();
    let reads = toy.live_contexts();
    let (factory, host_reads, resumes) = host_factory(HostAction::Call {
        callable: live_child_three(),
        arguments: vec![int(0), int(0), int(0)],
    });
    let mut registry = Registry::new(toy);
    assert_eq!(
        registry.register(add(), &signature_of_arity(2), factory),
        Admission::Accepted
    );
    let mut fuel = meter(u64::MAX);
    let stopped = registry
        .begin_call(add(), vec![int(0), int(0)], limit(10, 0), &(), &mut fuel)
        .expect("root is admitted")
        .run()
        .expect("a live stop is an evaluation");

    let record = incomplete(&stopped);
    assert_eq!(record.limit_kind, LimitKind::LiveOccurrences);
    assert_eq!(record.limit, 10);
    assert_eq!(record.consumed, 7);
    assert_eq!(record.next_charge, Integer::from(4_u64));
    assert_eq!(record.charge_point, ChargePoint::FunctionCall);
    assert_eq!(stopped.location, Some(site()));
    assert_eq!(stopped.frame_depth, 1);
    assert_eq!(stopped.peak_live_occurrences, 7);
    assert_eq!(spent(&fuel), 3);
    assert_eq!(
        *host_reads.lock().expect("host reads"),
        vec![ContextRead {
            live: 3,
            limit: 10,
            peak: 3,
            nested_held: 3,
        }]
    );
    assert_eq!(*resumes.lock().expect("resume log"), vec!["start"]);
    assert_eq!(
        *binds.lock().expect("bind log"),
        vec![(live_child_three(), vec![0, 0, 0])]
    );
    assert_eq!(*reads.lock().expect("live-context log"), vec![(7, 10, 7)]);
}

#[trace("TC-005", "FR-013-AC-2", "FR-013-AC-3")]
#[test]
fn in_transition_frames_share_the_session_count_and_peak() {
    let toy = Toy::default();
    let contexts = toy.live_contexts();
    let (factory, host_reads, resumes) = host_factory(HostAction::Call {
        callable: live_child_two(),
        arguments: vec![int(0), int(0)],
    });
    let mut registry = Registry::new(toy);
    assert_eq!(
        registry.register(add(), &signature_of_arity(2), factory),
        Admission::Accepted
    );
    let mut fuel = meter(u64::MAX);
    let completed = registry
        .begin_call(add(), vec![int(0), int(0)], limit(10, 0), &(), &mut fuel)
        .expect("root is admitted")
        .run()
        .expect("application");

    assert_eq!(completed_int(&completed), Some(6));
    assert_eq!(completed.peak_live_occurrences, 10);
    assert_eq!(
        *host_reads.lock().expect("host reads"),
        vec![
            ContextRead {
                live: 3,
                limit: 10,
                peak: 3,
                nested_held: 3,
            },
            ContextRead {
                live: 3,
                limit: 10,
                peak: 3,
                nested_held: 3,
            }
        ]
    );
    assert_eq!(*resumes.lock().expect("resume log"), vec!["start", "child"]);
    assert_eq!(
        *contexts.lock().expect("live-context log"),
        vec![(6, 10, 6)]
    );
}

#[trace("TC-005", "FR-013-AC-3")]
#[test]
fn a_frame_carries_opened_occurrences_across_transitions() {
    let (factory, reads, resumes) = host_factory(HostAction::Carry);
    let limits = limit(10, 2);
    let mut registry = Registry::new(Toy::default());
    assert_eq!(
        registry.register(add(), &signature_of_arity(2), factory),
        Admission::Accepted
    );
    let mut fuel = meter(u64::MAX);
    let session = registry
        .begin_call(add(), vec![int(0), int(0)], limits, &(), &mut fuel)
        .expect("root is admitted");
    let session = match session.step() {
        filament_eval::Progress::Runnable(session) => session,
        filament_eval::Progress::Terminal(result) => panic!("root ended early: {result:?}"),
    };
    assert_eq!(session.live_occurrences(), 5);
    assert_eq!(session.limits(), limits);

    let session = match session.step() {
        filament_eval::Progress::Runnable(session) => session,
        filament_eval::Progress::Terminal(result) => panic!("body ended early: {result:?}"),
    };
    assert_eq!(session.live_occurrences(), 9);

    let completed = match session.step() {
        filament_eval::Progress::Terminal(Ok(evaluation)) => evaluation,
        filament_eval::Progress::Terminal(Err(failure)) => panic!("body failed: {failure:?}"),
        filament_eval::Progress::Runnable(_) => panic!("body did not finish"),
    };
    assert_eq!(completed.peak_live_occurrences, 9);
    assert_eq!(
        *reads.lock().expect("context reads"),
        vec![
            ContextRead {
                live: 5,
                limit: 10,
                peak: 5,
                nested_held: 5,
            },
            ContextRead {
                live: 9,
                limit: 10,
                peak: 9,
                nested_held: 9,
            }
        ]
    );
    assert_eq!(*resumes.lock().expect("resume log"), vec!["start", "again"]);
}

#[trace("TC-005", "FR-013-AC-4")]
#[test]
fn nested_sessions_inherit_live_and_report_their_peak() {
    let (factory, reads, resumes) = host_factory(HostAction::Nested {
        callable: live_child_two(),
        arguments: vec![int(0), int(0)],
    });
    let mut registry = Registry::new(Toy::default());
    assert_eq!(
        registry.register(add(), &signature_of_arity(2), factory),
        Admission::Accepted
    );
    let mut fuel = meter(u64::MAX);
    let completed = registry
        .begin_call(add(), vec![int(0), int(0)], limit(10, 0), &(), &mut fuel)
        .expect("root is admitted")
        .run()
        .expect("nested application completes");
    assert_eq!(completed_int(&completed), Some(6));
    assert_eq!(completed.peak_live_occurrences, 10);
    assert_eq!(
        *reads.lock().expect("context reads"),
        vec![ContextRead {
            live: 3,
            limit: 10,
            peak: 3,
            nested_held: 3,
        }]
    );
    assert_eq!(*resumes.lock().expect("resume log"), vec!["start"]);

    let (factory, _reads, resumes) = host_factory(HostAction::Nested {
        callable: live_child_two(),
        arguments: vec![int(0), int(0)],
    });
    let mut registry = Registry::new(Toy::default());
    assert_eq!(
        registry.register(add(), &signature_of_arity(2), factory),
        Admission::Accepted
    );
    let mut fuel = meter(u64::MAX);
    let stopped = registry
        .begin_call(add(), vec![int(0), int(0)], limit(9, 0), &(), &mut fuel)
        .expect("root is admitted")
        .run()
        .expect("a nested live stop ends the application");
    let record = incomplete(&stopped);
    assert_eq!(record.limit, 9);
    assert_eq!(record.consumed, 6);
    assert_eq!(record.next_charge, Integer::from(4_u64));
    assert_eq!(stopped.peak_live_occurrences, 6);
    assert_eq!(*resumes.lock().expect("resume log"), vec!["start"]);
}

#[trace("TC-005", "FR-013-AC-2")]
#[test]
fn a_lower_limit_refuses_the_in_transition_frame() {
    let (factory, _reads, resumes) = host_factory(HostAction::Call {
        callable: live_child_two(),
        arguments: vec![int(0), int(0)],
    });
    let mut registry = Registry::new(Toy::default());
    assert_eq!(
        registry.register(add(), &signature_of_arity(2), factory),
        Admission::Accepted
    );
    let mut fuel = meter(u64::MAX);
    let stopped = registry
        .begin_call(add(), vec![int(0), int(0)], limit(9, 0), &(), &mut fuel)
        .expect("root is admitted")
        .run()
        .expect("a live stop is an evaluation");
    let record = incomplete(&stopped);
    assert_eq!(record.limit, 9);
    assert_eq!(record.consumed, 6);
    assert_eq!(record.next_charge, Integer::from(4_u64));
    assert_eq!(stopped.frame_depth, 1);
    assert_eq!(stopped.peak_live_occurrences, 6);
    assert_eq!(*resumes.lock().expect("resume log"), vec!["start"]);
}

#[trace("TC-005", "FR-013-AC-4")]
#[test]
fn registry_nested_sessions_inherit_the_session_and_interpreted_frames() {
    let (factory, _reads, resumes) = host_factory(HostAction::Call {
        callable: nested_fit_child(),
        arguments: vec![int(0), int(0)],
    });
    let mut registry = Registry::new(Toy::default());
    assert_eq!(
        registry.register(add(), &signature_of_arity(2), factory),
        Admission::Accepted
    );
    let mut fuel = meter(u64::MAX);
    let completed = registry
        .begin_call(add(), vec![int(0), int(0)], limit(10, 0), &(), &mut fuel)
        .expect("root is admitted")
        .run()
        .expect("nested session completes at held 6 plus frame 4");
    assert_eq!(completed.peak_live_occurrences, 10);
    assert_eq!(*resumes.lock().expect("resume log"), vec!["start", "child"]);

    let (factory, _reads, resumes) = host_factory(HostAction::Call {
        callable: nested_over_child(),
        arguments: vec![int(0), int(0)],
    });
    let mut registry = Registry::new(Toy::default());
    assert_eq!(
        registry.register(add(), &signature_of_arity(2), factory),
        Admission::Accepted
    );
    let mut fuel = meter(u64::MAX);
    let stopped = registry
        .begin_call(add(), vec![int(0), int(0)], limit(10, 0), &(), &mut fuel)
        .expect("root is admitted")
        .run()
        .expect("nested session stop ends the outer application");
    let record = incomplete(&stopped);
    assert_eq!(record.limit, 10);
    assert_eq!(record.consumed, 6);
    assert_eq!(record.next_charge, Integer::from(5_u64));
    assert_eq!(stopped.peak_live_occurrences, 6);
    assert_eq!(*resumes.lock().expect("resume log"), vec!["start"]);
}

#[trace("TC-005", "FR-013-AC-4", "FR-013-AC-5")]
#[test]
fn nested_sessions_started_inside_a_transition_inherit_held_frames_and_peak() {
    let (factory, _reads, resumes) = host_factory(HostAction::NestedAfterOpen {
        occurrences: 4,
        callable: live_done_two(),
        arguments: vec![int(0), int(0)],
    });
    let mut registry = Registry::new(Toy::default());
    assert_eq!(
        registry.register(add(), &signature_of_arity(2), factory),
        Admission::Accepted
    );
    let mut fuel = meter(u64::MAX);
    let completed = registry
        .begin_call(add(), vec![int(0), int(0)], limit(10, 0), &(), &mut fuel)
        .expect("root is admitted")
        .run()
        .expect("nested frame of 3 fits under held 7");
    assert_eq!(completed.peak_live_occurrences, 10);
    assert_eq!(*resumes.lock().expect("resume log"), vec!["start"]);

    let (factory, _reads, resumes) = host_factory(HostAction::NestedAfterOpen {
        occurrences: 4,
        callable: live_done_three(),
        arguments: vec![int(0), int(0), int(0)],
    });
    let mut registry = Registry::new(Toy::default());
    assert_eq!(
        registry.register(add(), &signature_of_arity(2), factory),
        Admission::Accepted
    );
    let mut fuel = meter(u64::MAX);
    let stopped = registry
        .begin_call(add(), vec![int(0), int(0)], limit(10, 0), &(), &mut fuel)
        .expect("root is admitted")
        .run()
        .expect("nested frame of 4 exceeds held 7");
    let record = incomplete(&stopped);
    assert_eq!(record.limit, 10);
    assert_eq!(record.consumed, 7);
    assert_eq!(record.next_charge, Integer::from(4_u64));
    assert_eq!(stopped.peak_live_occurrences, 7);
    assert_eq!(*resumes.lock().expect("resume log"), vec!["start"]);

    let (factory, _reads, _resumes) = host_factory(HostAction::NestedAfterOpen {
        occurrences: 3,
        callable: live_done_two(),
        arguments: vec![int(0), int(0)],
    });
    let mut registry = Registry::new(Toy::default());
    assert_eq!(
        registry.register(add(), &signature_of_arity(2), factory),
        Admission::Accepted
    );
    let mut fuel = meter(u64::MAX);
    let completed = registry
        .begin_call(add(), vec![int(0), int(0)], limit(20, 0), &(), &mut fuel)
        .expect("root is admitted")
        .run()
        .expect("nested peak propagates");
    assert_eq!(completed.peak_live_occurrences, 9);
}

#[trace("TC-005", "FR-013-AC-7")]
#[test]
fn carried_frames_contribute_to_the_next_scheduler_refusal_depth() {
    let (factory, reads, resumes) = host_factory(HostAction::CallWithCarries {
        callable: count(),
        arguments: vec![int(0)],
    });
    let mut registry = Registry::new(Toy::default());
    assert_eq!(
        registry.register(add(), &signature_of_arity(2), factory),
        Admission::Accepted
    );
    let mut fuel = meter(u64::MAX);
    let stopped = registry
        .begin_call(add(), vec![int(0), int(0)], limit(12, 0), &(), &mut fuel)
        .expect("root is admitted")
        .run()
        .expect("the next scheduler call exceeds the limit");
    let record = incomplete(&stopped);
    assert_eq!(record.limit, 12);
    assert_eq!(record.consumed, 11);
    assert_eq!(record.next_charge, Integer::from(2_u64));
    assert_eq!(stopped.frame_depth, 2);
    assert_eq!(stopped.peak_live_occurrences, 11);
    assert_eq!(
        reads.lock().expect("context reads").last(),
        Some(&ContextRead {
            live: 11,
            limit: 12,
            peak: 11,
            nested_held: 11,
        })
    );
    assert_eq!(*resumes.lock().expect("resume log"), vec!["start"]);
}

#[trace("TC-005", "FR-013-AC-4")]
#[test]
fn interpreted_nested_sessions_inherit_in_transition_frames() {
    let mut registry = Registry::new(Toy::default());
    let mut fuel = meter(u64::MAX);
    let completed = registry
        .begin_call(
            native_nested_fit(),
            vec![int(0), int(0)],
            limit(10, 0),
            &(),
            &mut fuel,
        )
        .expect("native root is admitted")
        .run()
        .expect("nested frame of 3 fits under held 7");
    assert_eq!(completed.peak_live_occurrences, 10);

    let mut registry = Registry::new(Toy::default());
    let mut fuel = meter(u64::MAX);
    let stopped = registry
        .begin_call(
            native_nested_over(),
            vec![int(0), int(0)],
            limit(10, 0),
            &(),
            &mut fuel,
        )
        .expect("native root is admitted")
        .run()
        .expect("nested frame of 4 exceeds held 7");
    let record = incomplete(&stopped);
    assert_eq!(record.limit, 10);
    assert_eq!(record.consumed, 7);
    assert_eq!(record.next_charge, Integer::from(4_u64));
    assert_eq!(stopped.peak_live_occurrences, 7);
}
