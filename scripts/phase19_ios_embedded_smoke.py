#!/usr/bin/env python3
"""Phase 19 iOS build smoke: Xcode output contract + embedded payload markers."""

from __future__ import annotations

import os
import platform
import shlex
import shutil
import subprocess
import tempfile
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
    override_value = os.environ.get("PYCRO_PHASE19_IOS_SMOKE_CMD")
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
        "ios",
    ]


def latest_generated_payload(repo_root: Path) -> Path:
    target_root = repo_root / "target"
    candidates: list[Path] = []
    for package_glob in ("pycro-*", "pycro_cli-*"):
        candidates.extend(
            target_root.glob(f"**/build/{package_glob}/out/embedded_project_payload.rs")
        )
    if not candidates:
        raise SystemExit(
            "ios smoke failed: embedded payload metadata not found under target/**/build"
        )
    return max(candidates, key=lambda path: path.stat().st_mtime)


def validate_xcode_output(project_root: Path) -> None:
    xcode_root = project_root / "dist" / "ios" / "xcode"
    if not xcode_root.is_dir():
        raise SystemExit(
            f"ios smoke failed: output contract directory missing: {xcode_root.relative_to(project_root)}"
        )

    xcode_projects = sorted(
        path
        for path in xcode_root.glob("**/*.xcodeproj")
        if path.is_dir() and (path / "project.pbxproj").is_file()
    )
    if not xcode_projects:
        raise SystemExit(
            "ios smoke failed: no Xcode project bundle found under dist/ios/xcode"
        )


def main() -> None:
    if platform.system() != "Darwin":
        raise SystemExit("ios smoke failed: iOS builds require macOS/Xcode")

    repo_root = Path(__file__).resolve().parents[1]
    pattern_asset = repo_root / "examples" / "assets" / "pattern.png"
    if not pattern_asset.is_file():
        raise SystemExit(f"missing required test asset: {pattern_asset}")

    with tempfile.TemporaryDirectory(prefix="pycro-phase19-smoke-") as temp_dir:
        project_root = Path(temp_dir)
        (project_root / "assets").mkdir(parents=True, exist_ok=True)
        shutil.copy2(pattern_asset, project_root / "assets" / "pattern.png")
        (project_root / "support.py").write_text(
            "TEXT='embedded ios smoke sidecar import ok'\n", encoding="utf-8"
        )
        (project_root / "main.py").write_text(
            "import pycro\n"
            "from support import TEXT\n\n"
            "tex = None\n\n"
            "def update(dt: float) -> None:\n"
            "    global tex\n"
            "    if tex is None:\n"
            "        tex = pycro.load_texture('assets/pattern.png')\n"
            "    pycro.clear_background((0.03, 0.03, 0.04, 1.0))\n"
            "    pycro.draw_texture(tex, (16.0, 72.0), (96.0, 96.0))\n"
            "    pycro.draw_text(TEXT, (16.0, 48.0), 24.0, (0.9, 0.94, 0.98, 1.0))\n",
            encoding="utf-8",
        )

        run(smoke_command(repo_root, project_root), cwd=repo_root)

        validate_xcode_output(project_root)

        generated_payload = latest_generated_payload(repo_root)
        generated_payload_text = generated_payload.read_text(encoding="utf-8")
        for marker in ("main.py",):
            if marker not in generated_payload_text:
                raise SystemExit(
                    f"ios smoke failed: embedded payload metadata missing marker `{marker}` ({generated_payload})"
                )

        print("phase19 ios embedded smoke: PASS")


if __name__ == "__main__":
    main()
