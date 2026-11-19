// SPDX-License-Identifier: AGPL-3.0-or-later
//! A toy evaluator behind the `CheckedClosure` contract, and recording host
//! bodies, for exercising the leaf through its public API alone.
//!
//! The toy language has interpreted callables over non-negative integers:
//!
//! - `COUNT(n)`: `0` when `n = 0`, otherwise `COUNT(n - 1) + 1`;
//! - `BOOM(n)`: always undefined, located at its call site;
//! - `REFUSE(n)`: always refused, located at its call site;
//! - `LOSSY(n)`: completes with `1` and one loss record;
//! - `ADD(a, b)`: `a + b`;
//! - `REPEAT(n)`: applies `COUNT(0)` n times in sequence, then `n`;
//!
//! and `HOST_A` and `HOST_B`, admitted callables with no interpreted body, so
//! they can only run as registered host bodies. These tests exercise the
//! scheduler through the contract; they are not the std reference adapter's
//! tests.
#![allow(dead_code)] // Each integration test uses a different part of this module.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use filament_eval::{
    Body, BodyFactory, BodyFailure, CallFailure, CheckedClosure, ChildRequest, Evaluation,
    FamilyOutcome, LiveContext, RequestRefusal, Resume, Signature, Transition,
};
use filament_exact::{
    Charge, ChargePoint, IeeeFlag, IeeeWidth, Integer, LimitKind, Meter, NodeKey, Outcome,
    PathSegment, Refusal, ScalarLimits, Undefined, Value, ValueType,
};
use filament_semantic_value::call::InputRefusal;
use filament_semantic_value::catalog::InternalFault;
use filament_semantic_value::declaration::{EnvironmentFailure, StorageRequest, StorageUnit};
use filament_semantic_value::location::{ExpressionLocation, ExpressionOwner};
use filament_semantic_value::loss::{LocatedLoss, ValueLoss};

/// A fixture node key: the toy closure's own structural identities, the
/// `ordinal`th node of the toy package.
pub fn key(ordinal: u8) -> NodeKey {
    NodeKey::new(
        "toy",
        vec![PathSegment::Text("closure".into())],
        u64::from(ordinal),
    )
    .expect("a toy node identity is well formed")
}

pub fn count() -> NodeKey {
    key(1)
}
pub fn boom() -> NodeKey {
    key(2)
}
pub fn host_a() -> NodeKey {
    key(3)
}
pub fn host_b() -> NodeKey {
    key(4)
}
pub fn add() -> NodeKey {
    key(5)
}
pub fn refuse() -> NodeKey {
    key(6)
}
pub fn lossy() -> NodeKey {
    key(7)
}
/// Analogue of a ranged parameter: admits arguments up to 9.
pub fn ranged() -> NodeKey {
    key(8)
}
/// Analogue of a storage-bound parameter: its membership never decides.
pub fn storage() -> NodeKey {
    key(9)
}
/// Analogue of a `Population` parameter of declared maximum 3: it admits only
/// 3; 4 is the wrong maximum and 99 an id the environment does not resolve.
pub fn population() -> NodeKey {
    key(10)
}
/// A key no function of the toy closure has.
pub fn absent() -> NodeKey {
    key(99)
}
pub fn recursive_undefined() -> NodeKey {
    key(11)
}
pub fn completed_depth() -> NodeKey {
    key(12)
}
/// `REPEAT(n)`: applies `COUNT` to 0 n times in sequence and completes with n.
pub fn repeat() -> NodeKey {
    key(13)
}
pub fn live_child_two() -> NodeKey {
    key(14)
}
pub fn live_child_three() -> NodeKey {
    key(15)
}
pub fn nested_fit_child() -> NodeKey {
    key(16)
}
pub fn nested_over_child() -> NodeKey {
    key(17)
}
pub fn live_done_two() -> NodeKey {
    key(18)
}
pub fn live_done_three() -> NodeKey {
    key(19)
}
pub fn live_done_four() -> NodeKey {
    key(20)
}
pub fn native_nested_fit() -> NodeKey {
    key(21)
}
pub fn native_nested_over() -> NodeKey {
    key(22)
}
pub fn native_chain() -> NodeKey {
    key(23)
}
pub fn native_peak_then_call() -> NodeKey {
    key(24)
}
pub fn live_observe() -> NodeKey {
    key(25)
}

