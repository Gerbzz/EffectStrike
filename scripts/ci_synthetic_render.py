"""Bounded end-to-end headless smoke test using generated, openly reusable pixels."""

from __future__ import annotations

import hashlib
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


def png_pixels(path: Path) -> tuple[tuple[int, int], bytes]:
    """Decode the RGB/RGBA 8-bit PNGs from the CLI, including adaptive row filters."""
    data = path.read_bytes()
    assert data.startswith(b"\x89PNG\r\n\x1a\n"), f"not PNG: {path}"
    offset, compressed, header = 8, bytearray(), None
    while offset < len(data):
        length = struct.unpack_from(">I", data, offset)[0]
        kind = data[offset + 4 : offset + 8]
        payload = data[offset + 8 : offset + 8 + length]
        if kind == b"IHDR":
            header = struct.unpack(">IIBBBBB", payload)
        elif kind == b"IDAT":
            compressed.extend(payload)
        elif kind == b"IEND":
            break
        offset += length + 12
    assert header is not None, f"PNG lacks IHDR: {path}"
    width, height, depth, color, compression, filtering, interlace = header
    assert (depth, compression, filtering, interlace) == (8, 0, 0, 0), header
    assert color in (2, 6), f"expected RGB or RGBA output, got color type {color}"
    channels = 3 if color == 2 else 4
    stride = width * channels
    rows = zlib.decompress(compressed)
    assert len(rows) == height * (stride + 1), f"unexpected PNG row size: {path}"
    output = bytearray()
    previous = bytearray(stride)
    for y in range(height):
        start = y * (stride + 1)
        filter_type = rows[start]
        row = bytearray(rows[start + 1 : start + 1 + stride])
        assert filter_type in range(5), f"unsupported PNG filter {filter_type}"
        for x in range(stride):
            left = row[x - channels] if x >= channels else 0
            above = previous[x]
            upper_left = previous[x - channels] if x >= channels else 0
            if filter_type == 1:
                predictor = left
            elif filter_type == 2:
                predictor = above
            elif filter_type == 3:
                predictor = (left + above) // 2
            elif filter_type == 4:
                p = left + above - upper_left
                distances = (abs(p - left), abs(p - above), abs(p - upper_left))
                predictor = (left, above, upper_left)[distances.index(min(distances))]
            else:
                predictor = 0
            row[x] = (row[x] + predictor) & 255
        output.extend(row)
        previous = row
    return (width, height), bytes(output)


def changed_pixels(before: Path, after: Path) -> int:
    dimensions, a = png_pixels(before)
    other_dimensions, b = png_pixels(after)
    assert dimensions == other_dimensions, (dimensions, other_dimensions)
    assert len(a) == len(b), "rendered PNGs use different channel counts"
    channels = len(a) // (dimensions[0] * dimensions[1])
    return sum(a[i : i + 3] != b[i : i + 3] for i in range(0, len(a), channels))


