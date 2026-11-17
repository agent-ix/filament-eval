// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-010 (TC-002): the quantity-scaling descriptors.
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

#[trace("TC-002", "FR-010-AC-3")]
#[test]
#[ignore = "QSL-2256: quantity scaling values and charges (FSpec FR-142-AC-41) not implemented yet"]
fn scaling_keeps_the_unit_and_charges_one_identity_read_and_arithmetic() {
    // d = 3 ms, k = 4: scale(d, 2) = 6 ms, scale(d, k) = 12 ms,
    // scale(d, rational(1, 2)) = 3/2 ms, scale(d, decimal(15, 1)) = 9/2 ms,
    // scale_div(d, 2) = 3/2 ms, scale_div(d, k) = 3/4 ms; each one
    // unit.identity-read, one unit.rational-arithmetic, unit.result-retain.
    panic!("quantity scaling has no evaluation");
}

#[trace("TC-002", "FR-010-AC-4")]
#[test]
#[ignore = "QSL-2256: a zero scale_div factor returning Undefined::DivisionByZero not implemented yet"]
fn a_zero_divisor_is_undefined_before_any_scaling_charge() {
    // Kernel evaluation of scale_div(d, 0) returns DivisionByZero at the node
    // with no scaling charge; a stopped quantity operand propagates and the
    // factor is not evaluated.
    panic!("quantity scaling has no evaluation");
}
