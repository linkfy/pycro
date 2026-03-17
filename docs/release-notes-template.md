# Release Notes Template

Use this template when editing the final GitHub release notes after Release Please creates the tag.

Recommended for cumulative patch lines (for example `0.4.1` onward rolled into `0.4.4`).

```md
## pycro_cli v<version>

This release supersedes the <older-range> line and is the recommended stable version.

### Cumulative improvements since <start-version>

- <area>: <summary of user-facing reliability/feature improvement>
- <area>: <summary>
- <area>: <summary>

### Included fixes in <version>

- **ci:** <fix summary>
- **runtime:** <fix summary>
- **input:** <fix summary>
- **windows-input:** <fix summary>

### Validation baseline

- `cargo fmt --all --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
```

## Operator Checklist

1. Confirm release artifacts workflow finished successfully for the new tag.
2. Edit the new release notes using this template.
3. If the previous release should lose prominence, mark it as `Pre-release` and add `Superseded` note.
4. Verify `Latest` points to the new stable tag.
5. Keep the summary in English.
