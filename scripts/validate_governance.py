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

REQUIRED_DOCS = [
    "docs/project-vision.md",
    "docs/architecture-plan.md",
    "docs/task-tracker.txt",
    "docs/agent-playbook.md",
    "docs/agent-registry.md",
    "docs/adr/README.md",
    "docs/platform-capability-matrix.md",
    "docs/validation-policy.md",
    "docs/branch-commit-workflow.md",
    "state/repo-state.json",
]


def parse_tracker() -> dict[str, dict[str, str]]:
    rows: dict[str, dict[str, str]] = {}
    current_task: str | None = None

    for raw_line in TRACKER.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if line.startswith("TASK "):
            current_task = line.removeprefix("TASK ").strip()
            rows[current_task] = {}
            continue

        if line == "ENDTASK":
            current_task = None
            continue

        if current_task is None:
            continue

        if ": " not in line:
            continue

        key, value = line.split(": ", 1)
        rows[current_task][key.strip().replace(" ", "_")] = value.strip()

    return rows


def validate_agents() -> list[str]:
    content = AGENTS.read_text(encoding="utf-8")
    errors = []
    for doc in REQUIRED_DOCS:
        if doc not in content:
            errors.append(f"AGENTS.md is missing canonical reference to {doc}")
    return errors


def validate_tracker_state() -> list[str]:
    tracker_rows = parse_tracker()
    state = json.loads(STATE.read_text(encoding="utf-8"))
    errors = []

    branch = state.get("current_branch")
    if not branch or not re.fullmatch(r"codex/[a-z0-9._-]+", branch):
        errors.append("state/repo-state.json current_branch must match codex/<task>")

    for task in state.get("tasks", []):
        task_id = task["id"]
        tracker = tracker_rows.get(task_id)
        if tracker is None:
            errors.append(f"Tracker is missing task row for {task_id}")
            continue

        comparisons = {
            "title": task.get("title", ""),
            "phase": task["phase"],
            "owner": task["owner"],
            "status": task["status"],
            "next_action": task["next_action"],
        }
        for field, expected in comparisons.items():
            actual = tracker.get(field, "")
            if actual != expected:
                errors.append(
                    f"Tracker/state mismatch for {task_id}.{field}: tracker={actual!r}, state={expected!r}"
                )
        if not task.get("evidence"):
            errors.append(f"Task {task_id} must include at least one evidence item in state")
        if not tracker.get("evidence"):
            errors.append(f"Task {task_id} must include evidence text in tracker")

    return errors


def main() -> int:
    errors = []
    errors.extend(validate_agents())
    errors.extend(validate_tracker_state())

    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        return 1

    print("governance validation passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
