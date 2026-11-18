// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-009 (TC-001): the tuple-projection and collection-index descriptors and
//! their borrowed projections. FSL evaluates the nodes (its FR-680).
//!
use filament_checked::node::{IndexedKind, RecordField};
use filament_checked::NodeKind;
use filament_exact::{NodeKey, PathSegment, ValueType};
use filament_semantic_value::location::{ExpressionLocation, ExpressionOwner};
use ix_trace_rs::trace;

#[trace("TC-001", "FR-009-AC-1")]
#[test]
fn tuple_project_read_maps_its_operand_once_and_keeps_its_position() {
    let declaration = NodeKey::new("fixture", vec![PathSegment::Text("tuple".into())], 0).unwrap();
    let stored: NodeKind<usize> = NodeKind::TupleProject {
        operand: 9,
        declaration: declaration.clone(),
        position: 1,
    };
    let mut visited = Vec::new();
    let read = stored.map_ref(
        |child| {
            visited.push(*child);
            *child + 20
        },
        |_| (),
    );
    assert_eq!(visited, [9]);
    assert_eq!(
        read,
        NodeKind::TupleProject {
            operand: 29,
            declaration: declaration.clone(),
            position: 1,
        }
    );
    let value_type = ValueType::Boolean;
    let location = ExpressionLocation::root(ExpressionOwner::Expression);
    let borrowed = filament_checked::NodeRead {
        operation: NodeKind::TupleProject {
            operand: 29,
            declaration: declaration.clone(),
            position: 1,
        },
        value_type: &value_type,
        location: &location,
    };
    assert!(core::ptr::eq(borrowed.value_type, &value_type));
    assert!(core::ptr::eq(borrowed.location, &location));
    assert_ne!(
        read,
        NodeKind::Field {
            operand: 29,
            field: RecordField::new(
                NodeKey::new("fixture", vec![PathSegment::Text("record".into())], 0).unwrap(),
                "field",
            ),
            optional: false,
        }
    );
    assert_ne!(
        read,
        NodeKind::TupleProject {
            operand: 29,
            declaration,
            position: 2,
        }
    );
}

#[trace("TC-001", "FR-009-AC-2")]
#[test]
fn index_read_maps_collection_then_index_and_keeps_its_kind() {
    for kind in [
        IndexedKind::Sequence,
        IndexedKind::OrderedSet,
        IndexedKind::Map,
    ] {
        let stored: NodeKind<usize> = NodeKind::Index {
            collection: 9,
            index: 4,
            kind,
        };
        let mut visited = Vec::new();
        let read = stored.map_ref(
            |child| {
                visited.push(*child);
                *child + 20
            },
            |_| (),
        );
        assert_eq!(visited, [9, 4]);
        assert_eq!(
            read,
            NodeKind::Index {
                collection: 29,
                index: 24,
                kind,
            }
        );
    }
}
