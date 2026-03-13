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


def parse_table_row(line: str) -> list[str]:
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def parse_task_table() -> dict[str, dict[str, str]]:
    content = TRACKER.read_text(encoding="utf-8").splitlines()
    in_task_section = False
    headers: list[str] = []
    rows: dict[str, dict[str, str]] = {}

    for raw in content:
        line = raw.strip()
        if line == "## Task Table":
            in_task_section = True
            continue

        if in_task_section and line.startswith("## ") and line != "## Task Table":
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

        row = dict(zip(headers, cells))
        task_id = row.get("Task ID", "").strip("`")
        if task_id:
            rows[task_id] = row

    return rows


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
