# KolibriOS AI Release Readiness

## What is runnable now

The repository can now be exercised as a **developer preview** rather than a fully bootable operating system release:

- `cnd_orchestrator/cnd_orchestrator.py` can run as the Python orchestration layer.
- `unified_ai_agent/unified_mind/main.py` can now start correctly after fixing the entry-point syntax error.
- Python test suites can run in a clean environment without requiring `pytest-asyncio` or `pytest-cov` to be preinstalled.
- The Rust workspace manifest now parses again, which unblocks targeted Cargo inspection and future crate-by-crate fixes.

## What still blocks a production launch

### Rust workspace / build gaps

- Several Rust application crates still need dependency and integration cleanup before a full workspace build will succeed.
- A minimal shared protocol crate now exists to unblock dependency resolution, but it is still only a placeholder and not a finalized cross-component contract.
- The current root `Makefile` and release scripts assume a more complete boot pipeline than the repository currently implements.

### OS release gaps

- No verified bootable ISO has been produced from this repository state.
- No validated APK/mobile packaging flow has been demonstrated end-to-end.
- QEMU/AVD flows are documented, but the repository does not yet provide a confirmed one-command release build for both targets.

## Recommended next milestones

1. Normalize Rust crate dependencies so `cargo check --workspace` passes.
2. Restore or replace the missing shared protocol crate used by the GUI/client code.
3. Produce one verified runnable target first:
   - either a **developer preview** (Python orchestrator + Unified Mind CLI), or
   - a **bootable kernel demo** in QEMU.
4. Only after one target is stable, wire release packaging (`ISO`, `APK`, checksums, GitHub release upload).

## Definition of “launchable” for the next pass

For this project, a realistic launch target should mean all of the following are true:

- a documented `setup` command completes on a clean machine,
- a documented `run` command starts the selected target,
- automated tests pass for the shipped components,
- release scripts do not contain embedded secrets,
- release artifacts are generated from real builds rather than placeholders.
