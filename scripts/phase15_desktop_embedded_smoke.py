#!/usr/bin/env python3
"""Phase 15 desktop build smoke: embedded startup + sidecar import + asset path."""

from __future__ import annotations

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


def main() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    pattern_asset = repo_root / "examples" / "assets" / "pattern.png"
    if not pattern_asset.is_file():
        raise SystemExit(f"missing required test asset: {pattern_asset}")

    with tempfile.TemporaryDirectory(prefix="pycro-phase15-smoke-") as temp_dir:
        project_root = Path(temp_dir)
        (project_root / "assets").mkdir(parents=True, exist_ok=True)
        shutil.copy2(pattern_asset, project_root / "assets" / "pattern.png")

        (project_root / "support.py").write_text(
            "TEXT='embedded smoke sidecar import ok'\n", encoding="utf-8"
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

        run(
            [
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
                "desktop",
            ],
            cwd=repo_root,
        )

        binary_name = "game.exe" if os.name == "nt" else "game"
        binary = project_root / "dist" / "desktop" / binary_name
        if not binary.is_file():
            fallback = project_root / "dist" / "desktop" / "game"
            fallback_exe = project_root / "dist" / "desktop" / "game.exe"
            binary = fallback if fallback.is_file() else fallback_exe
        if not binary.is_file():
            raise SystemExit("desktop smoke failed: built artifact `dist/desktop/game` not found")

        shutil.rmtree(project_root / "assets")
        (project_root / "main.py").unlink()
        (project_root / "support.py").unlink()

        env = dict(os.environ)
        env["PYCRO_FRAMES"] = "2"
        env["PYCRO_FRAME_DT"] = "0.016"
        run([str(binary)], cwd=project_root, env=env)

        print("phase15 desktop embedded smoke: PASS")


if __name__ == "__main__":
    main()
