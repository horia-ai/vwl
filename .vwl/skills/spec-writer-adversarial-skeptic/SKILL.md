---
name: spec-writer-adversarial-skeptic
description: Adversarial spec writer that grills user with rigorous questions or auto-reviews reproducible issues into verifiable tasks
---

# Adversarial Spec Writer & Skeptic Protocol (Gate 0)

## 1. Dual-Path Specification Protocol
- **Path A (New Features & Ambiguous Specs)**: Grilling Interview. The agent refuses subjective or conversational drift ('clean code', 'as discussed') and grills the user until falsifiable commands and system invariants are defined.
- **Path B (Reproducible Bugs & GitHub Issues)**: Ingest via `vwl spec from-issue <id>`. If reproducible with clear test commands (`cargo test`, `npm test`), auto-admit via Skeptic self-review with zero human interrogation.
- **Path C (Roll-Forward Regressions)**: Ingest via `vwl spec repair <incident-id>`. Self-reviews the reproducer test and admits to the Work Frontier.

## 2. Invariant Checklist
1. Falsifiability: Deterministic command assertion.
2. Invariants: Explicit data constraints that must not break.
3. Base SHA: Canonical Git base SHA (HEAD).
4. Self-Containment: Zero conversational dependencies.

## 3. Spec Ingestion
Write `.vwl/ledger/tasks/<task_id>.yaml` and admit via `vwl task add <file> --skeptic` or `vwl spec new`.
