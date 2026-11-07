import importlib.util
import html
import io
import json
import os
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from contextlib import redirect_stdout, redirect_stderr

SCRIPT = Path(__file__).resolve().parents[1] / "skills/higgsfield-unlimited-watch/scripts/huw.py"
spec = importlib.util.spec_from_file_location("huw", SCRIPT)
huw = importlib.util.module_from_spec(spec)
spec.loader.exec_module(huw)
A = "00000000-0000-4000-8000-000000000001"
B = "00000000-0000-4000-8000-000000000002"
LIVE = json.dumps({"address": "t1", "state": "live", "status": {"app": "copilot", "state": "idle"}})


def form(checked="true", label="Generate Unlimited", disabled="", extra="", switch_disabled=""):
    return (f'<div data-prompt-field="true"><div role="textbox" contenteditable="true" '
            'data-lexical-editor="true">A <span>paper</span> bird</div></div>'
            '<button data-model-select-trigger="true">Seedance 2.0 Unlimited</button>'
            f'<span role="switch" aria-label="Unlimited mode" aria-checked="{checked}" {switch_disabled}></span>'
            f'<div data-tour-anchor="tour-generate-button"><button type="submit" {disabled}>{label}</button></div>' + extra)


def asset(status="completed", aid=A, video=True):
    return (f'<div data-asset-id="{aid}" data-job-status="{status}">'
            + ('<video></video>' if video else '<img src="private.png">') + '</div>')


class FormTests(unittest.TestCase):
    def test_prompt_line_boundaries_and_empty_editor(self):
        examples = {
            '<p>first</p><p><br></p><p>last</p>': 'first\n\nlast',
            '<div>first</div><div>last</div>': 'first\nlast',
            '<p>first<br>last</p>': 'first\nlast',
            '<p> first\t </p><p><br></p>': ' first\t \n',
            '<p><br></p>': '', '<br>': '', 'first<br><br>last': 'first\n\nlast',
        }
        for markup, expected in examples.items():
            with self.subTest(markup=markup):
                page = form().replace('A <span>paper</span> bird', markup)
                self.assertEqual(huw.form_state(page)['prompt_sha256'], huw.digest(expected))

    def test_verified(self):
        self.assertTrue(huw.form_state(form())["unlimited_verified"])

    def test_badge_not_toggle(self):
        self.assertFalse(huw.form_state(form("false"))["unlimited_verified"])

    def test_credit_label_rejected(self):
        for text in ("Generate 90", "Generate Unlimited 90", "Unlimited loading", "Generate"):
            self.assertFalse(huw.form_state(form(label=text))["unlimited_verified"])

    def test_disabled_rejected(self):
        for value in ('disabled', 'aria-disabled="true"'):
            self.assertFalse(huw.form_state(form(disabled=value))["unlimited_verified"])
            self.assertFalse(huw.form_state(form(switch_disabled=value))["unlimited_verified"])

    def test_korean(self):
        html = form(label="생성하기 무제한").replace('Unlimited mode', 'Unlimited 모드')
        self.assertTrue(huw.form_state(html)["unlimited_verified"])

    def test_duplicate_stops(self):
        with self.assertRaises(huw.GuardError):
            huw.form_state(form(extra='<span role="switch" aria-label="Unlimited mode"></span>'))

    def test_text_preserves_inline_spaces(self):
        self.assertEqual(huw.form_state(form())["prompt_sha256"], huw.digest("A paper bird"))

    def test_private_values_not_output(self):
        output = json.dumps(huw.form_state(form(extra='<div data-tour-anchor="tour-video-reference"><img src="https://private.example/image"></div>')))
        self.assertNotIn('private.example', output)
        self.assertNotIn('A paper bird', output)


