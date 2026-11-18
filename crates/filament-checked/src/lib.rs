// SPDX-License-Identifier: AGPL-3.0-or-later
//! `filament-checked`: the checked terms an evaluator reads, at ADR-011 layer C.
//!
//! The checker builds checked terms; the evaluator and the simulation read
//! them. This crate holds the terms so a reader never depends on the checker.
//! It is `no_std` with `alloc`, with a default-on `std` feature, and it
//! forbids unsafe code.
//!
//! A checked term's existence claims that the checker admitted it, so every
//! field is private and the only constructors are the functions of the
//! `construct` module, compiled only under the `construct` feature.
#![no_std]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

pub mod integer_division;
pub mod key;
pub mod node;

pub use node::{NodeKind, NodeRead};

/// The only constructors of a checked term. Enabled by the checker alone.
#[cfg(feature = "construct")]
pub mod construct {
    use alloc::vec::Vec;

    use filament_exact::DivisionProfile;
    use filament_exact::EffectiveId;

    use crate::integer_division::AdmittedIntegerDivision;
    use crate::NodeKind;

    /// A pending reference literal cannot have an empty identity.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ReferenceLiteralError {
        /// The decoded key contained no bytes.
        EmptyIdentity,
    }

    /// Construct a subject-independent reference literal node.
    ///
    /// The identity is the UTF-8 encoding of the literal's decoded string
    /// key. Subject binding later determines whether it names a fixed or
    /// created object.
    pub fn reference_literal(
        object_type: EffectiveId,
        identity: Vec<u8>,
    ) -> Result<NodeKind, ReferenceLiteralError> {
        if identity.is_empty() {
            return Err(ReferenceLiteralError::EmptyIdentity);
        }
        Ok(NodeKind::ReferenceLiteral {
            object_type,
            identity,
        })
    }

    /// The admitted `div`/`rem` law `profile`. Only the definition admission
    /// calls it (FR-400, "Checking authority" rule 3).
    pub fn admitted_integer_division(profile: DivisionProfile) -> AdmittedIntegerDivision {
        AdmittedIntegerDivision { profile }
    }
}
