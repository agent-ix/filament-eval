---
id: TC-002
title: "Read and evaluate quantity-scaling nodes"
type: TC
relationships:
  - { target: ix://agent-ix/filament-eval/FR-010, type: verifies }
---
# TC-002: Read and evaluate quantity-scaling nodes

## Description

Check the `QuantityScale` and `QuantityScaleDiv` descriptors of FR-010, their
borrowed projections, values, charges and the zero-divisor outcome.

## Test Procedure

1. Project both descriptors with descending stored handles; record visit
   order and compare the borrowed type and location pointers. Compare each
   against `Quantity(Multiply, ..)`, `Quantity(Divide, ..)` and each other.
2. With the unit `ms` (target `s`, scale `1/1000`), `d = 3 ms` and `k = 4`,
   evaluate `QuantityScale(d, 2)`, `(d, k)`, `(d, rational(1, 2))`,
   `(d, decimal(15, 1))`, `QuantityScaleDiv(d, 2)` and `(d, k)`, recording
   values, units and charges.
3. Under direct kernel evaluation, evaluate `QuantityScaleDiv(d, 0)` and
   both nodes over a stopped `quantity` operand.

## Expected Results

Projection visits `quantity` then `factor` once each; no descriptor equals
another. The values are `6 ms`, `12 ms`, `3/2 ms`, `9/2 ms`, `3/2 ms` and
`3/4 ms`, all in `ms`, each with one `unit.identity-read`, one
`unit.rational-arithmetic` and `unit.result-retain`. A zero factor returns
`Undefined::DivisionByZero` at the node with no scaling charge; a stopped
quantity propagates and the factor is not evaluated.

## Status

Ignored stubs in `crates/filament-checked/tests/quantity_scale_nodes.rs`.