class HistoryTests(unittest.TestCase):
    def test_completed(self):
        self.assertEqual(huw.history_state(asset(), A)["state"], "completed")

    def test_missing_and_duplicate(self):
        for html in (asset(aid=B), asset() + asset()):
            self.assertEqual(huw.history_state(html, A)["state"], "unknown")

    def test_id_ignores_order(self):
        self.assertEqual(huw.history_state(asset(aid=B) + asset("processing"), A)["state"], "pending")

    def test_no_video_no_completion(self):
        self.assertEqual(huw.history_state(asset(video=False), A)["state"], "unknown")

    def test_candidates(self):
        for status in huw.PENDING:
            self.assertEqual(huw.history_state(asset(status), A)["state"], "pending")
        for status in huw.FAILED:
            self.assertEqual(huw.history_state(asset(status), A)["state"], "failed")
        self.assertEqual(huw.history_state(asset("new_status"), A)["state"], "unknown")


class BrowserTests(unittest.TestCase):
    def test_shell_free_prompt_argument(self):
        browser = huw.Browser({"command": ["wsl.exe", "-d", "Ubuntu", "--exec", "/bin/terminal-browser"], "browser": "session"})
        prompt = 'hello "quoted"\n$(whoami); `secret`'
        with patch.object(huw.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, "ok", "")) as run:
            browser.action("fill", "selector", prompt)
        self.assertEqual(run.call_args.args[0][-1], prompt)
        self.assertFalse(run.call_args.kwargs["shell"])

    def test_origin_guard(self):
        browser = huw.Browser({"command": ["terminal-browser"], "browser": "session"})
        for url in ("https://higgsfield.ai.evil.test", "http://higgsfield.ai", "https://example.org"):
            with patch.object(browser, "action", return_value=url), self.assertRaises(huw.GuardError):
                browser.check_origin()

    def test_transport_error_does_not_leak(self):
        browser = huw.Browser({"command": ["terminal-browser"], "browser": "session"})
        with patch.object(huw.subprocess, "run", return_value=subprocess.CompletedProcess([], 1, "private prompt", "secret URL")):
            with self.assertRaises(huw.GuardError) as exc:
                browser.action("get", "html", "main")
        self.assertNotIn("secret", str(exc.exception))

    def test_ensure_only_clicks_off_switch(self):
        browser = huw.Browser({"command": ["terminal-browser"], "browser": "session"})
        states = [huw.form_state(form("false")), huw.form_state(form())]
        with patch.object(browser, "inspect", side_effect=states), patch.object(browser, "action") as act:
            browser.ensure_unlimited()
        act.assert_called_once_with("click", huw.SELECTORS["unlimited"])

    def test_prompt_clear_failure_never_fills(self):
        browser = huw.Browser({"command": ["terminal-browser"], "browser": "session"})
        with patch.object(browser, "inspect", return_value={"prompt_length": 3}), patch.object(browser, "action") as act:
            with self.assertRaises(huw.GuardError):
                browser.replace_prompt("new prompt")
        self.assertNotIn("fill", [call.args[0] for call in act.call_args_list])

    def test_prompt_readback_mismatch_stops(self):
        browser = huw.Browser({"command": ["terminal-browser"], "browser": "session"})
        with patch.object(browser, "inspect", side_effect=[{}, {"prompt_length": 0}, {"prompt_sha256": "wrong"}]), patch.object(browser, "action"):
            with self.assertRaises(huw.GuardError):
                browser.replace_prompt("new prompt")

    def test_already_on_not_toggled_off(self):
        browser = huw.Browser({"command": ["terminal-browser"], "browser": "session"})
        with patch.object(browser, "inspect", return_value=huw.form_state(form())), patch.object(browser, "action") as act:
            browser.ensure_unlimited()
        act.assert_not_called()


class WatchTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.tick = 0

    def tearDown(self):
        self.temp.cleanup()

    def sleep(self, seconds):
        self.tick += seconds

    def watch(self, states, **kw):
        iterator = iter(states)
        def observe(aid):
            state = next(iterator)
            if isinstance(state, Exception):
                raise state
            return {"state": state, "asset_id": aid}
        return huw.watch(observe, A, self.root, clock=lambda: self.tick, sleep=self.sleep, **kw)

    def test_two_confirmations_and_replay(self):
        event = self.watch(["pending", "completed", "completed"])
        self.assertEqual(event["polls"], 3)
        self.assertEqual(event["monitor_llm_calls"], 0)
        replay = self.watch([])
        self.assertEqual(replay, event)
        self.assertFalse((self.root / "monitor.lock").exists())

    def test_disruption_resets_confirmations(self):
        event = self.watch(["completed", huw.GuardError("lost"), "completed", "completed"])
        self.assertEqual(event["polls"], 4)

    def test_timeout_is_not_failure_and_explicit_resume(self):
        event = self.watch(["unknown", "unknown"], timeout=5)
        self.assertEqual(event["state"], "timeout")
        self.assertEqual(self.watch([]), event)
        resumed = self.watch(["completed", "completed"], resume=True)
        self.assertEqual(resumed["attempt"], 2)

    def test_failure(self):
        self.assertEqual(self.watch(["failed", "failed"])["state"], "failed")

    def test_asset_cannot_change(self):
        self.watch(["completed", "completed"])
        with self.assertRaises(huw.GuardError):
            huw.watch(lambda _: {}, B, self.root)

    def test_exclusive_monitor(self):
        with huw.lock(self.root), self.assertRaises(huw.GuardError):
            self.watch([])

    def test_cancel_stops_monitor_not_job(self):
        def observe(_):
            raise KeyboardInterrupt()
        event = huw.watch(observe, A, self.root)
        self.assertEqual(event["state"], "cancelled")
        self.assertIn("job_not_cancelled", event["reason"])

    def test_relay_only_once(self):
        self.watch(["completed", "completed"])
        with patch.object(huw.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, LIVE, "")) as run:
            first = huw.relay(self.root, "hc", "t1", expected_app="copilot", runner=run)
            second = huw.relay(self.root, "hc", "t1", expected_app="copilot", runner=run)
        self.assertEqual(first, second)
        self.assertEqual(run.call_count, 2)
        self.assertEqual(first["state"], "accepted")

    def test_relay_ambiguous_not_retried(self):
        self.watch(["completed", "completed"])
        with patch.object(huw.subprocess, "run", side_effect=[subprocess.CompletedProcess([], 0, LIVE, ""), subprocess.TimeoutExpired("hc", 30)]) as run:
            first = huw.relay(self.root, "hc", "t1", expected_app="copilot", runner=run)
            huw.relay(self.root, "hc", "t1", expected_app="copilot", runner=run)
        self.assertEqual(first["state"], "delivery_unknown")
        self.assertEqual(run.call_count, 2)

    def test_relay_crash_claim_preserved(self):
        self.watch(["completed", "completed"])
        def crash(*args, **kwargs):
            if args[0][1] == "get":
                return subprocess.CompletedProcess([], 0, LIVE, "")
            raise KeyboardInterrupt()
        with self.assertRaises(KeyboardInterrupt):
            huw.relay(self.root, "hc", "t1", expected_app="copilot", runner=crash)
        self.assertEqual(huw.relay(self.root, "hc", "t1", expected_app="copilot", runner=crash)["state"], "attempting")

    def test_relay_does_not_type_into_shell_or_dialog(self):
        self.watch(["completed", "completed"])
        for status in (None, {"app": "copilot", "state": "blocked"}, {"app": "copilot", "state": "working"}):
            data = json.dumps({"state": "live", "status": status})
            with patch.object(huw.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, data, "")) as run:
                with self.assertRaises(huw.GuardError):
                    huw.relay(self.root, "hc", "t1", expected_app="copilot", runner=run)
                self.assertEqual(run.call_count, 1)

    def test_relay_wrong_app_or_address_rejected_without_receipt(self):
        self.watch(["completed", "completed"])
        for app, address in [('bash', 't1'), ('vim', 't1'), ('codex', 't1'), ('copilot', 't2')]:
            data = json.dumps({'address': address, 'state': 'live', 'status': {'app': app, 'state': 'idle'}})
            with patch.object(huw.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0, data, '')) as run:
                with self.assertRaises(huw.GuardError):
                    huw.relay(self.root, 'hc', 't1', expected_app='copilot', runner=run)
                self.assertEqual(run.call_count, 1)
                self.assertEqual(list(self.root.glob('relay-*.json')), [])

    def test_relay_unknown_expected_app_rejected(self):
        with self.assertRaises(huw.GuardError):
            huw.relay(self.root, 'hc', 't1', expected_app='bash')

    def test_relay_legacy_receipt_does_not_resend(self):
        event = self.watch(['completed', 'completed'])
        huw.atomic_json(self.root / f"relay-t1-{event['event_id']}.json", {'state': 'attempting'})
        with patch.object(huw.subprocess, 'run') as run, self.assertRaises(huw.GuardError):
            huw.relay(self.root, 'hc', 't1', expected_app='copilot', runner=run)
        run.assert_not_called()

    def test_invalid_asset(self):
        with self.assertRaises(huw.GuardError):
            huw.watch(lambda _: {}, '../secret', self.root)

    def test_nonfinite_deadlines_rejected(self):
        for value in (float('nan'), float('inf')):
            with self.assertRaises(huw.GuardError):
                self.watch([], timeout=value)
            with self.assertRaises(huw.GuardError):
                self.watch([], interval=value)


