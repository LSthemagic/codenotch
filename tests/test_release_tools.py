"""Release tooling tests: real files and CLI processes, no network or providers."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / "scripts" / "release_tools.py"


class ReleaseToolsTests(unittest.TestCase):
    def setUp(self):
        self.assertTrue(SCRIPT.is_file(), "release_tools.py has not been implemented")
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        (self.root / "nyrva").mkdir()
        self.configure("0.3.0", "0.3.0")

    def configure(self, tauri, cargo):
        (self.root / "nyrva/tauri.conf.json").write_text(
            json.dumps({"version": tauri}), encoding="utf-8")
        (self.root / "nyrva/Cargo.toml").write_text(
            f'[package]\nname = "nyrva"\nversion = "{cargo}"\n', encoding="utf-8")

    def run_tool(self, *arguments):
        return subprocess.run([sys.executable, str(SCRIPT), *map(str, arguments)],
                              capture_output=True, text=True, check=False)

    def tag(self, name="v0.3.0", ref_type="tag"):
        return self.run_tool("validate-tag", "--root", self.root,
                             "--ref-type", ref_type, "--tag", name)

    def packages(self):
        source = self.root / "artifacts"
        for folder, name in [("windows", "Nyrva_0.3.0_x64-setup.exe"),
                             ("linux/deb", "Nyrva_0.3.0_amd64.deb"),
                             ("linux/appimage", "Nyrva_0.3.0_amd64.AppImage")]:
            file = source / folder / name
            file.parent.mkdir(parents=True, exist_ok=True)
            file.write_bytes(("fixture:" + name).encode())
        return source

    def stage(self, source):
        return self.run_tool("prepare-assets", "--source", source,
                             "--output", self.root / "dist")

    def test_matching_version_is_accepted(self):
        result = self.tag()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("v0.3.0", result.stdout)

    def test_prerelease_is_accepted_when_all_versions_match(self):
        self.configure("0.3.1-rc.1", "0.3.1-rc.1")
        result = self.tag("v0.3.1-rc.1")
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_wrong_tag_version_is_rejected(self):
        result = self.tag("v9.9.9")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("does not match", result.stderr)

    def test_cargo_and_tauri_drift_is_rejected(self):
        self.configure("0.3.0", "0.3.1")
        result = self.tag()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("versions differ", result.stderr)

    def test_branch_event_is_rejected(self):
        result = self.tag(ref_type="branch")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("tag event", result.stderr)

    def test_invalid_tag_is_rejected(self):
        for tag in ["0.3.0", "v0.3.0;echo bad", "v0.3.0\n", "main"]:
            with self.subTest(tag=tag):
                result = self.tag(tag)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("Invalid version tag", result.stderr)

    def test_invalid_config_fails_cleanly(self):
        (self.root / "nyrva/tauri.conf.json").write_text("{", encoding="utf-8")
        result = self.tag()
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn("Traceback", result.stderr)

    def test_missing_config_fails_cleanly(self):
        (self.root / "nyrva/Cargo.toml").unlink()
        result = self.tag()
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn("Traceback", result.stderr)

    def test_nested_packages_are_flattened_and_hashed(self):
        source = self.packages()
        originals = {p.name: p.read_bytes() for p in source.rglob("*") if p.is_file()}
        result = self.stage(source)
        self.assertEqual(result.returncode, 0, result.stderr)
        output = self.root / "dist"
        self.assertEqual({p.name for p in output.iterdir()}, set(originals) | {"SHA256SUMS"})
        expected = "".join(f"{hashlib.sha256(data).hexdigest()}  {name}\n"
                           for name, data in sorted(originals.items()))
        self.assertEqual((output / "SHA256SUMS").read_text(), expected)
        for name, data in originals.items():
            self.assertEqual((output / name).read_bytes(), data)
        self.assertEqual({p.name: p.read_bytes() for p in source.rglob("*") if p.is_file()}, originals)

    def test_each_missing_package_format_blocks_staging(self):
        for extension in [".exe", ".deb", ".AppImage"]:
            with self.subTest(extension=extension):
                source = self.packages()
                next(source.rglob("*" + extension)).unlink()
                result = self.stage(source)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("Expected exactly one", result.stderr)
                self.assertFalse((self.root / "dist").exists())

    def test_duplicate_package_blocks_staging(self):
        source = self.packages()
        (source / "duplicate.exe").write_bytes(b"duplicate")
        result = self.stage(source)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Expected exactly one", result.stderr)
        self.assertFalse((self.root / "dist").exists())

    def test_empty_package_blocks_staging(self):
        source = self.packages()
        next(source.rglob("*.deb")).write_bytes(b"")
        result = self.stage(source)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("empty", result.stderr)
        self.assertFalse((self.root / "dist").exists())

    def test_existing_output_is_never_overwritten(self):
        source = self.packages()
        output = self.root / "dist"
        output.mkdir()
        (output / "keep.txt").write_text("keep", encoding="utf-8")
        result = self.stage(source)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual((output / "keep.txt").read_text(), "keep")

    def test_missing_artifacts_directory_fails_cleanly(self):
        result = self.stage(self.root / "missing")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("directory", result.stderr)
        self.assertNotIn("Traceback", result.stderr)

    def test_unrelated_files_are_not_published(self):
        source = self.packages()
        (source / "debug.log").write_text("diagnostics", encoding="utf-8")
        result = self.stage(source)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse((self.root / "dist/debug.log").exists())

    def test_unsafe_asset_name_is_rejected(self):
        source = self.packages()
        original = next(source.rglob("*.exe"))
        original.rename(original.with_name("bad#label.exe"))
        result = self.stage(source)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Unsafe asset filename", result.stderr)

    def test_symlink_package_is_rejected(self):
        source = self.packages()
        original = next(source.rglob("*.exe"))
        real = self.root / "external-installer"
        original.replace(real)
        try:
            original.symlink_to(real)
        except OSError:
            self.skipTest("symlinks are unavailable on this host")
        result = self.stage(source)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("symlink", result.stderr)


if __name__ == "__main__":
    unittest.main()
