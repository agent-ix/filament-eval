---
id: TC-002
title: "Read quantity-scaling nodes"
type: TC
relationships:
  - { target: ix://agent-ix/filament-eval/FR-010, type: verifies }
---
# TC-002: Read quantity-scaling nodes

## Description

Check the `QuantityScale` and `QuantityScaleDiv` descriptors of FR-010 and
their borrowed projections. Their evaluation is FSL's (FR-680) and is not run
here.

## Test Procedure

1. Project both descriptors with descending stored handles; record the visit
   order and compare the borrowed type and location pointers.
2. Compare each against `Quantity(Multiply, ..)`, `Quantity(Divide, ..)` and
   each other, and destructure each to check it holds only its two operands.

## Expected Results

Projection visits `quantity` then `factor` once each and keeps the borrowed
pointers; no descriptor equals another, and neither has a unit, law, mode,
member or magnitude field.

## Status

Ignored stubs in `crates/filament-checked/tests/quantity_scale_nodes.rs`.
