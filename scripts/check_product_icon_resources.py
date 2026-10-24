#!/usr/bin/env python3
"""Verify export hashes and real Windows PE icon resources. Never executes an EXE."""
import argparse
import ctypes
from ctypes import wintypes
import hashlib
import json
import os
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / "assets/branding/icons"


def check_manifest(require_icns=False):
    manifest = json.loads((DIRECTORY / "provenance.json").read_text())
    for item in manifest["exports"]:
        path = ROOT / item["path"]
        data = path.read_bytes()
        if len(data) != item["bytes"] or hashlib.sha256(data).hexdigest() != item["sha256"]:
            raise ValueError(f"{item['path']}: export hash/size mismatch")
    for key in ["source_png", "source_svg"]:
        if hashlib.sha256((ROOT / manifest[key]).read_bytes()).hexdigest() != manifest[key + "_sha256"]:
            raise ValueError(f"{key}: master hash mismatch")
    if require_icns:
        expected = "assets/branding/icons/EffectStrike.icns"
        if expected not in {item["path"] for item in manifest["exports"]}:
            raise ValueError("Final package gate: native-generated ICNS has not been reviewed and committed")
        if not (ROOT / (expected + ".attribution")).is_file():
            raise ValueError("Final package gate: ICNS attribution missing")
        if f"`{expected}`" not in (ROOT / "ATTRIBUTION.md").read_text():
            raise ValueError("Final package gate: ICNS asset index entry missing")
    print(f"Verified {len(manifest['exports'])} attributed export hashes and both original masters")


def ico_entries(data):
    if len(data) < 6 or data[:4] != b"\0\0\1\0":
        raise ValueError("invalid ICO header")
    count = struct.unpack_from("<H", data, 4)[0]
    if count != 9 or len(data) < 6 + count * 16:
        raise ValueError("ICO must contain all nine product sizes")
    result = []
    for index in range(count):
        w, h, colors, reserved, planes, bits, length, offset = struct.unpack_from("<BBBBHHII", data, 6 + 16 * index)
        width, height = w or 256, h or 256
        if offset < 6 + count * 16 or offset + length > len(data):
            raise ValueError("ICO payload bounds invalid")
        png = data[offset:offset + length]
        if png[:8] != b"\x89PNG\r\n\x1a\n" or len(png) < 24 or struct.unpack_from(">II", png, 16) != (width, height):
            raise ValueError("ICO dimensions differ from PNG payload")
        result.append(((w, h, colors, reserved, planes, bits, length), png))
    if [(entry[0][0] or 256) for entry in result] != [16, 20, 24, 32, 40, 48, 64, 128, 256]:
        raise ValueError("ICO size list differs")
    return result


def check_windows_executable(executable):
    if os.name != "nt":
        raise RuntimeError("PE resource verification requires native Windows")
    expected = ico_entries((DIRECTORY / "EffectStrike.ico").read_bytes())
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.LoadLibraryExW.argtypes = [wintypes.LPCWSTR, wintypes.HANDLE, wintypes.DWORD]
    kernel.LoadLibraryExW.restype = wintypes.HMODULE
    kernel.FreeLibrary.argtypes = [wintypes.HMODULE]
    kernel.FindResourceW.argtypes = [wintypes.HMODULE, ctypes.c_void_p, ctypes.c_void_p]
    kernel.FindResourceW.restype = wintypes.HRSRC
    kernel.SizeofResource.argtypes = [wintypes.HMODULE, wintypes.HRSRC]
    kernel.SizeofResource.restype = wintypes.DWORD
    kernel.LoadResource.argtypes = [wintypes.HMODULE, wintypes.HRSRC]
    kernel.LoadResource.restype = wintypes.HGLOBAL
    kernel.LockResource.argtypes = [wintypes.HGLOBAL]
    kernel.LockResource.restype = ctypes.c_void_p
    # LOAD_LIBRARY_AS_DATAFILE reads PE resources without loading/executing its application code.
    module = kernel.LoadLibraryExW(str(Path(executable).resolve()), None, 0x00000002)
    if not module:
        raise ctypes.WinError(ctypes.get_last_error())

    def resource(kind, name):
        found = kernel.FindResourceW(module, name, kind)
        if not found:
            raise ctypes.WinError(ctypes.get_last_error())
        length = kernel.SizeofResource(module, found)
        if not length or length > 1024 * 1024:
            raise ValueError("invalid/oversized icon resource")
        pointer = kernel.LockResource(kernel.LoadResource(module, found))
        if not pointer:
            raise ctypes.WinError(ctypes.get_last_error())
        return ctypes.string_at(pointer, length)

    callback_type = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HMODULE, ctypes.c_void_p, ctypes.c_void_p, wintypes.LPARAM)
    groups = []

    @callback_type
    def collect(_module, _kind, name, _parameter):
        # Numeric resource ID or copy its string while the callback pointer is valid.
        value = name or 0
        groups.append(value if value <= 65535 else ctypes.wstring_at(value))
        return True

    kernel.EnumResourceNamesW.argtypes = [wintypes.HMODULE, ctypes.c_void_p, callback_type, wintypes.LPARAM]
    kernel.EnumResourceNamesW.restype = wintypes.BOOL
    try:
        if not kernel.EnumResourceNamesW(module, 14, collect, 0):
            raise ctypes.WinError(ctypes.get_last_error())
        for group in groups:
            name_buffer = ctypes.create_unicode_buffer(group) if isinstance(group, str) else None
            name = ctypes.cast(name_buffer, ctypes.c_void_p) if name_buffer is not None else group
            data = resource(14, name)
            if len(data) != 6 + 14 * len(expected) or data[:6] != struct.pack("<HHH", 0, 1, len(expected)):
                continue
            matches = True
            for index, (wanted, png) in enumerate(expected):
                fields = struct.unpack_from("<BBBBHHIH", data, 6 + index * 14)
                if fields[:-1] != wanted or resource(3, fields[-1]) != png:
                    matches = False
                    break
            if matches:
                print("Real PE icon group: all nine resource PNGs byte-match the approved ICO")
                return
        raise ValueError("No PE icon group matches the accepted product ICO")
    finally:
        kernel.FreeLibrary(module)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--exe", type=Path)
    parser.add_argument("--require-icns", action="store_true")
    args = parser.parse_args()
    check_manifest(args.require_icns)
    ico_entries((DIRECTORY / "EffectStrike.ico").read_bytes())
    if args.exe:
        check_windows_executable(args.exe)