pub fn signature_of_arity(arity: usize) -> Signature {
    Signature::new(vec![ValueType::Integer; arity], ValueType::Integer)
}

pub fn int_signature() -> Signature {
    signature_of_arity(1)
}

pub fn int(n: u64) -> Value {
    Value::Integer(Integer::from(n))
}

/// The call site of a request that does not depend on depth.
pub fn site() -> ExpressionLocation {
    ExpressionLocation::root(ExpressionOwner::Expression)
}

/// The checked body root of an admitted callable in the toy closure.
pub fn root_site(callable: &NodeKey) -> ExpressionLocation {
    let index = usize::try_from(callable.ordinal()).expect("toy ordinals fit usize");
    ExpressionLocation::root(ExpressionOwner::Body {
        function: format!("toy_{index}"),
        index,
    })
}

/// The call site of the request that applies `COUNT` to `argument`, so a
/// denied call identifies how deep the run got.
pub fn count_site(argument: u64) -> ExpressionLocation {
    let step = usize::try_from(argument).unwrap_or(usize::MAX);
    ExpressionLocation::at(ExpressionOwner::Expression, &[step])
}

/// The integer a completed evaluation holds.
pub fn completed_int(evaluation: &Evaluation) -> Option<u64> {
    match &evaluation.outcome {
        FamilyOutcome::Evaluated(Outcome::Completed(Value::Integer(value))) => value.to_u64(),
        _ => None,
    }
}

/// A meter with `work_units` fuel and every other limit generous.
pub fn meter(work_units: u64) -> Meter {
    let limit = u64::from(u32::MAX);
    Meter::new(ScalarLimits {
        integer_bits: limit,
        decimal_digits: limit,
        scale_expansion: limit,
        text_input_bytes: limit,
        text_scalars: limit,
        normalized_scalars: limit,
        unit_edges: limit,
        value_occurrences: limit,
        work_units,
        result_units: limit,
    })
}

pub fn spent(meter: &Meter) -> u64 {
    meter.consumed(LimitKind::WorkUnits)
}

pub enum Expression {
    /// A literal: completes with no application and no charge.
    Literal(u64),
    /// Apply `callee` to `argument`, completing with its value.
    Apply { callee: NodeKey, argument: u64 },
    /// As `Apply`, but admitted with any parameters, which it binds unused.
    ApplyWith { callee: NodeKey, argument: u64 },
    /// Apply `outer` to the values of `left` and then `right`: the source call
    /// `outer(left(x), right(y))`, whose argument applications run left to
    /// right before `outer`'s own call.
    Nested {
        outer: NodeKey,
        left: (NodeKey, u64),
        right: (NodeKey, u64),
    },
}

pub enum Task {
    Count,
    Boom,
    Literal(u64),
    Apply {
        callee: NodeKey,
        argument: u64,
    },
    Applied,
    Add,
    Refuse,
    Lossy,
    Nested {
        outer: NodeKey,
        left: (NodeKey, u64),
        right: (NodeKey, u64),
        values: Vec<u64>,
    },
    /// Admitted but never given an interpreted body.
    Bodiless,
    RecursiveUndefined,
    CompletedDepth,
    /// `REPEAT`: how many applications of `COUNT(0)` have returned.
    Repeat(u64),
    /// Opens and closes an in-transition frame of four occurrences.
    LiveOpen,
    /// Begins a nested application with the current context's limits.
    NestedCall {
        callable: NodeKey,
        arguments: Vec<Value>,
        open_occurrences: Option<u64>,
    },
    /// Calls a child which opens and closes an in-transition frame.
    NativeChain,
    /// Opens and closes two in-transition frames, then requests a child.
    NativePeakThenCall,
    /// Records the transition context and completes.
    LiveObserve,
    /// Completes without opening another frame.
    LiveDone,
}

