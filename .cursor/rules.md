# Project Rules

- Minimalism over features. Reduce code and complexity.
- Keep modules small, explicit, and predictable.
- No hidden behavior; clear inputs/outputs.
- Avoid new dependencies unless unavoidable.
- Prefer simple data flows and stable formats.
- Fail fast with precise error messages.
- Every process decision must be intentional.
- Keep output terse and machine-friendly.

## Adapted Philosophy (bspwm-inspired)

- Model processes as a strict tree: parent-child relations are first-class.
- Do one thing well: track, evaluate, and (optionally) kill processes. No UI.
- Configuration via CLI and files only; runtime control via flags.
- Keyboard-first workflows: every action must be scriptable.
- Lightweight and fast: prefer direct syscalls/FFI and minimal allocations.

Reference: https://suckless.org/philosophy/
