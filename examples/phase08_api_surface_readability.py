"""
Phase 08 non-interactive scenario:
Quickly inspect python/pycro/__init__.pyi readability from local workspace.

Run:
  python3 examples/phase08_api_surface_readability.py
"""

from __future__ import annotations

from pathlib import Path


def main() -> None:
    repo_root = Path(__file__).resolve().parents[1]
    stub_path = repo_root / "python" / "pycro" / "__init__.pyi"
    if not stub_path.exists():
        raise SystemExit(f"missing stub file: {stub_path}")

    lines = stub_path.read_text(encoding="utf-8").splitlines()
    public_symbols = [
        line.split("(")[0].replace("def ", "").strip()
        for line in lines
        if line.startswith("def ")
    ]

    print("phase08_api_surface_readability")
    print(f"stub_path={stub_path}")
    print(f"total_lines={len(lines)}")
    print(f"public_symbols={len(public_symbols)}")
    print("symbols_preview=" + ", ".join(public_symbols[:12]))


if __name__ == "__main__":
    main()
