"""Bounded end-to-end headless smoke test using generated, openly reusable pixels."""

from __future__ import annotations

import json
import struct
import subprocess
import sys
import zlib
from pathlib import Path


def chunk(kind: bytes, payload: bytes) -> bytes:
    body = kind + payload
    return struct.pack(">I", len(payload)) + body + struct.pack(">I", zlib.crc32(body))


def write_synthetic_png(path: Path) -> None:
    width = height = 32
    rows = []
    for y in range(height):
        pixels = bytearray()
        for x in range(width):
            pixels.extend((255, 192, 32, 255) if (x // 8 + y // 8) % 2 else (32, 224, 255, 255))
        rows.append(b"\x00" + pixels)
    path.write_bytes(
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(b"".join(rows)))
        + chunk(b"IEND", b"")
    )


def cli(binary: Path, *args: str) -> object:
    command = [str(binary), *args, "--json"]
    result = subprocess.run(command, text=True, capture_output=True, timeout=90, check=False)
    if result.returncode:
        raise AssertionError(f"CLI failed ({result.returncode}): {command}\n{result.stdout}\n{result.stderr}")
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise AssertionError(f"CLI returned invalid JSON: {result.stdout}\n{result.stderr}") from error


def main() -> None:
    binary = Path(sys.argv[1]).resolve()
    output = Path(sys.argv[2]).resolve()
    output.mkdir(parents=True, exist_ok=True)
    synthetic = output / "synthetic.png"
    project = output / "synthetic.ecproj"
    frame = output / "render.png"
    write_synthetic_png(synthetic)

    cli(
        binary,
        "run",
        "comp.new",
        json.dumps({"name": "Synthetic Smoke", "width": 160, "height": 90, "duration": 1, "frameRate": 30}),
        "layer.newSolid",
        json.dumps({"name": "Blue background", "color": "#254a9c", "width": 160, "height": 90}),
        "file.import",
        json.dumps({"paths": [str(synthetic)], "addToComp": True}),
        "--empty",
        "--save-as",
        str(project),
    )
    assert project.is_file() and project.stat().st_size > 100, "project was not saved"

    reopened = cli(binary, "info", "--project", str(project))
    comp = reopened["activeComp"]
    assert comp["name"] == "Synthetic Smoke", comp
    assert comp["width"] == 160 and comp["height"] == 90, comp
    assert len(comp["layers"]) >= 2, comp
    assert any("synthetic" in item["name"].lower() for item in reopened["project"]["items"]), reopened

    rendered = cli(binary, "render-frame", "--project", str(project), "--time", "0.25", "--out", str(frame))
    data = frame.read_bytes()
    assert data.startswith(b"\x89PNG\r\n\x1a\n"), "renderer did not produce PNG"
    width, height = struct.unpack(">II", data[16:24])
    assert (width, height) == (160, 90), (width, height)
    assert len(data) > 200, "rendered frame is suspiciously small"
    print(json.dumps({"project_bytes": project.stat().st_size, "render_bytes": len(data), "dimensions": [width, height], "comp": comp["name"], "layers": len(comp["layers"]), "renderer": rendered}))


if __name__ == "__main__":
    main()
