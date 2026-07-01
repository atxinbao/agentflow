# Software Dev Reference App Tests

更新日期：2026-07-01
执行者：Codex

This directory describes the first-party reference app fixtures used by the
release gate. Runtime tests still live in `crates/**`; this product directory is
the source boundary for Software Dev product definitions.

The v1.0.9 release gate certifies:

- product source exists under `products/software-dev/**`;
- `crates/pack/fixtures/packs/software-dev/**` remains a fixture mirror;
- Core crates do not gain Software Dev authority semantics;
- golden and negative fixtures can be projected into release evidence.
