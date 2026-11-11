"""Independent cross-language compatibility check; offline, no browser calls."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import tempfile

def normalized(value):
    if isinstance(value, dict):
        return {k: normalized(v) for k, v in value.items() if k not in {"observed_at", "created_at"}}
    if isinstance(value, list):
        return [normalized(v) for v in value]
    return value

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--rust", required=True)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    with tempfile.TemporaryDirectory() as tmp:
        py = Path(tmp) / "python"
        rust = Path(tmp) / "rust"
        commands = [([sys.executable, str(root / "skills/higgsfield-unlimited-watch/scripts/huw.py")], py), ([args.rust], rust)]
        results = []
        for command, directory in commands:
            process = subprocess.run(command + ["demo", "--state-dir", str(directory)], capture_output=True, text=True, encoding="utf-8", check=True)
            results.append(normalized(json.loads(process.stdout)))
        assert results[0] == results[1], "CLI event mismatch"
        for name in ["target.json", "event.json", "status.json"]:
            assert normalized(json.loads((py / name).read_text())) == normalized(json.loads((rust / name).read_text())), name
        def journal(directory):
            return [normalized(json.loads(line)) for line in (directory / "transitions.jsonl").read_text().splitlines()]
        assert journal(py) == journal(rust), "transition journal mismatch"
    print("Python/Rust demo output, target, status, event and transition journal match (timestamps excluded)")

if __name__ == "__main__":
    main()
