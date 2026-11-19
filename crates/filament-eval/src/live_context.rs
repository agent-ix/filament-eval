// SPDX-License-Identifier: AGPL-3.0-or-later
//! The live-occurrence count lent to a frame for one transition (FR-013).

use filament_exact::{ChargePoint, Incomplete, Integer, LimitKind};

use crate::session::SessionLimits;

/// The live-occurrence count and limit a session lends to one frame transition.
///
/// A body uses this context to count calls it evaluates within one transition.
/// Calls opened with [`Self::open`] remain counted across later transitions
/// until the body closes them. A nested session can inherit the current count
/// with [`Self::nested`], then report its peak with [`Self::reach`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveContext {
    live: u64,
    limit: u64,
    peak: u64,
    open_frames: u64,
    open_occurrences: u64,
}

impl LiveContext {
    /// Make a context for a body called outside any session.
    ///
    /// The count and peak start at zero. Calls opened by the body are bounded
    /// by `limit` and belong only to this direct call.
    #[must_use]
    pub const fn new(limit: u64) -> Self {
        Self {
            live: 0,
            limit,
            peak: 0,
            open_frames: 0,
            open_occurrences: 0,
        }
    }

    /// The current live-occurrence count.
    #[must_use]
    pub const fn live(&self) -> u64 {
        self.live
    }

    /// The live-occurrence limit this context enforces.
    #[must_use]
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    /// The largest live-occurrence count reached through this context.
    #[must_use]
    pub const fn peak(&self) -> u64 {
        self.peak
    }

    /// Count a frame opened inside this transition.
    ///
    /// The frame's occurrence count includes one for the frame itself. On
    /// success, returns that count for a later matching [`Self::close`]. If
    /// adding it would exceed the limit or overflow, the context is unchanged
    /// and the live-occurrence incomplete is returned.
    pub fn open(&mut self, occurrences: &Integer) -> Result<u64, Incomplete> {
        let Some(occurrences_u64) = occurrences.to_u64() else {
            return Err(self.incomplete(occurrences));
        };
        let Some(live) = self.live.checked_add(occurrences_u64) else {
            return Err(self.incomplete(occurrences));
        };
        if live > self.limit {
            return Err(self.incomplete(occurrences));
        }

        self.live = live;
        self.peak = self.peak.max(live);
        self.open_frames = self
            .open_frames
            .checked_add(1)
            .expect("the number of opened frames fits in the live count");
        self.open_occurrences = self
            .open_occurrences
            .checked_add(occurrences_u64)
            .expect("opened occurrences fit in the live count");
        Ok(occurrences_u64)
    }

    /// Remove a frame previously returned by [`Self::open`].
    ///
    /// `occurrences` must be the value returned by an `open` in this context
    /// or a context descended from it; violating this invariant panics.
    pub fn close(&mut self, occurrences: u64) {
        self.live = self
            .live
            .checked_sub(occurrences)
            .expect("a closed frame was previously opened");
        self.open_occurrences = self
            .open_occurrences
            .checked_sub(occurrences)
            .expect("a closed frame was previously opened");
        self.open_frames = self
            .open_frames
            .checked_sub(1)
            .expect("a closed frame was previously opened");
    }

    /// The limits a nested session must use at this point in the transition.
    #[must_use]
    pub const fn nested(&self) -> SessionLimits {
        SessionLimits {
            live_occurrences: self.limit,
            held: self.live,
        }
    }

    /// Include a nested evaluation's peak in this context's peak.
    pub fn reach(&mut self, peak: u64) {
        self.peak = self.peak.max(peak);
    }

    pub(crate) const fn for_transition(
        live: u64,
        limit: u64,
        open_frames: u64,
        open_occurrences: u64,
    ) -> Self {
        Self {
            live,
            limit,
            peak: live,
            open_frames,
            open_occurrences,
        }
    }

    pub(crate) const fn open_frames(&self) -> u64 {
        self.open_frames
    }

    pub(crate) const fn open_occurrences(&self) -> u64 {
        self.open_occurrences
    }

    fn incomplete(&self, next_charge: &Integer) -> Incomplete {
        Incomplete {
            limit_kind: LimitKind::LiveOccurrences,
            limit: self.limit,
            consumed: self.live,
            next_charge: next_charge.clone(),
            charge_point: ChargePoint::FunctionCall,
        }
    }
}
