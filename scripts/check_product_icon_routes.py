#!/usr/bin/env python3
"""Source routing contract, not native desktop/installer acceptance."""
from pathlib import Path
import hashlib
import json
import unittest

ROOT = Path(__file__).resolve().parents[1]
ICONS = "assets/branding/icons"


class ProductIconRoutes(unittest.TestCase):
    def test_approved_master_is_unchanged(self):
        self.assertEqual(hashlib.sha256((ROOT / "assets/branding/EffectStrike-logo.png").read_bytes()).hexdigest(),
                         "d53cdb8752db48d9b3a5c7b4d3f2b813cfa3e5408140ab7c2e1a752bf6a56181")
        self.assertEqual(hashlib.sha256((ROOT / "assets/branding/EffectStrike-logo.svg").read_bytes()).hexdigest(),
                         "5be41df5aa639f6b0557b1d8c0f40b4365dfb1a370b37bdd1f30b480e70c7973")

    def test_native_and_package_consumers_route_to_product_assets(self):
        routes = {
            "apps/effectcraft/build.rs": f"{ICONS}/EffectStrike.ico",
            "packaging/windows/package.ps1": "assets\\branding\\icons\\EffectStrike.ico",
            "packaging/macos/package.sh": f"{ICONS}/EffectStrike.icns",
            "packaging/linux/package.sh": f"{ICONS}/hicolor",
            "packaging/freebsd/package.sh": f"{ICONS}/hicolor",
            "packaging/linux/flatpak/ai.storyteller.effectcraft.yml": f"{ICONS}/hicolor",
            "xtask/src/main.rs": f"{ICONS}/hicolor",
        }
        for path, expected in routes.items():
            with self.subTest(path=path):
                source = (ROOT / path).read_text()
                self.assertIn(expected, source)
                self.assertNotIn("assets/app-icon/", source)
                self.assertNotIn("assets\\app-icon\\", source)
        main = (ROOT / "apps/effectcraft/src/main.rs").read_text()
        self.assertNotIn("effectcraft-macos-512.png", main)
        self.assertIn("assets/branding/EffectStrike-logo.png", main)

    def test_web_svg_manifest_and_png_sources_agree(self):
        html = (ROOT / "apps/effectcraft-web/web/index.html").read_text()
        manifest = json.loads((ROOT / "apps/effectcraft-web/web/manifest.webmanifest").read_text())
        self.assertIn('href="effectstrike-logo.svg"', html)
        self.assertIn('src="effectstrike-logo.svg"', html)
        self.assertEqual(manifest["icons"][0]["src"], "effectstrike-logo.svg")
        self.assertEqual({item["src"] for item in manifest["icons"][1:]}, {"icon-256.png", "icon-512.png"})
        task = (ROOT / "xtask/src/main.rs").read_text()
        self.assertIn('dist.join("effectstrike-logo.svg")', task)
        self.assertIn('assets/branding/EffectStrike-logo.svg', task)


if __name__ == "__main__":
    unittest.main()
