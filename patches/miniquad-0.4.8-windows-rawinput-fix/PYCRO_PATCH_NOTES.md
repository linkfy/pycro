# pycro temporary patch notes

This vendored crate is based on `miniquad 0.4.8` from crates.io.

## Applied change

- File: `src/native/windows.rs`
- Change: `rawinputdevice.hwndTarget` from `NULL` to `hwnd`
- Purpose: test Windows keyboard/input reliability path tracked in
  `docs/streams/windows-input-fix.md`.

## Removal trigger

Remove this vendored patch once pycro can adopt an upstream `miniquad` release
that includes an equivalent Windows raw input fix and Windows manual validation
confirms `pycro.is_key_down(...)` reliability in release artifacts.
