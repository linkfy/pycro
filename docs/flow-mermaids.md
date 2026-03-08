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

## Runtime Lifecycle (Phase 2 Active: Direct API Bridge)

```mermaid
flowchart TD
    A[main receives script path]
    A --> B[runtime installs pycro module from api registry]
    B --> C[runtime loads and executes main.py through RustPython]
    C --> D{setup exists}
    D -- yes --> E[call setup once]
    D -- no --> F[skip setup]
    E --> G[DesktopFrameLoop dispatches dt inside Macroquad loop]
    F --> G
    G --> H[call update dt]
    H --> I[python code calls pycro api]
    I --> J[direct API bridge calls EngineBackend callable]
    J --> K[backend executes real Macroquad operation]
    K --> L[rich return value or error mapped back to Python]
    L --> G
```

## Delivery Flow (Current Governance)

```mermaid
flowchart TD
    A[architecture orchestrator] --> B[domain workers]
    B --> C[qa reviewer]
    C --> D{findings present}
    D -- yes --> E[fix and re review]
    E --> C
    D -- no --> F[docs tracker sync tracker and state]
    F --> G[flow visualizer refresh mermaids]
    G --> H[commit eligible]
```
