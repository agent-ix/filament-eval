// SPDX-License-Identifier: AGPL-3.0-or-later
//! The session and its scheduler: one explicit heap task stack driving
//! interpreted frames and registered host bodies through one transition at a
//! time.

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem;

use filament_exact::{
    Charge, ChargePoint, Incomplete, Integer, LimitKind, Meter, NodeKey, Outcome, Value,
};
use filament_semantic_value::catalog::InternalFault;
use filament_semantic_value::location::ExpressionLocation;

use crate::body::{Body, ChildRequest, Resume, Transition};
use crate::closure::{CheckedClosure, RequestRefusal};
use crate::live_context::LiveContext;
use crate::registry::Bindings;
use crate::result::{CallFailure, Evaluation, FamilyOutcome};

/// The default live-occurrence limit of a session: 1,048,576, the default of
/// FSL's `evaluation.live_occurrences` setting (FR-011).
pub const DEFAULT_LIVE_OCCURRENCES: u64 = 1_048_576;

/// The bounds a session enforces itself, bound to it for life at `Begin`
/// (FR-011). The caller supplies them; this crate reads no ambient setting.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SessionLimits {
    /// The largest live-occurrence count the session may hold: the sum, over
    /// its frames, of one per frame plus the occurrences (`occ`) of the values
    /// bound in it at entry. Every value is a valid limit; 0 stops the first
    /// call.
    pub live_occurrences: u64,
    /// The count already held by an enclosing application when this session
    /// begins (FR-013). Use zero for an independent application.
    pub held: u64,
}

impl Default for SessionLimits {
    fn default() -> Self {
        Self {
            live_occurrences: DEFAULT_LIVE_OCCURRENCES,
            held: 0,
        }
    }
}

/// What the scheduler enters next, before any frame of it exists.
pub(crate) enum Entry<C: CheckedClosure> {
    /// The root call, which `Begin` already validated.
    Root {
        callable: NodeKey,
        arguments: Vec<Value>,
        site: ExpressionLocation,
    },
    /// A body's saved request, validated when it is entered.
    Request(ChildRequest),
    /// Evaluate a checked expression root.
    Expression {
        expression: C::Expression,
        parameters: Vec<Value>,
    },
}

enum Engine<C: CheckedClosure> {
    Interpreted(C::Task),
    Host(Box<dyn Body>),
}

/// One suspended or running body, and what it is resumed with next.
struct Frame<C: CheckedClosure> {
    engine: Engine<C>,
    input: Resume,
    /// This frame's live occurrences, counted while it is on the stack.
    occurrences: u64,
    /// Occurrences opened by this frame inside its transitions and still live.
    carried_occurrences: u64,
    /// Frames opened by this frame inside its transitions and still live.
    carried_frames: u64,
}

/// The result of one [`Session::step`].
#[must_use = "dropping a runnable session abandons the application"]
pub enum Progress<'a, C: CheckedClosure> {
    /// The application is still runnable: step the session again.
    Runnable(Session<'a, C>),
    /// The application ended, with the evaluation or the call failure. The
    /// session is gone; suspended frames were released without executing
    /// their continuations.
    Terminal(Result<Evaluation, CallFailure>),
}

/// The opaque, live session of one application.
///
/// A session retains its task stack, call frames, lexical bindings, source
/// locations, losses and accounting across steps. It is bound for life to the
/// closure and sealed registry it began from, and to its environment and
/// meter. It cannot be copied or rebound, and [`Self::step`] consumes it, so a session cannot be resumed after a
/// terminal result.
///
/// Dropping a session releases its suspended frames and executes no
/// continuation.
pub struct Session<'a, C: CheckedClosure> {
    closure: Arc<C>,
    bindings: Arc<Bindings>,
    environment: &'a C::Environment,
    meter: &'a mut Meter,
    /// What to enter at the next transition, when set.
    entry: Option<Entry<C>>,
    /// The task stack: the callee is on top, its suspended callers below.
    frames: Vec<Frame<C>>,
    limits: SessionLimits,
    /// The live-occurrence count of `held`, `frames` and their in-transition
    /// frames.
    live: u64,
    /// The largest live count reached by this session or nested transitions.
    peak: u64,
}

impl<'a, C: CheckedClosure> Session<'a, C> {
    pub(crate) fn new(
        closure: Arc<C>,
        bindings: Arc<Bindings>,
        entry: Entry<C>,
        limits: SessionLimits,
        environment: &'a C::Environment,
        meter: &'a mut Meter,
    ) -> Self {
        Self {
            closure,
            bindings,
            environment,
            meter,
            entry: Some(entry),
            frames: Vec::new(),
            limits,
            live: limits.held,
            peak: limits.held,
        }
    }

    /// The current live-occurrence count, including `held` and frames opened
    /// inside transitions.
    #[must_use]
    pub const fn live_occurrences(&self) -> u64 {
        self.live
    }

