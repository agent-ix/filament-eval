// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-009 (TC-001): the tuple-projection and collection-index descriptors and
//! their borrowed projections. FSL evaluates the nodes (its FR-680).
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