/// The callables entered, in order, with their integer arguments.
pub type Binds = Arc<Mutex<Vec<(NodeKey, Vec<u64>)>>>;

#[derive(Default)]
pub struct Toy {
    binds: Binds,
    live_contexts: Arc<Mutex<Vec<(u64, u64, u64)>>>,
    missing_root_site: bool,
}

impl Toy {
    /// The log of interpreted callables bound, in the order they were entered.
    pub fn binds(&self) -> Binds {
        Arc::clone(&self.binds)
    }

    pub fn live_contexts(&self) -> Arc<Mutex<Vec<(u64, u64, u64)>>> {
        Arc::clone(&self.live_contexts)
    }

    /// An admitted closure with a broken root-location invariant.
    pub fn without_root_site() -> Self {
        Self {
            missing_root_site: true,
            ..Self::default()
        }
    }

    fn admitted(callable: &NodeKey) -> bool {
        [
            count(),
            boom(),
            host_a(),
            host_b(),
            add(),
            refuse(),
            lossy(),
            ranged(),
            storage(),
            population(),
            recursive_undefined(),
            completed_depth(),
            repeat(),
            live_child_two(),
            live_child_three(),
            nested_fit_child(),
            nested_over_child(),
            live_done_two(),
            live_done_three(),
            live_done_four(),
            native_nested_fit(),
            native_nested_over(),
            native_chain(),
            native_peak_then_call(),
            live_observe(),
        ]
        .contains(callable)
    }

    fn arity(callable: &NodeKey) -> usize {
        if *callable == add()
            || *callable == live_child_two()
            || *callable == nested_fit_child()
            || *callable == nested_over_child()
            || *callable == native_nested_fit()
            || *callable == native_nested_over()
            || *callable == native_chain()
            || *callable == native_peak_then_call()
            || *callable == live_done_two()
        {
            2
        } else if *callable == live_child_three() || *callable == live_done_three() {
            3
        } else if *callable == live_done_four() {
            4
        } else {
            1
        }
    }
}

fn integers(values: &[Value]) -> Vec<u64> {
    values
        .iter()
        .filter_map(|value| match value {
            Value::Integer(n) => n.to_u64(),
            _ => None,
        })
        .collect()
}

impl CheckedClosure for Toy {
    type Expression = Expression;
    type Environment = ();
    type Task = (Task, Vec<Value>);

    fn signature(&self, callable: &NodeKey) -> Option<Signature> {
        Self::admitted(callable).then(|| signature_of_arity(Self::arity(callable)))
    }

    fn root_call_site(&self, callable: &NodeKey) -> Option<ExpressionLocation> {
        (Self::admitted(callable) && !self.missing_root_site).then(|| root_site(callable))
    }

    fn admit_call(
        &self,
        callable: &NodeKey,
        arguments: &[Value],
        _environment: &(),
        meter: &Meter,
    ) -> Result<(), CallFailure> {
        if let Some(cause) = meter.cancel().and_then(filament_exact::Cancel::cause) {
            return Err(CallFailure::Cancelled(cause));
        }
        if !Self::admitted(callable) {
            return Err(InputRefusal::UnknownFunction("absent".into()).into());
        }
        if arguments.len() != Self::arity(callable) {
            return Err(InputRefusal::Arity {
                declared: Self::arity(callable),
                supplied: arguments.len(),
            }
            .into());
        }
        Ok(())
    }

    fn admit_request(
        &self,
        callable: &NodeKey,
        arguments: &[Value],
        _environment: &(),
    ) -> Result<(), RequestRefusal> {
        if !Self::admitted(callable) || arguments.len() != Self::arity(callable) {
            return Err(RequestRefusal::NotAdmitted);
        }
        let first = integers(arguments).first().copied().unwrap_or_default();
        if *callable == ranged() && first > 9 {
            return Err(RequestRefusal::NotAdmitted);
        }
        if *callable == population() && first != 3 {
            return Err(RequestRefusal::NotAdmitted);
        }
        if *callable == storage() {
            return Err(RequestRefusal::Storage {
                parameter: 0,
                failure: EnvironmentFailure::Allocation(StorageRequest::new(
                    8,
                    StorageUnit::AdditionalElements,
                )),
            });
        }
        Ok(())
    }

