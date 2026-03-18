#!/usr/bin/env python3
"""Phase 18 android build smoke: apk output + embedded payload markers."""

from __future__ import annotations

import shlex
import shutil
import subprocess
import tempfile
import os
from pathlib import Path


def run(cmd: list[str], *, cwd: Path, env: dict[str, str] | None = None) -> None:
    print(f"$ {' '.join(cmd)}")
    completed = subprocess.run(
        cmd,
        cwd=cwd,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.stdout:
        print(completed.stdout, end="")
    if completed.returncode != 0:
        if completed.stderr:
            print(completed.stderr, end="")
        raise SystemExit(completed.returncode)


def smoke_command(repo_root: Path, project_root: Path) -> list[str]:
    override_value = os.environ.get("PYCRO_PHASE18_ANDROID_SMOKE_CMD")
    if override_value:
        return [
            token.replace("{repo_root}", str(repo_root)).replace(
                "{project_root}", str(project_root)
            )
            for token in shlex.split(override_value)
        ]

    return [
        "cargo",
        "run",
        "--bin",
        "pycro",
        "--",
        "project",
        "build",
        "--project",
        str(project_root),
        "--target",
        "android",
    ]


def latest_generated_payload(repo_root: Path) -> Path:
    candidates = list(
        (repo_root / "target").glob("**/build/pycro_cli-*/out/embedded_project_payload.rs")
    )
    if not candidates:
        raise SystemExit(
            "android smoke failed: embedded payload metadata not found under target/**/build"
        )
    return max(candidates, key=lambda path: path.stat().st_mtime)


def main() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    pattern_asset = repo_root / "examples" / "assets" / "pattern.png"
    if not pattern_asset.is_file():
        raise SystemExit(f"missing required test asset: {pattern_asset}")

    with tempfile.TemporaryDirectory(prefix="pycro-phase18-smoke-") as temp_dir:
        project_root = Path(temp_dir)
        (project_root / "assets").mkdir(parents=True, exist_ok=True)
        shutil.copy2(pattern_asset, project_root / "assets" / "pattern.png")
        (project_root / "support.py").write_text(
            "TEXT='embedded android smoke sidecar import ok'\n", encoding="utf-8"
        )
        (project_root / "main.py").write_text(
            "import pycro\n"
            "from support import TEXT\n\n"
            "def update(dt: float) -> None:\n"
            "    pycro.clear_background((0.03, 0.03, 0.04, 1.0))\n"
            "    pycro.draw_text(TEXT, (16.0, 48.0), 24.0, (0.9, 0.94, 0.98, 1.0))\n",
            encoding="utf-8",
        )

        run(smoke_command(repo_root, project_root), cwd=repo_root)

        apk_dir = project_root / "dist" / "android" / "apk"
        apks = sorted(path for path in apk_dir.glob("*.apk") if path.is_file())
        if not apks:
            raise SystemExit(
                f"android smoke failed: no apk artifacts in {apk_dir.relative_to(project_root)}"
            )

        generated_payload = latest_generated_payload(repo_root)
        generated_payload_text = generated_payload.read_text(encoding="utf-8")
        for marker in ("main.py",):
            if marker not in generated_payload_text:
                raise SystemExit(
                    f"android smoke failed: embedded payload metadata missing marker `{marker}` ({generated_payload})"
                )

        if (
            "support.py" not in generated_payload_text
            and "assets/pattern.png" not in generated_payload_text
        ):
            raise SystemExit(
                "android smoke failed: embedded payload metadata missing sidecar/asset marker (`support.py` or `assets/pattern.png`)"
            )

        print("phase18 android embedded smoke: PASS")


if __name__ == "__main__":
    main()
