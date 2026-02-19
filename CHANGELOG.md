# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-02-19

### Added
- **Daemon Mode (`--daemon`)**: A continuous monitoring loop that runs indefinitely, checking for new processes at a specified interval.
- **macOS System Auto-Detection**: Automatically identifies and ignores known macOS system directories (`/System/`, `/usr/libexec/`, `/usr/sbin/`, etc.) by default, reducing manual allowlisting by ~87%. Can be disabled with `--no-ignore-system`.
- **System Overload Alerts**: System processes are not killed, but if they exceed CPU or RAM thresholds, safe `system-overload` alerts are generated.
- **Profiles (`--profile NAME`)**: Dynamically loads allowlist configurations from `profiles/NAME.txt`.
- **Audit Logging (`--audit-log`)**: Outputs a structured JSONL and CSV log file specifically for daemon events. Extends the `export.rs` module.
- **Deduplication**: The daemon remembers reported processes to prevent log spam, only reporting them once per lifecycle.
- **Documentation**: New `DAEMON.md` file explaining the daemon mode workflow in detail.

## [0.1.0] - Initial Release
- Basic single-shot execution mode.
- Stealth mode for CPU/RAM anomaly detection.
- Core allowlist functionality (Path, Name, SHA256, UID, PPID).
- CSV and JSONL exporting.