    fn admit_expression(
        &self,
        expression: &Expression,
        parameters: &[Value],
        _environment: &(),
        _meter: &Meter,
    ) -> Result<(), CallFailure> {
        if parameters.is_empty() || matches!(expression, Expression::ApplyWith { .. }) {
            Ok(())
        } else {
            Err(InputRefusal::Arity {
                declared: 0,
                supplied: parameters.len(),
            }
            .into())
        }
    }

    fn bind_call(
        &self,
        callable: &NodeKey,
        arguments: Vec<Value>,
        _environment: &(),
    ) -> Result<Self::Task, InternalFault> {
        self.binds
            .lock()
            .expect("bind log")
            .push((callable.clone(), integers(&arguments)));
        let task = if *callable == count() {
            Task::Count
        } else if *callable == boom() {
            Task::Boom
        } else if *callable == add() || *callable == ranged() || *callable == population() {
            Task::Add
        } else if *callable == refuse() {
            Task::Refuse
        } else if *callable == lossy() {
            Task::Lossy
        } else if *callable == recursive_undefined() {
            Task::RecursiveUndefined
        } else if *callable == completed_depth() {
            Task::CompletedDepth
        } else if *callable == repeat() {
            Task::Repeat(0)
        } else if *callable == live_child_two() || *callable == live_child_three() {
            Task::LiveOpen
        } else if *callable == nested_fit_child() {
            Task::NestedCall {
                callable: live_done_three(),
                arguments: vec![int(0), int(0), int(0)],
                open_occurrences: None,
            }
        } else if *callable == nested_over_child() {
            Task::NestedCall {
                callable: live_done_four(),
                arguments: vec![int(0), int(0), int(0), int(0)],
                open_occurrences: None,
            }
        } else if *callable == native_nested_fit() {
            Task::NestedCall {
                callable: live_done_two(),
                arguments: vec![int(0), int(0)],
                open_occurrences: Some(4),
            }
        } else if *callable == native_nested_over() {
            Task::NestedCall {
                callable: live_done_three(),
                arguments: vec![int(0), int(0), int(0)],
                open_occurrences: Some(4),
            }
        } else if *callable == native_chain() {
            Task::NativeChain
        } else if *callable == native_peak_then_call() {
            Task::NativePeakThenCall
        } else if *callable == live_observe() {
            Task::LiveObserve
        } else if *callable == live_done_two()
            || *callable == live_done_three()
            || *callable == live_done_four()
        {
            Task::LiveDone
        } else {
            Task::Bodiless
        };
        Ok((task, arguments))
    }

    fn bind_expression(
        &self,
        expression: &Expression,
        parameters: Vec<Value>,
        _environment: &(),
    ) -> Result<Self::Task, InternalFault> {
        let task = match expression {
            Expression::Literal(n) => Task::Literal(*n),
            Expression::Apply { callee, argument } | Expression::ApplyWith { callee, argument } => {
                Task::Apply {
                    callee: callee.clone(),
                    argument: *argument,
                }
            }
            Expression::Nested { outer, left, right } => Task::Nested {
                outer: outer.clone(),
                left: left.clone(),
                right: right.clone(),
                values: Vec::new(),
            },
        };
        Ok((task, parameters))
    }

