# Design

## Implementation Approach

- Add a new CLI namespace rooted at `pycro project` rather than overloading the current script-running entry path.
- Add a root alias `pycro build` that maps to `pycro project build` for shorter local workflows.
- Model project builds around external path input (`--project <path>` or positional `<path>`) so game content stays external to the `pycro` source tree.
- Define a canonical project contract once and make all future target phases consume that same contract.
- Define a canonical `project bundle` as the normalized internal representation produced before any target-specific packaging.
- Introduce a resource/provider abstraction in the design so targets can load scripts/assets from filesystem, packaged bundles, or platform-native asset containers.
- Keep the first target-specific implementation out of this phase; the goal here is to lock interfaces and sequencing.

## ADR And Contract Alignment

- This roadmap changes build/packaging strategy and future platform guarantees, so ADR coverage is expected when implementation starts.
