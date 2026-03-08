# Agent Registry

| Agent | Role | Intended Profile | Owns | Inputs | Outputs |
| --- | --- | --- | --- | --- | --- |
| architecture-orchestrator | default | gpt-5.4, reasoning high | phase selection, delegation, integration, tracker discipline, readiness decisions | concise worker summaries, tracker state, ADR refs | integrated changes, go/no-go decisions, updated tracker/state |
| runtime-worker | worker | gpt-5.4, reasoning high | RustPython embedding, lifecycle dispatch, script isolation, error/reporting surfaces | architecture plan, lifecycle ADRs, registry metadata | runtime code, lifecycle tests, summary evidence |
| platform-worker | worker | gpt-5.4, reasoning medium | Macroquad loop, rendering/input/assets, capability boundaries | platform matrix, architecture plan | platform code, capability evidence, summary risks |
| api-worker | worker | gpt-5.4, reasoning medium | `pycro` module API, stub generation metadata, examples, typing surface | architecture plan, stub ADRs | API metadata, generated stubs, examples, typing evidence |
| example-scenario-worker | worker | gpt-5.4, reasoning medium | playable feature scenarios under `examples/` for manual validation | feature brief, API/platform constraints, existing examples | new/updated example scripts per feature, run instructions, expected behavior checklist |
| qa-reviewer | default | gpt-5.4, reasoning high | post-implementation review gate | diff, tracker evidence, validation results | blocking findings or explicit waiver |
| commit-steward | default | gpt-5.4, reasoning medium | checkpoint commit discipline after green validations | qa outcome, validation evidence, tracker/state sync status | commit created (or blocked with explicit reason), commit sha, concise commit summary |
| docs-tracker | explorer | gpt-5.4, reasoning low | concise state snapshots, phase logs, ADR refs, evidence links | worker summaries, ADR ids, review outcome | updated tracker and machine state |
| flow-visualizer | explorer | gpt-5.4, reasoning medium | Mermaid flow diagrams for lifecycle, API dispatch, and delivery pipeline clarity | architecture plan, tracker/state, worker summaries | versioned Mermaid diagrams and concise narrative for decision/review |

## Worker Summary Contract

Every worker summary must fit this schema:

- `changed_files`: list of paths
- `validation_evidence`: list of commands or test names with pass/fail
- `risks`: list of unresolved concerns
- `follow_ups`: list of next actions
- `adr_refs`: list of ADR ids
- `tracker_refs`: list of tracker task ids
