import copy
import importlib.util
import json
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parent.parent / "scripts/prepare-waybar.py"
SPEC = importlib.util.spec_from_file_location("prepare_waybar", SCRIPT)
PREPARE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PREPARE)


class WaybarPreparation(unittest.TestCase):
    def config(self):
        return [{
            "modules-center": ["clock", "custom/pomodoro", "custom/dnd"],
            "custom/dnd": {"exec": "old-status", "on-click": "old-toggle",
                           "interval": 5, "format": "{}", "return-type": "json"},
            "custom/pomodoro": {"exec": "timer watch", "format-icons": {"focus-running": "green"}},
            "output": output,
        } for output in ["laptop", "monitor"]]

    def test_only_dnd_commands_change_on_both_panels(self):
        original = self.config()
        saved = copy.deepcopy(original)
        prepared, count = PREPARE.prepare(original, Path("/test/waybar-dnd"))
        self.assertEqual(count, 2)
        self.assertEqual(original, saved)
        for bar in prepared:
            self.assertEqual(bar["custom/dnd"]["exec"], "/test/waybar-dnd status")
            self.assertEqual(bar["custom/dnd"]["on-click"], "/test/waybar-dnd toggle")
            bar["custom/dnd"].update(exec="old-status", **{"on-click": "old-toggle"})
        self.assertEqual(prepared, original)

    def test_object_config_and_shell_quoting(self):
        binary = Path("/test/with spaces/'$(nope)/waybar-dnd")
        prepared, count = PREPARE.prepare(self.config()[0], binary)
        self.assertEqual(count, 1)
        self.assertEqual(shlex.split(prepared["custom/dnd"]["exec"]), [str(binary), "status"])
        again, _ = PREPARE.prepare(prepared, binary)
        self.assertEqual(again, prepared)

    def test_bars_without_dnd_remain_unchanged(self):
        original = self.config() + [{"modules-center": ["clock"]}]
        prepared, count = PREPARE.prepare(original, Path("/bin/waybar-dnd"))
        self.assertEqual(count, 2)
        self.assertEqual(prepared[-1], original[-1])

    def test_invalid_configs_are_rejected(self):
        for config in [[], None, [1], {}, {"custom/dnd": "wrong"}]:
            with self.subTest(config=config), self.assertRaises(ValueError):
                PREPARE.prepare(config, Path("/bin/waybar-dnd"))

    def test_preview_preserves_source_symlink_and_does_not_overwrite_review(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "managed.json"
            source.write_text(json.dumps(self.config()))
            original = source.read_bytes()
            config = root / "active.json"
            config.symlink_to(source)
            output = root / "review"
            command = [sys.executable, str(SCRIPT), "--config", str(config),
                       "--binary", str(root / "waybar-dnd"), "--output", str(output)]
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(config.is_symlink())
            self.assertEqual(source.read_bytes(), original)
            self.assertEqual((output / "config.before.json").read_bytes(), original)
            self.assertEqual(json.loads((output / "manifest.json").read_text())["panels"], 2)
            self.assertFalse(json.loads((output / "manifest.json").read_text())["applied"])
            preview = (output / "config.json").read_bytes()
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual((output / "config.json").read_bytes(), preview)
            self.assertEqual(source.read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
