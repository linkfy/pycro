# Design

## Implementation Approach

- Implement the first concrete target adapter: `desktop`.
- Reuse the shared project bundle generated from the phase 14 contract rather than inventing a desktop-only project format.
- Treat desktop output as a packaged runtime plus bundle payload staged into `dist/`.
- Keep the current runtime path intact for direct script execution and only activate packaged-project logic inside the `pycro project` path.
- Use desktop-specific smoke tests to validate imports, assets, and startup behavior.

## ADR And Contract Alignment

- If desktop packaging requires changing build outputs or release artifact policy, record the contract impact through ADR.
