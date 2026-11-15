"""Independent cross-language compatibility check; offline, no browser calls."""
import argparse
import json
from pathlib import Path
import subprocess
import sys
import tempfile
from test_huw import huw, form


def compare_forms(root, rust_binary, tmp):
    """Drive both real CLIs with an offline transport, never a real browser."""
    transport = tmp / "fake_browser.py"
    transport.write_text('''import json, sys
from pathlib import Path
fixture = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
args = sys.argv[sys.argv.index("--") + 1:]
if args == ["get", "url"]: print("https://higgsfield.ai/")
elif args[0:2] == ["get", "count"]:
    print("0" if fixture.get("missing_switch") and "role=\\\"switch\\\"" in args[2] else "1")
elif args[0:2] == ["get", "html"]: print(fixture["html"])
elif args[0] == "eval": print(json.dumps(fixture["evidence"]))
else: raise AssertionError("unexpected mutation")
''', encoding="utf-8")
    settings = ''.join(f'<button aria-label="{k}">{v}</button>' for k, v in
                       [('Duration','8s'),('Ratio','Auto'),('Quality','720p'),('Bitrate','High'),('Audio','ON')])
    cases = [form(), form(extra=settings), form('false', 'Generate 90'),
             form(disabled='data-disabled'), form(extra='<button aria-label="Duration"></button>'),
             form(label='생성하기 무제한').replace('Unlimited mode', 'Unlimited 모드'),
             form(extra=settings + '<button aria-label="Audio">OFF</button>')]
    fixtures = [{"html": html} for html in cases]
    for ready, login in [('complete', True), ('loading', False), ('complete', False), ('mystery', False)]:
        fixtures.append({'html': form().replace('Unlimited mode', 'Changed label'), 'missing_switch': True,
                         'evidence': {'ready': ready, 'login': login}})
    for index, fixture in enumerate(fixtures):
        path = tmp / 'fixture.json'
        path.write_text(json.dumps(fixture), encoding='utf-8')
        config = tmp / 'config.json'
        config.write_text(json.dumps({'command': [sys.executable, '-X', 'utf8', str(transport), str(path)], 'browser': 'synthetic'}), encoding='utf-8')
        commands = [[sys.executable, '-X', 'utf8', str(root / 'skills/higgsfield-unlimited-watch/scripts/huw.py')], [rust_binary]]
        outputs = [subprocess.run(c + ['inspect', '--config', str(config)], capture_output=True, text=True, encoding='utf-8') for c in commands]
        assert outputs[0].returncode == outputs[1].returncode, (index, [o.stderr for o in outputs])
        values = [json.loads(o.stdout if o.returncode == 0 else o.stderr) for o in outputs]
        assert values[0] == values[1], (index, values)
    print(f'Python/Rust inspect CLI parity: {len(fixtures)} offline form/diagnostic cases match')

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
        compare_forms(root, args.rust, Path(tmp))
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
