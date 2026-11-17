// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-009 (TC-001): the tuple-projection and collection-index descriptors.
//!
//! `NodeKind::TupleProject`, `NodeKind::Index` and `IndexedKind` do not exist
//! yet, so each stub states the decided assertion and fails until the code
//! ticket adds them.

use ix_trace_rs::trace;

#[trace("TC-001", "FR-009-AC-1")]
#[test]
#[ignore = "QSL-2171: NodeKind::TupleProject{operand, declaration, position} not implemented yet"]
fn tuple_project_read_maps_its_operand_once_and_keeps_its_position() {
    // NodeRead::from_stored(&NodeKind::TupleProject { operand: NodeId(9),
    // declaration: P, position: 1 }, ..) visits [9], yields operand 29,
    // keeps P and 1, borrows type/location, and differs from Field and Index.
    panic!("NodeKind::TupleProject is not in the checked vocabulary");
}

#[trace("TC-001", "FR-009-AC-2")]
#[test]
#[ignore = "QSL-2171: NodeKind::Index{collection, index, kind: IndexedKind} not implemented yet"]
fn index_read_maps_collection_then_index_and_keeps_its_kind() {
    // For each IndexedKind::{Sequence, OrderedSet, Map}, Index { collection:
    // NodeId(9), index: NodeId(4), kind } visits [9, 4] and keeps kind; the
    // enum has exactly those three kinds.
    panic!("NodeKind::Index and IndexedKind are not in the checked vocabulary");
}

#[trace("TC-001", "FR-009-AC-3")]
#[test]
#[ignore = "QSL-2171: tuple projection evaluation charged as Field not implemented yet"]
fn tuple_project_evaluates_the_position_with_the_field_charge() {
    // (3, true)[0] completes 3 and [1] completes true, each with the charge
    // vector of a one-field record Field read of that value; a stopped
    // operand propagates.
    panic!("tuple projection has no evaluation");
}

#[trace("TC-001", "FR-009-AC-4")]
#[test]
#[ignore = "QSL-2171: sequence, ordered-set and map index evaluation not implemented yet"]
fn index_selects_by_position_or_canonical_key_with_its_charges() {
    // [10, 20, 20][1] = 20, {10, 20, 30}[2] = 30, {"a": 1, "b": 2}["b"] = 2,
    // each one collection.element and one collection.result-retain; the map
    // lookup charges no member walk or member test.
    panic!("collection index has no evaluation");
}

#[trace("TC-001", "FR-009-AC-5")]
#[test]
#[ignore = "QSL-2171: an unmet index obligation returning Undefined::NoElement not implemented yet"]
fn an_unmet_index_obligation_is_undefined_no_element() {
    // Kernel evaluation of seq[-1], seq[3], oset[3] and map["c"] each return
    // Undefined::NoElement at the index node, no value, no collection.element
    // charge; never a default, wrapped or clamped element.
    panic!("collection index has no evaluation");
}
