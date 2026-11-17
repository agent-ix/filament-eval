// SPDX-License-Identifier: AGPL-3.0-or-later
//! FR-010 (TC-002): the quantity-scaling descriptors and their borrowed
//! projections. FSL evaluates the nodes (its FR-680).
//!
use filament_checked::NodeKind;
use filament_exact::{ArithmeticOperator, Value, ValueType};
use filament_semantic_value::location::{ExpressionLocation, ExpressionOwner};
use ix_trace_rs::trace;

#[trace("TC-002", "FR-010-AC-1")]
#[test]
fn scale_reads_map_quantity_then_factor_and_stay_distinct() {
    for divide in [false, true] {
        let stored: NodeKind<usize> = if divide {
            NodeKind::QuantityScaleDiv {
                quantity: 9,
                factor: 4,
            }
        } else {
            NodeKind::QuantityScale {
                quantity: 9,
                factor: 4,
            }
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
        if divide {
            assert_eq!(
                read,
                NodeKind::QuantityScaleDiv {
                    quantity: 29,
                    factor: 24,
                }
            );
            assert_ne!(
                read,
                NodeKind::QuantityScale {
                    quantity: 29,
                    factor: 24
                }
            );
            assert_ne!(read, NodeKind::Quantity(ArithmeticOperator::Divide, 29, 24));
        } else {
            assert_eq!(
                read,
                NodeKind::QuantityScale {
                    quantity: 29,
                    factor: 24,
                }
            );
            assert_ne!(
                read,
                NodeKind::QuantityScaleDiv {
                    quantity: 29,
                    factor: 24
                }
            );
            assert_ne!(
                read,
                NodeKind::Quantity(ArithmeticOperator::Multiply, 29, 24)
            );
        }
    }
}

#[trace("TC-002", "FR-010-AC-2")]
#[test]
fn scale_reads_borrow_metadata_and_carry_no_unit_or_law() {
    let value_type = ValueType::Integer;
    let location = ExpressionLocation::root(ExpressionOwner::Expression);
    let operations: [NodeKind<usize, &Value>; 2] = [
        NodeKind::QuantityScale {
            quantity: 1,
            factor: 2,
        },
        NodeKind::QuantityScaleDiv {
            quantity: 1,
            factor: 2,
        },
    ];
    for operation in operations {
        let read = filament_checked::NodeRead {
            operation: operation.clone(),
            value_type: &value_type,
            location: &location,
        };
        assert!(core::ptr::eq(read.value_type, &value_type));
        assert!(core::ptr::eq(read.location, &location));
        assert!(matches!(
            (&read.operation, &operation),
            (
                NodeKind::QuantityScale {
                    quantity: 1,
                    factor: 2
                },
                NodeKind::QuantityScale {
                    quantity: 1,
                    factor: 2
                }
            ) | (
                NodeKind::QuantityScaleDiv {
                    quantity: 1,
                    factor: 2
                },
                NodeKind::QuantityScaleDiv {
                    quantity: 1,
                    factor: 2
                }
            )
        ));
    }
}
