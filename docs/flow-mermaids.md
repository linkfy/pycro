# Flow Mermaids

This document is maintained by `flow-visualizer`. It is the fast-read visual reference for runtime lifecycle and API dispatch.

## Src Module Traversal (Current Code)

```mermaid
flowchart TD
    A[src/main.rs main]
    A --> B[script_path_from_args]
    A --> C[run_script_contract]
    C --> D[src/runtime.rs RuntimeConfig]
    C --> E[src/runtime.rs ScriptRuntime new with RustPythonVm]
    E --> F[load_main]
    E --> G[update dt via DesktopFrameLoop]
    F --> H[src/api.rs registration_plan]
    C --> I[src/api.rs module_spec]
    C --> J[print contract output]
    K[src/lib.rs re exports modules] --> A
    K --> H
    K --> D
    L[src/backend.rs EngineBackend + DesktopFrameLoop] --> K
```

## Stub Generation Traversal (Current Code)

```mermaid
flowchart LR
    A[src/bin/generate_stubs.rs]
    A --> B[module_spec from src/api.rs]
    B --> C[render_stub in src/api.rs]
    C --> D[python/pycro/__init__.pyi]
```

## Runtime Lifecycle (Phase 5 Active: Draw Batch + Submit Render)

```mermaid
flowchart TD
    A[main receives script path]
    A --> B[runtime installs pycro module from api registry]
    B --> C[runtime configures entry-script directory on sys.path]
    C --> D[runtime installs stdlib compatibility modules math/os when no local sidecar collision]
    D --> E[runtime preloads sidecar modules from script directory]
    E --> F[runtime loads and executes main.py through RustPython]
    F --> G{setup exists}
    G -- yes --> H[call setup once]
    G -- no --> I[skip setup]
    H --> J[DesktopFrameLoop dispatches dt inside Macroquad loop]
    I --> J
    J --> K[call update dt]
    K --> L[python code calls pycro api or submit_render commands]
    L --> M[direct API bridge queues draw ops in runtime draw batch]
    M --> N{update succeeded}
    N -- no --> O[discard queued draw batch and return runtime error]
    N -- yes --> P[main calls runtime flush_draw_batch once per frame]
    P --> Q[runtime replays queued ops into EngineBackend]
    Q --> R[backend executes real Macroquad operations]
    R --> S[rich return value or error mapped back to Python]
    S --> J
```

## Submit Render Dispatch (Current Runtime Contract)

```mermaid
flowchart LR
    A[python update calls pycro submit_render commands]
    A --> B[runtime parses commands with rollback mark]
    B --> C{all commands valid}
    C -- no --> D[rollback queued ops and raise ValueError]
    C -- yes --> E[enqueue compacted batch entries]
    E --> F[optional circle cache fast path on repeated layout]
    F --> G[frame end flush_draw_batch dispatches to backend]
```

## Delivery Flow (Current Governance And Manual Playtest Gate)

```mermaid
flowchart TD
    A[architecture orchestrator] --> B[domain workers]
    A --> C[example-scenario-worker]
    B --> D[qa reviewer]
    C --> E[user manual playtest feedback]
    D --> F{findings present}
    F -- yes --> G[fix and re review]
    G --> D
    F -- no --> H[docs tracker sync tracker and state]
    E --> I{feedback recorded}
    I -- no --> C
    I -- yes --> H
    H --> J[flow visualizer refresh mermaids]
    J --> K[commit steward creates checkpoint commit]
```