class CLITests(unittest.TestCase):
    def test_prompt_file_whitespace_roundtrip(self):
        expected = ' \tA paper crane.\n\nSoft light.\t \n'
        for rendering in ('literal', 'br', 'blocks'):
            with self.subTest(rendering=rendering), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                config = root / 'config.json'
                config.write_text(json.dumps({'command': ['terminal-browser'], 'browser': 'session'}), encoding='utf-8')
                prompt = root / 'prompt.txt'
                prompt.write_bytes(b'\xef\xbb\xbf' + expected.replace('\n', '\r\n').encode('utf-8'))
                editor = {'html': 'old prompt', 'filled': None}
                def action(_browser, *args):
                    if args == ('get', 'url'): return 'https://higgsfield.ai/ai/video'
                    if args[:2] == ('get', 'count'): return '1'
                    if args[:2] == ('get', 'html'):
                        return form().replace('A <span>paper</span> bird', editor['html'])
                    if args == ('press', 'Backspace'): editor['html'] = '<p><br></p>'
                    if args[0] == 'fill':
                        editor['filled'] = args[2]
                        escaped = html.escape(args[2])
                        editor['html'] = (escaped if rendering == 'literal' else escaped.replace('\n', '<br>')
                                          if rendering == 'br' else ''.join('<p>' + (line or '<br>') + '</p>' for line in escaped.split('\n')))
                    return ''
                output = io.StringIO()
                with patch.object(huw.Browser, 'action', action), redirect_stdout(output):
                    result = huw.main(['set-prompt', '--config', str(config), '--file', str(prompt)])
                self.assertEqual(result, 0)
                self.assertEqual(editor['filled'], expected)
                self.assertEqual(json.loads(output.getvalue())['prompt_sha256'], huw.digest(expected))

    def test_notification_requires_app_before_starting_monitor(self):
        with self.assertRaises(SystemExit), patch.object(huw, 'watch') as watch, redirect_stderr(io.StringIO()):
            huw.main(['watch', '--config', 'unused', '--asset-id', A, '--state-dir', 'unused', '--notify-terminal', 't1'])
        watch.assert_not_called()

    def test_demo_offline(self):
        with tempfile.TemporaryDirectory() as tmp:
            r = subprocess.run([huw.sys.executable, str(SCRIPT), "demo", "--state-dir", tmp],
                               capture_output=True, text=True, check=True)
            result = json.loads(r.stdout)
            self.assertTrue(result["synthetic"])
            self.assertEqual(result["event"]["state"], "completed")


if __name__ == "__main__":
    unittest.main()