def cli(binary: Path, *args: str) -> object:
    command = [str(binary), *args, "--json"]
    result = subprocess.run(command, text=True, encoding="utf-8", capture_output=True, timeout=90, check=False)
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

    created = cli(
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
    imported = next(step["result"] for step in created["result"] if step["command"] == "file.import")
    assert imported["items"] and not imported["errors"], imported
    assert project.is_file() and project.stat().st_size > 100, "project was not saved"

    reopened = cli(binary, "info", "--project", str(project))
    comp = reopened["activeComp"]
    assert comp["name"] == "Synthetic Smoke", comp
    assert comp["width"] == 160 and comp["height"] == 90, comp
    assert len(comp["layers"]) >= 2, comp
    assert any("synthetic" in item["name"].lower() for item in reopened["project"]["items"]), reopened
    footage_layer = next(layer for layer in comp["layers"] if "synthetic" in layer["name"].lower())
    for time, position in ((0, [32, 45]), (0.5, [128, 45])):
        cli(
            binary,
            "exec",
            "prop.addKey",
            "--params",
            json.dumps({"layer": footage_layer["id"], "path": "transform/position", "time": time, "value": position}),
            "--project",
            str(project),
            "--save",
        )
    animated = cli(binary, "get", comp["name"], str(footage_layer["id"]), "transform/position", "--time", "0.5", "--project", str(project))
    assert animated["animated"] and len(animated["keys"]) == 2, animated

    rendered = cli(binary, "render-frame", "--project", str(project), "--time", "0.25", "--out", str(frame))
    data = frame.read_bytes()
    assert data.startswith(b"\x89PNG\r\n\x1a\n"), "renderer did not produce PNG"
    width, height = struct.unpack(">II", data[16:24])
    assert (width, height) == (160, 90), (width, height)
    assert len(data) > 200, "rendered frame is suspiciously small"
    start = output / "start.png"
    finish = output / "finish.png"
    cli(binary, "render-frame", "--project", str(project), "--time", "0", "--out", str(start))
    cli(binary, "render-frame", "--project", str(project), "--time", "0.5", "--out", str(finish))
    assert start.read_bytes() != finish.read_bytes(), "keyframed footage rendered the same at both times"

    queued = cli(binary, "render", "--project", str(project), "--out", str(output / "sequence_[#####].png"), "--format", "png", "--start", "0", "--end", "0.1")
    sequence = sorted(output.glob("sequence_*.png"))
    assert queued["rendered"] and sequence, queued
    assert all(path.read_bytes().startswith(b"\x89PNG\r\n\x1a\n") for path in sequence)

    # Real CPU compositor output, not just an effect record in the saved project.
    effect_before = output / "effect-before.png"
    effect_after = output / "effect-after.png"
    expression_after = output / "expression-after.png"
    cli(binary, "render-frame", "--project", str(project), "--time", "0", "--out", str(effect_before))
    applied = cli(binary, "exec", "effect.apply", "--params", json.dumps({"layer": footage_layer["id"], "effect": "ec.blur.gaussian"}), "--project", str(project), "--save")
    assert applied["result"]["paths"] == ["effects/#1"], applied
    cli(binary, "exec", "prop.set", "--params", json.dumps({"layer": footage_layer["id"], "path": "effects/#1/blurriness", "value": 16}), "--project", str(project), "--save")
    blur = cli(binary, "get", comp["name"], str(footage_layer["id"]), "effects/#1/blurriness", "--project", str(project))
    assert blur["value"] == 16, blur
    cli(binary, "render-frame", "--project", str(project), "--time", "0", "--out", str(effect_after))
    effect_changed = changed_pixels(effect_before, effect_after)
    assert effect_changed > 50, f"Gaussian Blur changed only {effect_changed} RGB pixels"

    expression = "value + [0, 20]"
    set_expr = cli(binary, "exec", "prop.setExpression", "--params", json.dumps({"layer": footage_layer["id"], "path": "transform/position", "expression": expression}), "--project", str(project), "--save")
    assert set_expr["result"] == expression, set_expr
    evaluated = cli(binary, "get", comp["name"], str(footage_layer["id"]), "transform/position", "--time", "0", "--project", str(project))
    assert evaluated["expression"] == expression and evaluated.get("expressionError") is None, evaluated
    # Layer transforms store a third (Z) component even when the UI shows a 2D position.
    assert evaluated["value"] == [32, 45, 0] and evaluated["evaluated"] == [32, 65, 0], evaluated
    cli(binary, "render-frame", "--project", str(project), "--time", "0", "--out", str(expression_after))
    expression_changed = changed_pixels(effect_after, expression_after)
    assert expression_changed > 50, f"Position expression changed only {expression_changed} RGB pixels"
    print(json.dumps({"project_bytes": project.stat().st_size, "render_bytes": len(data), "render_sha256": hashlib.sha256(data).hexdigest(), "dimensions": [width, height], "comp": comp["name"], "layers": len(comp["layers"]), "keyframes": len(animated["keys"]), "queued_frames": len(sequence), "effect_changed_pixels": effect_changed, "expression_changed_pixels": expression_changed, "renderer": rendered}))


if __name__ == "__main__":
    main()