    fn advance(
        &self,
        task: &mut Self::Task,
        delivered: Option<Evaluation>,
        context: &mut LiveContext,
        _environment: &(),
        meter: &mut Meter,
    ) -> Result<Transition, InternalFault> {
        let (state, arguments) = task;
        if matches!(state, Task::LiveOpen | Task::LiveObserve) {
            self.live_contexts.lock().expect("live-context log").push((
                context.live(),
                context.limit(),
                context.peak(),
            ));
        }
        let argument = arguments
            .first()
            .and_then(|value| match value {
                Value::Integer(n) => n.to_u64(),
                _ => None,
            })
            .unwrap_or_default();
        match state {
            Task::Count => match delivered {
                Some(child) => Ok(Transition::Done(match completed_int(&child) {
                    Some(value) => Evaluation::completed(int(value + 1)),
                    // A non-completed child propagates unchanged.
                    None => child,
                })),
                None if argument == 0 => Ok(Transition::Done(Evaluation::completed(int(0)))),
                None => Ok(Transition::Call(ChildRequest {
                    callee: count(),
                    arguments: vec![int(argument - 1)],
                    site: count_site(argument - 1),
                })),
            },
            Task::RecursiveUndefined => match delivered {
                Some(child) => Ok(Transition::Done(child)),
                None if argument <= 1 => Ok(Transition::Done(Evaluation {
                    outcome: FamilyOutcome::Evaluated(Outcome::Undefined(
                        Undefined::DivisionByZero,
                    )),
                    location: Some(site()),
                    losses: Vec::new(),
                    frame_depth: 0,
                    peak_live_occurrences: 0,
                })),
                None => Ok(Transition::Call(ChildRequest {
                    callee: recursive_undefined(),
                    arguments: vec![int(argument - 1)],
                    site: count_site(argument - 1),
                })),
            },
            Task::Repeat(returned) => {
                if let Some(child) = delivered {
                    if completed_int(&child).is_none() {
                        return Ok(Transition::Done(child));
                    }
                    *returned += 1;
                }
                if *returned == argument {
                    return Ok(Transition::Done(Evaluation::completed(int(argument))));
                }
                Ok(Transition::Call(ChildRequest {
                    callee: count(),
                    arguments: vec![int(0)],
                    site: site(),
                }))
            }
            Task::LiveOpen => {
                meter
                    .charge(Charge::new(ChargePoint::FunctionCall))
                    .expect("the test meter admits the in-transition call");
                match context.open(&Integer::from(4_u64)) {
                    Ok(occurrences) => {
                        context.close(occurrences);
                        Ok(Transition::Done(Evaluation::completed(int(context.live()))))
                    }
                    Err(incomplete) => Ok(Transition::Done(Evaluation {
                        outcome: FamilyOutcome::Evaluated(Outcome::Incomplete(incomplete)),
                        location: Some(site()),
                        losses: Vec::new(),
                        frame_depth: 0,
                        peak_live_occurrences: 0,
                    })),
                }
            }
            Task::NestedCall {
                callable,
                arguments: nested_arguments,
                open_occurrences,
            } => {
                let opened = match open_occurrences {
                    Some(occurrences) => {
                        meter
                            .charge(Charge::new(ChargePoint::FunctionCall))
                            .expect("the test meter admits the in-transition call");
                        match context.open(&Integer::from(*occurrences)) {
                            Ok(opened) => Some(opened),
                            Err(incomplete) => {
                                return Ok(Transition::Done(Evaluation {
                                    outcome: FamilyOutcome::Evaluated(Outcome::Incomplete(
                                        incomplete,
                                    )),
                                    location: Some(site()),
                                    losses: Vec::new(),
                                    frame_depth: 0,
                                    peak_live_occurrences: 0,
                                }));
                            }
                        }
                    }
                    None => None,
                };
                let limits = context.nested();
                let mut registry = filament_eval::Registry::new(Toy::default());
                let nested = registry
                    .begin_call(
                        callable.clone(),
                        nested_arguments.clone(),
                        limits,
                        &(),
                        meter,
                    )
                    .map_err(|_| fault("nested-call-not-admitted"))?
                    .run()
                    .map_err(|_| fault("nested-call-failed"))?;
                context.reach(nested.peak_live_occurrences);
                if let Some(opened) = opened {
                    context.close(opened);
                }
                Ok(Transition::Done(nested))
            }
            Task::NativeChain => {
                *state = Task::Applied;
                Ok(Transition::Call(ChildRequest {
                    callee: live_child_two(),
                    arguments: vec![int(0), int(0)],
                    site: site(),
                }))
            }
            Task::NativePeakThenCall => {
                meter
                    .charge(Charge::new(ChargePoint::FunctionCall))
                    .expect("the test meter admits C");
                let c = context.open(&Integer::from(4_u64)).expect("C fits");
                meter
                    .charge(Charge::new(ChargePoint::FunctionCall))
                    .expect("the test meter admits N");
                let n = context.open(&Integer::from(4_u64)).expect("N fits");
                context.close(n);
                context.close(c);
                *state = Task::Applied;
                Ok(Transition::Call(ChildRequest {
                    callee: live_done_two(),
                    arguments: vec![int(0), int(0)],
                    site: site(),
                }))
            }
            Task::LiveDone => Ok(Transition::Done(Evaluation::completed(int(0)))),
            Task::LiveObserve => Ok(Transition::Done(Evaluation::completed(int(0)))),
            Task::CompletedDepth => Ok(Transition::Done(Evaluation {
                outcome: FamilyOutcome::Evaluated(Outcome::Completed(int(1))),
                location: None,
                losses: Vec::new(),
                frame_depth: 7,
                peak_live_occurrences: 0,
            })),
            Task::Boom => Ok(Transition::Done(Evaluation {
                outcome: FamilyOutcome::Evaluated(Outcome::Undefined(Undefined::DivisionByZero)),
                location: Some(site()),
                losses: Vec::new(),
                frame_depth: 0,
                peak_live_occurrences: 0,
            })),
            Task::Add => Ok(Transition::Done(Evaluation::completed(int(integers(
                arguments,
            )
            .iter()
            .sum())))),
            Task::Refuse => Ok(Transition::Done(Evaluation {
                outcome: FamilyOutcome::Evaluated(Outcome::Refused(
                    Refusal::IeeeNanPayloadNotRepresentable {
                        target: IeeeWidth::Binary32,
                        source: IeeeWidth::Binary64,
                    },
                )),
                location: Some(site()),
                losses: Vec::new(),
                frame_depth: 0,
                peak_live_occurrences: 0,
            })),
            Task::Lossy => Ok(Transition::Done(Evaluation {
                outcome: FamilyOutcome::Evaluated(Outcome::Completed(int(1))),
                location: None,
                losses: vec![loss()],
                frame_depth: 0,
                peak_live_occurrences: 0,
            })),
            Task::Nested {
                outer,
                left,
                right,
                values,
            } => {
                if let Some(child) = delivered {
                    match completed_int(&child) {
                        Some(value) => values.push(value),
                        None => return Ok(Transition::Done(child)),
                    }
                }
                let (callee, arguments) = match values.as_slice() {
                    [] => (left.0.clone(), vec![int(left.1)]),
                    [_] => (right.0.clone(), vec![int(right.1)]),
                    [l, r] => (outer.clone(), vec![int(*l), int(*r)]),
                    [.., last] => {
                        return Ok(Transition::Done(Evaluation::completed(int(*last))));
                    }
                };
                Ok(Transition::Call(ChildRequest {
                    callee,
                    arguments,
                    site: site(),
                }))
            }
            Task::Literal(n) => Ok(Transition::Done(Evaluation::completed(int(*n)))),
            Task::Apply { callee, argument } => {
                let request = ChildRequest {
                    callee: callee.clone(),
                    arguments: vec![int(*argument)],
                    site: site(),
                };
                *state = Task::Applied;
                Ok(Transition::Call(request))
            }
            Task::Applied => delivered
                .map(Transition::Done)
                .ok_or_else(|| fault("applied-frame-resumed-without-a-child")),
            Task::Bodiless => Err(fault("callable-has-no-interpreted-body")),
        }
    }
}

