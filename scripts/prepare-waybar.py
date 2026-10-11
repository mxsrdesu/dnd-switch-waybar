#!/usr/bin/env python3
"""prepare an existing waybar config for waybar-dnd without applying changes."""

import argparse
import copy
import difflib
import hashlib
import json
from pathlib import Path
import shlex


def prepare(config, binary):
    result = copy.deepcopy(config)
    bars = result if isinstance(result, list) else [result]
    if not bars or not all(isinstance(bar, dict) for bar in bars):
        raise ValueError("expected a waybar object or a nonempty list of bars")
    command = shlex.quote(str(binary))
    count = 0
    for bar in bars:
        if "custom/dnd" not in bar:
            continue
        module = bar["custom/dnd"]
        if not isinstance(module, dict):
            raise ValueError("custom/dnd must be an object")
        module["exec"] = f"{command} status"
        module["on-click"] = f"{command} toggle"
        count += 1
    if not count:
        raise ValueError("no existing custom/dnd module found")
    return result, count


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", required=True, type=Path, help="the active waybar json config")
    parser.add_argument("--binary", required=True, type=Path, help="the binary path to use after installation")
    parser.add_argument("--output", required=True, type=Path, help="a new directory for preview files")
    args = parser.parse_args()
    source = args.config.expanduser().resolve(strict=True)
    binary = args.binary.expanduser().absolute()
    destination = args.output.expanduser().absolute()
    original = source.read_bytes()
    config, count = prepare(json.loads(original), binary)
    rendered = json.dumps(config, ensure_ascii=False, indent=2) + "\n"
    patch = "".join(difflib.unified_diff(
        original.decode().splitlines(keepends=True), rendered.splitlines(keepends=True),
        fromfile=str(args.config.expanduser().absolute()),
        tofile=str(args.config.expanduser().absolute()),
    ))
    # Never overwrite an earlier review or write through an output symlink.
    destination.mkdir(parents=True, exist_ok=False)
    (destination / "config.before.json").write_bytes(original)
    (destination / "config.json").write_text(rendered)
    (destination / "config.patch").write_text(patch)
    (destination / "manifest.json").write_text(json.dumps({
        "source": str(args.config.expanduser().absolute()),
        "resolved_source": str(source),
        "source_sha256": hashlib.sha256(original).hexdigest(),
        "prepared_sha256": hashlib.sha256(rendered.encode()).hexdigest(),
        "target_binary": str(binary),
        "panels": count,
        "applied": False,
    }, indent=2) + "\n")
    print(f"prepared dnd commands for {count} panels in {destination}; nothing applied")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError) as error:
        raise SystemExit(f"could not prepare dnd: {error}") from error
