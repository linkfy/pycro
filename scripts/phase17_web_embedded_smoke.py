#!/usr/bin/env python3
"""Phase 17 web build smoke: web output layout + embedded payload markers."""

from __future__ import annotations

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


def main() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    pattern_asset = repo_root / "examples" / "assets" / "pattern.png"
    if not pattern_asset.is_file():
        raise SystemExit(f"missing required test asset: {pattern_asset}")

    with tempfile.TemporaryDirectory(prefix="pycro-phase17-smoke-") as temp_dir:
        project_root = Path(temp_dir)
        (project_root / "assets").mkdir(parents=True, exist_ok=True)
        shutil.copy2(pattern_asset, project_root / "assets" / "pattern.png")

        (project_root / "support.py").write_text(
            "TEXT='embedded web smoke sidecar import ok'\n", encoding="utf-8"
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
                "web",
            ],
            cwd=repo_root,
        )

        web_root = project_root / "dist" / "web"
        wasm = web_root / "pycro.wasm"
        gl_js = web_root / "gl.js"
        index_html = web_root / "index.html"

        missing = [path for path in (wasm, gl_js, index_html) if not path.is_file()]
        if missing:
            raise SystemExit(
                "web smoke failed: missing artifacts: "
                + ", ".join(str(path.relative_to(project_root)) for path in missing)
            )

        html = index_html.read_text(encoding="utf-8")
        for token in ("gl.js", "pycro.wasm", "load(\"pycro.wasm\")"):
            if token not in html:
                raise SystemExit(f"web smoke failed: index.html missing token `{token}`")

        # Ensure output remains valid even if project Python/assets are removed post-build.
        shutil.rmtree(project_root / "assets")
        (project_root / "main.py").unlink()
        (project_root / "support.py").unlink()

        generated_payload_candidates = list(
            (
                repo_root
                / "target"
                / "wasm32-unknown-unknown"
                / "release"
                / "build"
            ).glob("pycro_cli-*/out/embedded_project_payload.rs")
        )
        if not generated_payload_candidates:
            raise SystemExit(
                "web smoke failed: embedded payload metadata not found under target/wasm32-unknown-unknown/release/build"
            )
        generated_payload = max(
            generated_payload_candidates, key=lambda path: path.stat().st_mtime
        )
        generated_payload_text = generated_payload.read_text(encoding="utf-8")
        for marker in ("main.py", "support.py", "assets/pattern.png"):
            if marker not in generated_payload_text:
                raise SystemExit(
                    f"web smoke failed: embedded payload metadata missing marker `{marker}` ({generated_payload})"
                )

        print("phase17 web embedded smoke: PASS")


if __name__ == "__main__":
    main()