fn fault(invariant: &'static str) -> InternalFault {
    InternalFault::new("toy", invariant)
}

/// The loss record `LOSSY` completes with.
pub fn loss() -> LocatedLoss {
    LocatedLoss {
        location: site(),
        loss: ValueLoss::IeeeFlags([IeeeFlag::Inexact].into_iter().collect()),
    }
}

/// What a recording body factory observed.
#[derive(Clone, Default)]
pub struct Counts {
    begun: Arc<AtomicUsize>,
    resumed: Arc<AtomicUsize>,
}

impl Counts {
    pub fn begun(&self) -> usize {
        self.begun.load(Ordering::SeqCst)
    }
    pub fn resumed(&self) -> usize {
        self.resumed.load(Ordering::SeqCst)
    }
}

/// How a host body finishes after its child returns.
#[derive(Clone, Copy)]
pub enum Finish {
    /// Return the child's evaluation unchanged (a direct-return caller).
    Direct,
    /// Return the child's integer plus the argument the body saved at start.
    AddSavedArgument,
}

/// A host body that applies `target` to its argument and finishes per `finish`.
pub struct Relay {
    pub target: NodeKey,
    pub finish: Finish,
    pub counts: Counts,
}

impl BodyFactory for Relay {
    fn begin(&self) -> Box<dyn Body> {
        self.counts.begun.fetch_add(1, Ordering::SeqCst);
        Box::new(RelayBody {
            target: self.target.clone(),
            finish: self.finish,
            counts: self.counts.clone(),
            saved: None,
        })
    }
}

