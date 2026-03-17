# Incident Resolutions Log

Canonical log for cross-cutting incidents discovered outside sequential phase docs.

Use this log when the team needs a stable record of:

- symptom and blast radius,
- root cause,
- resolution commit(s),
- validation evidence,
- rollback/removal trigger.

## Entries

### 2026-03-17 - Windows artifact starts but does not run CLI/script

- id: `incident-2026-03-17-windows-artifact-startup-noop`
- scope: artifacts produced by `develop-artifacts` / `release-artifacts` Windows jobs
- symptom:
  - `pycro.exe` appears to launch and exits immediately on some Windows tester machines.
  - `pycro init <name>` and script execution do not run.
  - no Macroquad window appears, because process exits before runtime lifecycle starts.
- root_cause:
  - Windows artifacts were built with default MSVC dynamic CRT linkage.
  - On environments without the matching Visual C++ runtime DLLs, process startup fails before Rust `main()`.
- resolution:
  - Pin static CRT for Windows artifact builds by setting:
    - `RUSTFLAGS=-C target-feature=+crt-static`
  - Applied in:
    - `.github/workflows/develop-artifacts.yml`
    - `.github/workflows/release-artifacts.yml`
  - landed in commit: `13c1b76`
- validation_evidence:
  - local preflight passed before merge (`fmt`, `clippy -D warnings`, `test`).
  - user confirmed post-fix behavior: "funciona" on Windows artifact run.
- rollback_trigger:
  - if artifact size/runtime constraints require dynamic CRT again and target test environments are guaranteed to ship VC++ runtime.
- follow_ups:
  - add explicit `--help` command support to avoid confusion from current CLI fallback behavior (`--help` is currently parsed as a script path).
