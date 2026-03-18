#!/usr/bin/env python3
"""Validate repository governance invariants."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
AGENTS = ROOT / "AGENTS.md"
TRACKER = ROOT / "docs" / "task-tracker.txt"
STATE = ROOT / "state" / "repo-state.json"
PHASES_DIR = ROOT / "docs" / "phases"
PHASES_README = PHASES_DIR / "README.md"

REQUIRED_DOC_REFS = [
    "docs/project-vision.md",
    "docs/architecture-plan.md",
    "docs/phases/README.md",
    "docs/task-tracker.txt",
    "docs/agent-playbook.md",
    "docs/agent-registry.md",
    "docs/agents/agent-skills.md",
    "docs/agents/orchestration-contract.md",
    "docs/adr/README.md",
    "docs/platform-capability-matrix.md",
    "docs/validation-policy.md",
    "docs/branch-commit-workflow.md",
    "state/repo-state.json",
]

REQUIRED_PHASE_FILES = {
    "README.md",
    "requirements.md",
    "design.md",
    "implementation.md",
    "interactive-refinement.md",
}

REQUIRED_PHASE_HEADINGS = {
    "requirements.md": "## Acceptance Criteria",
    "design.md": "## Implementation Approach",
    "implementation.md": "## Execution Steps",
}


def parse_table_row(line: str) -> list[str]:
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def parse_markdown_table(path: Path, section_header: str) -> list[dict[str, str]]:
    content = path.read_text(encoding="utf-8").splitlines()
    in_task_section = False
    headers: list[str] = []
    rows: list[dict[str, str]] = []

    for raw in content:
        line = raw.strip()
        if line == section_header:
            in_task_section = True
            continue

        if in_task_section and line.startswith("## ") and line != section_header:
            break

        if not in_task_section or not line.startswith("|"):
            continue

        cells = parse_table_row(line)
        if not cells:
            continue

        if not headers:
            headers = cells
            continue

        if all(set(cell) <= {"-", " ", ":", "="} for cell in cells):
            continue

        if len(cells) != len(headers):
            continue

        rows.append(dict(zip(headers, cells)))

    return rows


def parse_task_table() -> dict[str, dict[str, str]]:
    rows = parse_markdown_table(TRACKER, "## Task Table")
    tasks: dict[str, dict[str, str]] = {}
    for row in rows:
        task_id = row.get("Task ID", "").strip("`")
        if task_id:
            tasks[task_id] = row
    return tasks


def parse_phase_index_table() -> dict[str, dict[str, str]]:
    rows = parse_markdown_table(TRACKER, "## Phase Index")
    phases: dict[str, dict[str, str]] = {}
    for row in rows:
        number = row.get("Phase", "").strip()
        if number:
            phases[number] = row
    return phases


def parse_streams_table() -> dict[str, dict[str, str]]:
    rows = parse_markdown_table(TRACKER, "## Streams (Non-Sequential)")
    streams: dict[str, dict[str, str]] = {}
    for row in rows:
        stream_id = row.get("Stream ID", "").strip()
        if stream_id:
            streams[stream_id] = row
    return streams


def parse_readme_phase_index() -> dict[str, dict[str, str]]:
    content = PHASES_README.read_text(encoding="utf-8").splitlines()
    in_section = False
    phases: dict[str, dict[str, str]] = {}
    bullet_pattern = re.compile(r"- `(\d{2}-[a-z0-9-]+)` \(([^)]+)\)")

    for raw in content:
        line = raw.strip()
        if line == "## Phase Folders":
            in_section = True
            continue

        if in_section and line.startswith("## ") and line != "## Phase Folders":
            break

        if not in_section:
            continue

        match = bullet_pattern.fullmatch(line)
        if not match:
            continue

        phase_id = match.group(1)
        status = match.group(2).strip()
        number = phase_id.split("-", 1)[0]
        phases[number] = {"id": phase_id, "status": status}

    return phases


def parse_immediate_next_workstream() -> dict[str, str]:
    content = TRACKER.read_text(encoding="utf-8").splitlines()
    in_section = False
    fields: dict[str, str] = {}

    for raw in content:
        line = raw.strip()
        if line == "## Immediate Next Workstream":
            in_section = True
            continue

        if in_section and line.startswith("## ") and line != "## Immediate Next Workstream":
            break

        if not in_section or not line.startswith("- "):
            continue

        key, sep, value = line[2:].partition(":")
        if not sep:
            continue
        clean = value.strip()
        if clean.startswith("`") and clean.endswith("`"):
            clean = clean.strip("`")
        fields[key.strip()] = clean

    return fields


def validate_agents() -> list[str]:
    content = AGENTS.read_text(encoding="utf-8")
    errors: list[str] = []
    for doc in REQUIRED_DOC_REFS:
        if doc not in content:
            errors.append(f"AGENTS.md is missing canonical reference to {doc}")
    return errors


def validate_phase_structure() -> list[str]:
    errors: list[str] = []
    if not PHASES_DIR.exists():
        return ["docs/phases directory is missing"]

    phase_dirs = sorted(
        p.name
        for p in PHASES_DIR.iterdir()
        if p.is_dir() and re.fullmatch(r"\d{2}-[a-z0-9-]+", p.name)
    )
    if not phase_dirs:
        return ["No numbered phase directories found under docs/phases"]

    numbers = [int(name.split("-", 1)[0]) for name in phase_dirs]
    expected = list(range(numbers[0], numbers[-1] + 1))
    if numbers != expected:
        errors.append(
            f"Phase numbering is not consecutive: found {numbers}, expected consecutive {expected}"
        )

    for name in phase_dirs:
        phase_path = PHASES_DIR / name
        missing = [fname for fname in REQUIRED_PHASE_FILES if not (phase_path / fname).exists()]
        if missing:
            errors.append(f"Phase {name} is missing required files: {', '.join(sorted(missing))}")
            continue

        # Enforce planning-ready phase docs so orchestrator can gate execution.
        for filename, heading in REQUIRED_PHASE_HEADINGS.items():
            content = (phase_path / filename).read_text(encoding="utf-8")
            if heading not in content:
                errors.append(f"Phase {name} {filename} is missing required heading: {heading}")

    return errors


def validate_phase_registry_consistency(state: dict[str, object]) -> list[str]:
    errors: list[str] = []

    phase_dirs = sorted(
        p.name
        for p in PHASES_DIR.iterdir()
        if p.is_dir() and re.fullmatch(r"\d{2}-[a-z0-9-]+", p.name)
    )
    dir_map = {name.split("-", 1)[0]: name for name in phase_dirs}
    readme_map = parse_readme_phase_index()
    tracker_map = parse_phase_index_table()

    state_sequence = state.get("phase_sequence", [])
    if not isinstance(state_sequence, list) or not state_sequence:
        return ["state/repo-state.json phase_sequence must be a non-empty list"]

    state_map: dict[str, dict[str, str]] = {}
    for phase in state_sequence:
        if not isinstance(phase, dict):
            errors.append("state phase_sequence contains a non-object entry")
            continue
        number = str(phase.get("number", "")).zfill(2)
        state_map[number] = {
            "id": str(phase.get("id", "")),
            "status": str(phase.get("status", "")),
            "path": str(phase.get("path", "")),
        }

    numbers = sorted(dir_map.keys())
    if numbers != sorted(readme_map.keys()):
        errors.append(
            "docs/phases/README.md phase list must match numbered phase directories exactly"
        )
    if numbers != sorted(tracker_map.keys()):
        errors.append("docs/task-tracker.txt Phase Index must match phase directories exactly")
    if numbers != sorted(state_map.keys()):
        errors.append("state/repo-state.json phase_sequence must match phase directories exactly")

    for number in numbers:
        phase_id = dir_map[number]
        expected_path = f"docs/phases/{phase_id}"

        readme_phase = readme_map.get(number)
        if readme_phase is None:
            errors.append(f"docs/phases/README.md is missing phase {number}")
        else:
            if readme_phase["id"] != phase_id:
                errors.append(
                    f"docs/phases/README.md mismatch for phase {number}: id={readme_phase['id']!r}, expected={phase_id!r}"
                )

        tracker_phase = tracker_map.get(number)
        if tracker_phase is None:
            errors.append(f"docs/task-tracker.txt Phase Index is missing phase {number}")
        else:
            tracker_folder = tracker_phase.get("Folder", "")
            if tracker_folder != expected_path:
                errors.append(
                    f"Tracker phase folder mismatch for phase {number}: tracker={tracker_folder!r}, expected={expected_path!r}"
                )

        state_phase = state_map.get(number)
        if state_phase is None:
            errors.append(f"state/repo-state.json phase_sequence is missing phase {number}")
            continue

        if state_phase["id"] != phase_id:
            errors.append(
                f"state phase_sequence id mismatch for phase {number}: state={state_phase['id']!r}, expected={phase_id!r}"
            )
        if state_phase["path"] != expected_path:
            errors.append(
                f"state phase_sequence path mismatch for phase {number}: state={state_phase['path']!r}, expected={expected_path!r}"
            )

        readme_status = (readme_map.get(number) or {}).get("status", "")
        tracker_status = (tracker_map.get(number) or {}).get("Status", "")
        state_status = state_phase["status"]
        if readme_status and readme_status != state_status:
            errors.append(
                f"status mismatch for phase {number} between README and state (state is canonical): readme={readme_status!r}, state={state_status!r}"
            )
        if tracker_status and tracker_status != state_status:
            errors.append(
                f"status mismatch for phase {number} between tracker and state (state is canonical): tracker={tracker_status!r}, state={state_status!r}"
            )

    active_phase = state.get("active_phase", {})
    if not isinstance(active_phase, dict):
        errors.append("state/repo-state.json active_phase must be an object")
        return errors

    active_number = str(active_phase.get("number", "")).zfill(2)
    active_id = str(active_phase.get("id", ""))
    active_status = str(active_phase.get("status", ""))
    active_path = str(active_phase.get("path", ""))

    if active_number not in state_map:
        errors.append(
            f"active_phase.number {active_number!r} is not present in state phase_sequence"
        )
    else:
        phase_entry = state_map[active_number]
        if active_id != phase_entry["id"]:
            errors.append(
                f"active_phase.id mismatch: active={active_id!r}, phase_sequence={phase_entry['id']!r}"
            )
        if active_status != phase_entry["status"]:
            errors.append(
                f"active_phase.status mismatch: active={active_status!r}, phase_sequence={phase_entry['status']!r}"
            )
        if active_path != phase_entry["path"]:
            errors.append(
                f"active_phase.path mismatch: active={active_path!r}, phase_sequence={phase_entry['path']!r}"
            )

    in_progress_numbers = [n for n, p in state_map.items() if p["status"] == "in_progress"]
    if len(in_progress_numbers) > 1:
        errors.append(
            f"state phase_sequence must have at most one in_progress phase, found {in_progress_numbers}"
        )
    if len(in_progress_numbers) == 1 and in_progress_numbers[0] != active_number:
        errors.append(
            f"active_phase.number must match the only in_progress phase: active={active_number!r}, in_progress={in_progress_numbers[0]!r}"
        )
    if active_status == "in_progress" and (
        len(in_progress_numbers) != 1 or in_progress_numbers[0] != active_number
    ):
        errors.append("active_phase must be the only in_progress entry in state phase_sequence")

    return errors


def validate_phase_ownership_and_kickoff(state: dict[str, object]) -> list[str]:
    errors: list[str] = []
    tasks = state.get("tasks", [])
    if not isinstance(tasks, list):
        return ["state/repo-state.json tasks must be a list"]

    active_phase = state.get("active_phase", {})
    if not isinstance(active_phase, dict):
        return ["state/repo-state.json active_phase must be an object"]
    active_phase_number = str(active_phase.get("number", "")).zfill(2)

    tasks_by_phase: dict[str, dict[str, object]] = {}
    for task in tasks:
        if not isinstance(task, dict):
            continue
        phase = str(task.get("phase", "")).zfill(2)
        status = str(task.get("status", ""))
        owner = str(task.get("owner", ""))
        if status in {"planned", "in_progress"} and owner != "architecture-orchestrator":
            errors.append(
                f"planned/in_progress task {task.get('id', '')!r} must be owned by architecture-orchestrator; found {owner!r}"
            )
        if phase and phase not in tasks_by_phase:
            tasks_by_phase[phase] = task

    active_task = tasks_by_phase.get(active_phase_number)
    if active_task is None:
        errors.append(f"no state task found for active phase {active_phase_number}")
        return errors
    if str(active_task.get("owner", "")) != "architecture-orchestrator":
        errors.append("active phase task owner must be architecture-orchestrator")

    current_branch = str(state.get("current_branch", ""))
    task_branch = str(active_task.get("branch", ""))
    if current_branch != task_branch:
        errors.append(
            f"state current_branch must match active phase task branch: current={current_branch!r}, task={task_branch!r}"
        )

    current_objective = state.get("current_objective", {})
    if not isinstance(current_objective, dict):
        errors.append("state/repo-state.json current_objective must be an object")
        return errors

    startup_gate = current_objective.get("startup_gate", {})
    if not isinstance(startup_gate, dict):
        errors.append("state current_objective.startup_gate must be an object")
    else:
        if startup_gate.get("requirements_validated") is not True:
            errors.append("state startup_gate.requirements_validated must be true")
        if startup_gate.get("design_validated") is not True:
            errors.append("state startup_gate.design_validated must be true")
        kickoff = str(startup_gate.get("kickoff_date", ""))
        if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", kickoff):
            errors.append(
                f"state startup_gate.kickoff_date must use YYYY-MM-DD; found {kickoff!r}"
            )

    resume_checkpoint = current_objective.get("resume_checkpoint", {})
    if not isinstance(resume_checkpoint, dict):
        errors.append("state current_objective.resume_checkpoint must be an object")
    else:
        checkpoint_branch = str(resume_checkpoint.get("branch", ""))
        checkpoint_worktree = str(resume_checkpoint.get("worktree", ""))
        if checkpoint_branch != task_branch:
            errors.append(
                f"resume_checkpoint.branch must match active task branch: checkpoint={checkpoint_branch!r}, task={task_branch!r}"
            )
        if checkpoint_worktree != str(active_task.get("worktree", "")):
            errors.append(
                "resume_checkpoint.worktree must match active task worktree"
            )
        if checkpoint_branch and not re.fullmatch(r"codex/[a-z0-9._-]+", checkpoint_branch):
            errors.append("resume_checkpoint.branch must match codex/<task>")
        if checkpoint_worktree and not re.fullmatch(
            r"\.worktrees/[a-z0-9._-]+", checkpoint_worktree
        ):
            errors.append(
                "resume_checkpoint.worktree must match .worktrees/<phase>-<task>-<agent>"
            )

    phase_doc = str(current_objective.get("phase_doc", ""))
    if not phase_doc.startswith(str(active_phase.get("path", ""))):
        errors.append("current_objective.phase_doc must be under active_phase.path")
    if phase_doc != str(active_task.get("phase_doc", "")):
        errors.append("current_objective.phase_doc must match active phase task phase_doc")
    execution_mode = str(current_objective.get("execution_mode", ""))
    if execution_mode != "orchestrator_owned_phase_with_worker_handoffs":
        errors.append(
            "current_objective.execution_mode must be 'orchestrator_owned_phase_with_worker_handoffs'"
        )

    orchestration = state.get("orchestration", {})
    if not isinstance(orchestration, dict):
        errors.append("state orchestration must be an object")
        return errors
    if orchestration.get("phase_owner_must_be_orchestrator") is not True:
        errors.append("state orchestration.phase_owner_must_be_orchestrator must be true")

    fallback = orchestration.get("write_constrained_worker_fallback", {})
    if not isinstance(fallback, dict):
        errors.append("state orchestration.write_constrained_worker_fallback must be an object")
    else:
        if fallback.get("enabled") is not True:
            errors.append("write_constrained_worker_fallback.enabled must be true")
        if fallback.get("worker_delivers_summary_input") is not True:
            errors.append("write_constrained_worker_fallback.worker_delivers_summary_input must be true")
        if fallback.get("orchestrator_integrates_and_codes") is not True:
            errors.append("write_constrained_worker_fallback.orchestrator_integrates_and_codes must be true")
        if str(fallback.get("state_label", "")) != "orchestrator_integration_from_worker_handoff":
            errors.append(
                "write_constrained_worker_fallback.state_label must be 'orchestrator_integration_from_worker_handoff'"
            )
        if (
            fallback.get("enabled") is True
            and execution_mode == "orchestrator_owned_phase_with_worker_handoffs"
        ):
            pass
        elif fallback.get("enabled") is True:
            errors.append(
                "current_objective.execution_mode must align with enabled write_constrained_worker_fallback"
            )

    return errors


def validate_stream_registry_consistency(state: dict[str, object]) -> list[str]:
    errors: list[str] = []
    streams = state.get("streams", [])
    if not isinstance(streams, list):
        return ["state/repo-state.json streams must be a list"]

    state_streams: dict[str, dict[str, str]] = {}
    for stream in streams:
        if not isinstance(stream, dict):
            errors.append("state stream entry must be an object")
            continue
        stream_id = str(stream.get("id", ""))
        if not stream_id:
            errors.append("state stream entry is missing id")
            continue
        state_streams[stream_id] = {
            "status": str(stream.get("status", "")),
            "owner": str(stream.get("owner", "")),
            "doc": str(stream.get("doc", "")),
        }

    tracker_streams = parse_streams_table()
    if set(state_streams) != set(tracker_streams):
        errors.append("tracker/state stream ids must match exactly")

    for stream_id, state_stream in state_streams.items():
        tracker_stream = tracker_streams.get(stream_id)
        if tracker_stream is None:
            errors.append(f"tracker is missing stream row for {stream_id}")
            continue
        if tracker_stream.get("Status", "") != state_stream["status"]:
            errors.append(
                f"tracker/state mismatch for {stream_id}.status: tracker={tracker_stream.get('Status', '')!r}, state={state_stream['status']!r}"
            )
        if tracker_stream.get("Owner", "") != state_stream["owner"]:
            errors.append(
                f"tracker/state mismatch for {stream_id}.owner: tracker={tracker_stream.get('Owner', '')!r}, state={state_stream['owner']!r}"
            )
        if tracker_stream.get("Doc", "") != state_stream["doc"]:
            errors.append(
                f"tracker/state mismatch for {stream_id}.doc: tracker={tracker_stream.get('Doc', '')!r}, state={state_stream['doc']!r}"
            )

    immediate_stream = parse_immediate_next_workstream()
    immediate_id = immediate_stream.get("id", "").strip("`")
    immediate_status = immediate_stream.get("status", "")
    active_phase = state.get("active_phase")
    if isinstance(active_phase, dict):
        active_status = str(active_phase.get("status", ""))
    else:
        active_status = ""

    if immediate_id:
        if immediate_id not in state_streams:
            errors.append(
                f"Immediate Next Workstream id {immediate_id!r} is not present in state streams"
            )
        if immediate_status and immediate_id in state_streams:
            state_status = state_streams[immediate_id]["status"]
            if state_status not in immediate_status:
                errors.append(
                    f"Immediate Next Workstream status must include stream status {state_status!r}; found {immediate_status!r}"
                )
        if (
            active_status == "in_progress"
            and "in_progress" in immediate_status
            and "parallel" not in immediate_status.lower()
        ):
            errors.append(
                "Immediate Next Workstream status must explicitly mark in-progress streams as parallel while a phase is active"
            )

    return errors


def validate_tracker_state() -> list[str]:
    rows = parse_task_table()
    state = json.loads(STATE.read_text(encoding="utf-8"))
    errors: list[str] = []

    branch = state.get("current_branch", "")
    if not re.fullmatch(r"codex/[a-z0-9._-]+", branch):
        errors.append("state/repo-state.json current_branch must match codex/<task>")

    tasks = state.get("tasks", [])
    if not isinstance(tasks, list) or not tasks:
        errors.append("state/repo-state.json tasks must be a non-empty list")
        return errors

    for task in tasks:
        task_id = task.get("id", "")
        if not task_id:
            errors.append("state task is missing id")
            continue

        tracker = rows.get(task_id)
        if tracker is None:
            errors.append(f"Tracker is missing row for task {task_id}")
            continue

        expected_phase = str(task.get("phase", "")).zfill(2)
        tracker_phase = tracker.get("Phase", "")
        if tracker_phase != expected_phase:
            errors.append(
                f"Tracker/state mismatch for {task_id}.phase: tracker={tracker_phase!r}, state={expected_phase!r}"
            )

        tracker_owner = tracker.get("Owner Agent", "")
        if tracker_owner != task.get("owner", ""):
            errors.append(
                f"Tracker/state mismatch for {task_id}.owner: tracker={tracker_owner!r}, state={task.get('owner', '')!r}"
            )

        tracker_status = tracker.get("Status", "")
        if tracker_status != task.get("status", ""):
            errors.append(
                f"Tracker/state mismatch for {task_id}.status: tracker={tracker_status!r}, state={task.get('status', '')!r}"
            )

    errors.extend(validate_phase_registry_consistency(state))
    errors.extend(validate_phase_ownership_and_kickoff(state))
    errors.extend(validate_stream_registry_consistency(state))

    return errors


def main() -> int:
    errors: list[str] = []
    errors.extend(validate_agents())
    errors.extend(validate_phase_structure())
    errors.extend(validate_tracker_state())

    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1

    print("governance validation passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
