// SPDX-License-Identifier: AGPL-3.0-or-later
//! Checked descriptors for the Bytes length and concatenation operations.

use filament_checked::{NodeKind, NodeRead};
use filament_exact::ValueType;
use filament_semantic_value::location::{ExpressionLocation, ExpressionOwner};
use ix_trace_rs::trace;

#[trace("TC-280", "FR-322-AC-103")]
#[test]
fn bytes_operation_nodes_preserve_and_map_their_operands() {
    let length = NodeKind::<usize>::BytesLength(3);
    let concat = NodeKind::<usize>::BytesConcat(5, 8);
    let mut visited = Vec::new();

    let length_read = length.map_ref(
        |child| {
            visited.push(*child);
            *child + 10
        },
        |_| (),
    );
    let concat_read = concat.map_ref(
        |child| {
            visited.push(*child);
            *child + 10
        },
        |_| (),
    );

    assert_eq!(length_read, NodeKind::<usize, ()>::BytesLength(13));
    assert_eq!(concat_read, NodeKind::<usize, ()>::BytesConcat(15, 18));
    assert_eq!(visited, [3, 5, 8]);
}

#[trace("TC-280", "FR-322-AC-103")]
#[test]
fn bytes_node_read_carries_result_type_and_location() {
    let location = ExpressionLocation::root(ExpressionOwner::Expression);
    let result_type = ValueType::Integer;
    let read = NodeRead {
        operation: NodeKind::<usize, &filament_exact::Value>::BytesLength(7),
        value_type: &result_type,
        location: &location,
    };

    assert!(matches!(read.operation, NodeKind::BytesLength(7)));
    assert_eq!(*read.value_type, ValueType::Integer);
    assert_eq!(read.location, &location);

    let concat_type = ValueType::Bytes;
    let concat_read = NodeRead {
        operation: NodeKind::<usize, &filament_exact::Value>::BytesConcat(4, 9),
        value_type: &concat_type,
        location: &location,
    };
    assert!(matches!(concat_read.operation, NodeKind::BytesConcat(4, 9)));
    assert_eq!(*concat_read.value_type, ValueType::Bytes);
}
