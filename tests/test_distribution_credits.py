"""Lock distribution notices in the common Windows/Linux Tauri config."""
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]


class DistributionCreditsTests(unittest.TestCase):
    def test_all_distribution_notices_are_common_bundle_resources(self):
        config = json.loads((ROOT / "nyrva/tauri.conf.json").read_text(encoding="utf-8"))
        resources = config["bundle"].get("resources", {})
        expected = {
            "../LICENSE": "LICENSE",
            "../CREDITS.md": "CREDITS.md",
            "glyphs/NOTICE.md": "PROVIDER_GLYPH_NOTICES.md",
        }
        for source, destination in expected.items():
            with self.subTest(source=source):
                self.assertEqual(resources.get(source), destination)
                self.assertTrue((ROOT / "nyrva" / source).is_file())

    def test_upstream_credit_in_readme_and_release_notes(self):
        for name in ("README.md", "CREDITS.md", "docs/releases/v0.3.0.md"):
            with self.subTest(name=name):
                path = ROOT / name
                self.assertTrue(path.is_file(), f"Missing attribution in {name}")
                text = path.read_text(encoding="utf-8")
                self.assertIn("https://github.com/vinzdg/codenotch", text)
                self.assertIn("Im-Midi", text)

    def test_upstream_copyright_and_permission_notice_are_retained(self):
        text = (ROOT / "LICENSE").read_text(encoding="utf-8")
        self.assertIn("Copyright (c) 2026 Vinz", text)
        self.assertIn("Copyright (c) 2026 Im-Midi (NG) and contributors", text)
        self.assertIn("Permission is hereby granted, free of charge", text)
        self.assertIn('THE SOFTWARE IS PROVIDED "AS IS"', text)


if __name__ == "__main__":
    unittest.main()