    /// The limits bound to this session at `Begin`.
    #[must_use]
    pub const fn limits(&self) -> SessionLimits {
        self.limits
    }

    /// Advance the application by one finite saved transition.
    ///
    /// A transition commits its state before returning. Yielding, saving a
    /// continuation, pushing or popping a frame and delivering a child add
    /// no charge; the only charges are the call charge of an entered
    /// application and the kernel charges its body makes.
    pub fn step(mut self) -> Progress<'a, C> {
        match self.transition() {
            Ok(None) => Progress::Runnable(self),
            Ok(Some(mut evaluation)) => {
                evaluation.peak_live_occurrences = self.peak;
                Progress::Terminal(Ok(evaluation))
            }
            Err(failure) => Progress::Terminal(Err(failure)),
        }
    }

    /// Drive the application to its terminal result, as the synchronous
    /// `call` and `evaluate` do.
    ///
    /// # Errors
    ///
    /// The call failure that ended the application.
    pub fn run(self) -> Result<Evaluation, CallFailure> {
        let mut session = self;
        loop {
            match session.step() {
                Progress::Runnable(next) => session = next,
                Progress::Terminal(result) => return result,
            }
        }
    }

    fn transition(&mut self) -> Result<Option<Evaluation>, CallFailure> {
        if let Some(entry) = self.entry.take() {
            return self.enter(entry);
        }
        let live = self.live;
        let limit = self.limits.live_occurrences;
        let Some(frame) = self.frames.last_mut() else {
            return Err(fault("session-without-frame").into());
        };
        let mut context = LiveContext::for_transition(
            live,
            limit,
            frame.carried_frames,
            frame.carried_occurrences,
        );
        let input = mem::replace(&mut frame.input, Resume::Again);
        let transition = match &mut frame.engine {
            Engine::Interpreted(task) => {
                let delivered = match input {
                    Resume::Child(evaluation) => Some(*evaluation),
                    Resume::Again => None,
                    Resume::Start(_) => {
                        return Err(fault("interpreted-frame-resumed-with-start").into());
                    }
                };
                self.closure
                    .advance(task, delivered, &mut context, self.environment, self.meter)?
            }
            Engine::Host(body) => body.resume(input, &mut context, self.meter)?,
        };
        self.live = context.live();
        self.peak = self.peak.max(context.peak());
        frame.carried_frames = context.open_frames();
        frame.carried_occurrences = context.open_occurrences();
        match transition {
            Transition::Continue => {}
            Transition::Call(request) => {
                // The request is saved; the next transition validates,
                // charges and pushes the callee.
                self.entry = Some(Entry::Request(request));
            }
            Transition::Done(mut evaluation) => {
                if is_live_occurrence_incomplete(&evaluation) {
                    evaluation.frame_depth = evaluation.frame_depth.saturating_add(1);
                    evaluation.peak_live_occurrences = self.peak;
                    return Ok(Some(evaluation));
                }
                let completed = is_completed(&evaluation);
                if completed {
                    evaluation.frame_depth = 0;
                }
                self.pop();
                match self.frames.last_mut() {
                    Some(caller) => {
                        if !completed {
                            evaluation.frame_depth = evaluation.frame_depth.saturating_add(1);
                        }
                        evaluation.peak_live_occurrences = self.peak;
                        caller.input = Resume::Child(Box::new(evaluation));
                    }
                    None => return Ok(Some(evaluation)),
                }
            }
        }
        Ok(None)
    }

    /// Enter an application: validate, charge `function.call`, bind and push
    /// its frame, in that order. An expression root has no call charge.
    fn enter(&mut self, entry: Entry<C>) -> Result<Option<Evaluation>, CallFailure> {
        match entry {
            Entry::Root {
                callable,
                arguments,
                site,
            } => self.enter_call(callable, arguments, Some(site)),
            Entry::Request(ChildRequest {
                callee,
                arguments,
                site,
            }) => {
                // The caller's own input was admitted at `Begin`, so a request
                // that fails admission is a broken body, not an input refusal.
                match self
                    .closure
                    .admit_request(&callee, &arguments, self.environment)
                {
                    Ok(()) => {}
                    Err(RequestRefusal::NotAdmitted) => {
                        return Err(fault("child-request-not-admitted").into());
                    }
                    Err(RequestRefusal::Storage { parameter, failure }) => {
                        return Err(CallFailure::Storage { parameter, failure });
                    }
                }
                self.enter_call(callee, arguments, Some(site))
            }
            Entry::Expression {
                expression,
                parameters,
            } => {
                // An expression root makes no call, so its frame is counted
                // but not tested; each call it reaches is tested with it
                // included. A count past `u64::MAX` saturates there, which
                // still exceeds every limit at the first call.
                let occurrences = frame_occurrences(&parameters).to_u64().unwrap_or(u64::MAX);
                let task =
                    self.closure
                        .bind_expression(&expression, parameters, self.environment)?;
                self.push(Frame {
                    engine: Engine::Interpreted(task),
                    input: Resume::Again,
                    occurrences,
                    carried_occurrences: 0,
                    carried_frames: 0,
                });
                Ok(None)
            }
        }
    }

    /// Charge `function.call` for an admitted application, test its frame
    /// against the live-occurrence limit, then bind and push the frame: a
    /// registered host body when `callable` has one, otherwise the closure's
    /// interpreted body.
    fn enter_call(
        &mut self,
        callable: NodeKey,
        arguments: Vec<Value>,
        site: Option<ExpressionLocation>,
    ) -> Result<Option<Evaluation>, CallFailure> {
        if let Err(denied) = self.meter.charge(Charge::new(ChargePoint::FunctionCall)) {
            return Ok(Some(Self::incomplete(denied, site, self.caller_depth())));
        }
        let next_charge = frame_occurrences(&arguments);
        let Some(occurrences) = self.admit_frame(&next_charge) else {
            // The granted call charge stays spent; the callee is not bound.
            let stopped = Incomplete {
                limit_kind: LimitKind::LiveOccurrences,
                limit: self.limits.live_occurrences,
                consumed: self.live,
                next_charge,
                charge_point: ChargePoint::FunctionCall,
            };
            return Ok(Some(Self::incomplete(stopped, site, self.caller_depth())));
        };
        let frame = match self.bindings.get(&callable) {
            Some(factory) => Frame {
                engine: Engine::Host(factory.begin()),
                input: Resume::Start(arguments),
                occurrences,
                carried_occurrences: 0,
                carried_frames: 0,
            },
            None => Frame {
                engine: Engine::Interpreted(self.closure.bind_call(
                    &callable,
                    arguments,
                    self.environment,
                )?),
                input: Resume::Again,
                occurrences,
                carried_occurrences: 0,
                carried_frames: 0,
            },
        };
        self.push(frame);
        Ok(None)
    }

    /// The new frame's occurrences when adding them keeps the live count
    /// within the limit, or `None` when it would exceed it. The arithmetic
    /// is checked: a count past `u64::MAX` exceeds every limit.
    fn admit_frame(&self, occurrences: &Integer) -> Option<u64> {
        let occurrences = occurrences.to_u64()?;
        let total = self.live.checked_add(occurrences)?;
        (total <= self.limits.live_occurrences).then_some(occurrences)
    }

    /// Push `frame`, adding its occurrences to the live count.
    fn push(&mut self, frame: Frame<C>) {
        self.live = self.live.saturating_add(frame.occurrences);
        self.peak = self.peak.max(self.live);
        self.frames.push(frame);
    }

    /// Pop the top frame, removing its occurrences from the live count. The
    /// live count is the sum of the pushed frames, so it never saturates.
    fn pop(&mut self) {
        if let Some(frame) = self.frames.pop() {
            self.live = self
                .live
                .saturating_sub(frame.occurrences)
                .saturating_sub(frame.carried_occurrences);
        }
    }

    /// The number of caller frames above the call being entered.
    fn caller_depth(&self) -> u32 {
        let depth = u64::try_from(self.frames.len().saturating_sub(1))
            .unwrap_or(u64::MAX)
            .saturating_add(self.frames.iter().fold(0_u64, |total, frame| {
                total.saturating_add(frame.carried_frames)
            }));
        u32::try_from(depth).unwrap_or(u32::MAX)
    }

    /// The located incomplete result of a denied charge. A charge the meter's
    /// cancellation handle denied is a denied charge like any other
    /// (FR-262): earlier spend is retained, the denied charge is unrecorded,
    /// and the handle's `tripped()` reports the cause for the caller to read.
    fn incomplete(
        denied: Incomplete,
        site: Option<ExpressionLocation>,
        frame_depth: u32,
    ) -> Evaluation {
        Evaluation {
            outcome: FamilyOutcome::Evaluated(Outcome::Incomplete(denied)),
            location: site,
            losses: Vec::new(),
            frame_depth,
            peak_live_occurrences: 0,
        }
    }
}

/// One for the frame plus `occ` of each value bound in it at entry (FR-011).
fn frame_occurrences(values: &[Value]) -> Integer {
    values
        .iter()
        .fold(Integer::one(), |sum, value| sum.add(&value.occ()))
}

fn is_completed(evaluation: &Evaluation) -> bool {
    matches!(
        &evaluation.outcome,
        FamilyOutcome::Evaluated(Outcome::Completed(_))
    )
}

fn is_live_occurrence_incomplete(evaluation: &Evaluation) -> bool {
    matches!(
        &evaluation.outcome,
        FamilyOutcome::Evaluated(Outcome::Incomplete(incomplete))
            if incomplete.limit_kind == LimitKind::LiveOccurrences
    )
}

/// A broken scheduler invariant, never a caller-input refusal.
fn fault(invariant: &'static str) -> InternalFault {
    InternalFault::new("filament-eval", invariant)
}
