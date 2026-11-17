---
id: FR-010
title: Read the checked quantity-scaling operations
type: FR
relationships:
  - target: ix://agent-ix/filament-specification/FR-142
    type: implements
  - target: ix://agent-ix/filament-specification/FR-322
    type: implements
---
# FR-010: Read the checked quantity-scaling operations

## Description

The checked node vocabulary SHALL represent quantity scaling by an exact
number as `QuantityScale { quantity, factor }` and quantity division by an
exact number as `QuantityScaleDiv { quantity, factor }`. Both are distinct from
two-quantity arithmetic (`Quantity`), quantity negation and quantity power.
Each descriptor carries its two operands and no unit, law, mode, member or
magnitude field.

## Rationale

A number has no unit, so scaling keeps the quantity operand's own unit, while
two-quantity multiplication forms a compound unit over root units. Separate
descriptors keep the two results apart for every evaluator, and fixing the
operand order as quantity then factor makes `2 * d` and `d * 2` one node.

## Behavior

1. `QuantityScale` represents an admitted `filament.op.quantity.scale`
   application (`q * n` and `n * q`) and `QuantityScaleDiv` an admitted
   `filament.op.quantity.scale_div` application (`q / n`). `quantity` is the
   application's first argument, a non-affine, non-carried `Quantity<u>`
   operand, and `factor` its second, an exact number (`Integer`, `Int[..]`,
   `Rational[..]` or `Decimal[..]`). Contract-IR projects each application
   into its node with the argument handles in argument order, whatever the
   source order.
2. A borrowed projection SHALL visit `quantity` before `factor`, each exactly
   once, retain both mapped handles in that order, and borrow the supplied
   checked result type and location unchanged. The result type is the
   quantity operand's `Quantity<u>` type (`result operand:0`).
3. The descriptor SHALL not choose a unit, convert a magnitude, evaluate the
   operation or infer admission from its shape.
4. `QuantityScale` carries no obligation. `QuantityScaleDiv` carries the
   FSpec FR-146 nonzero obligation on `factor`, which a linked body has
   proved; the descriptor holds no obligation field.
5. This crate does not evaluate either node: no evaluation entry of
   `filament-eval` interprets a `NodeKind`. FSL's checked closure evaluates
   both under FSL FR-680, with the operand order, unit-preserving exact
   values, `unit.identity-read`, `unit.rational-arithmetic` and
   `unit.result-retain` charges and zero-divisor outcome of FSpec FR-142
   "Quantity scaling" and FR-142-AC-41.

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-010-AC-1 | `QuantityScale` and `QuantityScaleDiv` reads each retain their distinct operation and map `quantity` before `factor`, each exactly once, including when the stored handles have descending indices; neither compares equal to `Quantity` multiplication or division over the same handles, nor to each other. | Test (TC-002) |
| FR-010-AC-2 | Each read borrows the supplied checked result type and location without reconstructing either; neither descriptor has a unit, law, mode, member or magnitude field. | Test (TC-002) |

## Dependencies

- QSpec FR-142 "Quantity scaling" and FR-142-AC-41 own the values, the charges and the zero-divisor outcome.
- QSpec FR-322 "Quantity scaling" owns the application shapes, argument order and `result operand:0` typing.
- QSpec FR-146 owns the nonzero obligation.
- Contract-IR owns admission and the projection of both applications into these nodes.
- FSL FR-680 owns lowering of `q * n`, `n * q` and `q / n` and the evaluation of both nodes, with its values, charges and zero-divisor outcome; `filament-semantic-value` owns the quantity value operations it uses.