struct RelayBody {
    target: NodeKey,
    finish: Finish,
    counts: Counts,
    saved: Option<u64>,
}

impl Body for RelayBody {
    fn resume(
        &mut self,
        input: Resume,
        _context: &mut LiveContext,
        _meter: &mut Meter,
    ) -> Result<Transition, BodyFailure> {
        self.counts.resumed.fetch_add(1, Ordering::SeqCst);
        match input {
            Resume::Start(arguments) => {
                let argument = arguments.first().cloned().unwrap_or_else(|| int(0));
                if let Value::Integer(n) = &argument {
                    self.saved = n.to_u64();
                }
                Ok(Transition::Call(ChildRequest {
                    callee: self.target.clone(),
                    arguments: vec![argument],
                    site: site(),
                }))
            }
            Resume::Child(child) => Ok(Transition::Done(match self.finish {
                Finish::Direct => *child,
                Finish::AddSavedArgument => match (completed_int(&child), self.saved) {
                    (Some(value), Some(saved)) => Evaluation::completed(int(value + saved)),
                    _ => *child,
                },
            })),
            Resume::Again => Err(fault("relay-resumed-without-input").into()),
        }
    }
}

/// A host body for FR-262-AC-8: it charges the meter, saves a local,
/// requests a child, charges again and continues with the child's value,
/// keeping the child's loss records.
pub struct ChargingRelay {
    pub target: NodeKey,
    pub counts: Counts,
}

impl BodyFactory for ChargingRelay {
    fn begin(&self) -> Box<dyn Body> {
        self.counts.begun.fetch_add(1, Ordering::SeqCst);
        Box::new(ChargingBody {
            target: self.target.clone(),
            counts: self.counts.clone(),
            saved: 0,
        })
    }
}

struct ChargingBody {
    target: NodeKey,
    counts: Counts,
    saved: u64,
}

impl ChargingBody {
    /// One unit of the body's own kernel work. A denial ends the body with the
    /// located incomplete evaluation, as the `Body` contract says.
    fn work(meter: &mut Meter) -> Option<Transition> {
        let denied = meter
            .charge(Charge::new(ChargePoint::IntegerArithmeticArithmetic))
            .err()?;
        Some(Transition::Done(Evaluation {
            outcome: FamilyOutcome::Evaluated(Outcome::Incomplete(denied)),
            location: Some(site()),
            losses: Vec::new(),
            frame_depth: 0,
            peak_live_occurrences: 0,
        }))
    }
}

impl Body for ChargingBody {
    fn resume(
        &mut self,
        input: Resume,
        _context: &mut LiveContext,
        meter: &mut Meter,
    ) -> Result<Transition, BodyFailure> {
        self.counts.resumed.fetch_add(1, Ordering::SeqCst);
        if let Some(done) = Self::work(meter) {
            return Ok(done);
        }
        match input {
            Resume::Start(arguments) => {
                self.saved = integers(&arguments).first().copied().unwrap_or_default();
                Ok(Transition::Call(ChildRequest {
                    callee: self.target.clone(),
                    arguments,
                    site: site(),
                }))
            }
            Resume::Child(child) => Ok(Transition::Done(match completed_int(&child) {
                Some(value) => Evaluation {
                    outcome: FamilyOutcome::Evaluated(Outcome::Completed(int(value + self.saved))),
                    location: None,
                    losses: child.losses,
                    frame_depth: 0,
                    peak_live_occurrences: 0,
                },
                None => *child,
            })),
            Resume::Again => Err(fault("charging-body-resumed-without-input").into()),
        }
    }
}

