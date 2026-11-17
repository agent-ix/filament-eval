// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-010 (TC-002): the quantity-scaling descriptors and their borrowed
//! projections. FSL evaluates the nodes (its FR-680).
//!
//! `NodeKind::QuantityScale` and `NodeKind::QuantityScaleDiv` do not exist
//! yet, so each stub states the decided assertion and fails until the code
//! ticket adds them.

use ix_trace_rs::trace;

#[trace("TC-002", "FR-010-AC-1")]
#[test]
#[ignore = "QSL-2256: NodeKind::QuantityScale and QuantityScaleDiv not implemented yet"]
fn scale_reads_map_quantity_then_factor_and_stay_distinct() {
    // QuantityScale { quantity: NodeId(9), factor: NodeId(4) } and
    // QuantityScaleDiv { .. } visit [9, 4]; neither equals
    // Quantity(Multiply | Divide, 9, 4) or the other.
    panic!("the quantity-scaling descriptors are not in the checked vocabulary");
}

#[trace("TC-002", "FR-010-AC-2")]
#[test]
#[ignore = "QSL-2256: NodeKind::QuantityScale and QuantityScaleDiv not implemented yet"]
fn scale_reads_borrow_metadata_and_carry_no_unit_or_law() {
    // Both reads borrow the supplied type and location pointers; the
    // descriptors hold only the two operands.
    panic!("the quantity-scaling descriptors are not in the checked vocabulary");
}
