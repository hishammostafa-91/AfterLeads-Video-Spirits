# Data Model: WP-00 — Workstation and repository bootstrap

**Date**: 2026-11-02 · **Spec**: [spec.md](spec.md)

## Entities

None. WP-00 creates infrastructure (workspace manifests, empty crates, an application scaffold); it introduces no persisted entities, files, or schemas. The owner constraint for this work package states: "no IPC interfaces (04) and no data files (03) in this work package", and the card defines no REQ-xx (docs/10 §10.7, WP-00: "Implements: foundations for G-03..G-08 (no product REQ)").

The project data model is defined canonically in docs/03-data.md and first implemented by WP-02 (core types and project model) and WP-03 (storage). Those work packages own `data-model.md` files with real entities.

## Validation rules carried by this package

Although no entities exist, two structural invariants of the skeleton are data-adjacent and verified by the acceptance gates (spec SC-001..SC-006):

- Every source file carries the exact header pair `SPDX-License-Identifier: AGPL-3.0-or-later` and `Copyright (C) 2026 AfterLeads <afterleadseg@gmail.com>` (docs/02 §2.14; docs/00 §0.5).
- Identity values in the scaffold configuration equal the constants of docs/00 §0.5 exactly (product name, bundle identifier, window title).