/// A host body that requests `callee` with `arguments` as given, however
/// invalid, to exercise the scheduler's request validation.
pub struct BadRequest {
    pub callee: NodeKey,
    pub arguments: Vec<Value>,
}

impl BodyFactory for BadRequest {
    fn begin(&self) -> Box<dyn Body> {
        Box::new(BadRequestBody {
            callee: self.callee.clone(),
            arguments: self.arguments.clone(),
        })
    }
}

struct BadRequestBody {
    callee: NodeKey,
    arguments: Vec<Value>,
}

impl Body for BadRequestBody {
    fn resume(
        &mut self,
        _input: Resume,
        _context: &mut LiveContext,
        _meter: &mut Meter,
    ) -> Result<Transition, BodyFailure> {
        Ok(Transition::Call(ChildRequest {
            callee: self.callee.clone(),
            arguments: self.arguments.clone(),
            site: site(),
        }))
    }
}

/// A host body that yields once (a chargeless `Continue`) before requesting
/// `target` with its argument.
pub struct Yielding {
    pub target: NodeKey,
}

impl BodyFactory for Yielding {
    fn begin(&self) -> Box<dyn Body> {
        Box::new(YieldingBody {
            target: self.target.clone(),
            arguments: Vec::new(),
        })
    }
}

struct YieldingBody {
    target: NodeKey,
    arguments: Vec<Value>,
}

impl Body for YieldingBody {
    fn resume(
        &mut self,
        input: Resume,
        _context: &mut LiveContext,
        _meter: &mut Meter,
    ) -> Result<Transition, BodyFailure> {
        match input {
            Resume::Start(arguments) => {
                self.arguments = arguments;
                Ok(Transition::Continue)
            }
            Resume::Again => Ok(Transition::Call(ChildRequest {
                callee: self.target.clone(),
                arguments: self.arguments.clone(),
                site: site(),
            })),
            Resume::Child(child) => Ok(Transition::Done(*child)),
        }
    }
}

/// A host body that completes with its argument and requests nothing.
pub struct Echo;

impl BodyFactory for Echo {
    fn begin(&self) -> Box<dyn Body> {
        Box::new(EchoBody)
    }
}

struct EchoBody;

impl Body for EchoBody {
    fn resume(
        &mut self,
        input: Resume,
        _context: &mut LiveContext,
        _meter: &mut Meter,
    ) -> Result<Transition, BodyFailure> {
        match input {
            Resume::Start(arguments) => Ok(Transition::Done(Evaluation::completed(
                arguments.into_iter().next().unwrap_or_else(|| int(0)),
            ))),
            Resume::Again | Resume::Child(_) => Err(fault("echo-resumed-after-start").into()),
        }
    }
}

/// A host body that applies `target` to 0 `times` times in sequence, then
/// completes with `times`.
pub struct Sequence {
    pub target: NodeKey,
    pub times: u64,
}

impl BodyFactory for Sequence {
    fn begin(&self) -> Box<dyn Body> {
        Box::new(SequenceBody {
            target: self.target.clone(),
            remaining: self.times,
            times: self.times,
        })
    }
}

struct SequenceBody {
    target: NodeKey,
    remaining: u64,
    times: u64,
}

impl Body for SequenceBody {
    fn resume(
        &mut self,
        input: Resume,
        _context: &mut LiveContext,
        _meter: &mut Meter,
    ) -> Result<Transition, BodyFailure> {
        if let Resume::Child(child) = input {
            if completed_int(&child).is_none() {
                return Ok(Transition::Done(*child));
            }
        }
        if self.remaining == 0 {
            return Ok(Transition::Done(Evaluation::completed(int(self.times))));
        }
        self.remaining -= 1;
        Ok(Transition::Call(ChildRequest {
            callee: self.target.clone(),
            arguments: vec![int(0)],
            site: site(),
        }))
    }
}
