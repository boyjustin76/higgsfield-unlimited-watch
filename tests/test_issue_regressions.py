"""Sanitized reproductions for public issues #1, #2 and #4."""
import json
import subprocess
import tempfile
import unittest
from unittest.mock import patch
from test_huw import huw, form, asset, A


class SettingsRegressionTests(unittest.TestCase):
    def test_missing_settings_are_explicit(self):
        state = huw.form_state(form())
        self.assertIs(state.get('settings_complete'), False)
        self.assertEqual(state['missing_settings'], ['Duration', 'Ratio', 'Quality', 'Bitrate', 'Audio'])
        self.assertTrue(state['unlimited_verified'])

    def test_observed_labels_without_old_attribute(self):
        controls = ''.join(f'<button aria-label="{k}">{v}</button>' for k, v in
                           [('Duration', '8s'), ('Ratio', 'Auto'), ('Quality', '720p'), ('Bitrate', 'High'), ('Audio', 'ON')])
        self.assertIs(huw.form_state(form(extra=controls)).get('settings_complete'), True)

    def test_unknown_label_and_empty_value_do_not_complete_settings(self):
        state = huw.form_state(form(extra='<button aria-label="Duration"></button><button aria-label="Sound">ON</button>'))
        self.assertIn('Duration', state.get('missing_settings', []))
        self.assertIn('Audio', state['missing_settings'])

    def test_duplicate_setting_fails_closed(self):
        with self.assertRaises(huw.GuardError):
            huw.form_state(form(extra='<button data-form-kit-setting-button aria-label="Audio">ON</button>' * 2))

    def test_data_disabled_cannot_verify(self):
        self.assertFalse(huw.form_state(form(disabled='data-disabled'))['unlimited_verified'])


class HistoryRegressionTests(unittest.TestCase):
    def test_nested_agreement_and_descendant_video(self):
        for status, expected in [('queued', 'pending'), ('in_progress', 'pending'), ('completed', 'completed')]:
            markup = f'<div data-asset-id="{A}" data-job-status="{status}">{asset(status)}</div>'
            self.assertEqual(huw.history_state(markup, A)['state'], expected)

    def test_conflicting_and_separate_cards_remain_unknown(self):
        for markup in [f'<div data-asset-id="{A}" data-job-status="queued">{asset()}</div>', asset() + asset(), '',
                       f'<div data-asset-id="{A}" data-job-status="completed">{asset(video=False)}</div>']:
            self.assertEqual(huw.history_state(markup, A)['state'], 'unknown')


class DiagnosticRegressionTests(unittest.TestCase):
    def test_missing_switch_with_existing_form_diagnosed(self):
        cases = [('complete', True, 'signed_out'), ('loading', False, 'page_not_ready'),
                 ('complete', False, 'missing_control'), ('mystery', False, 'unknown')]
        for ready, login, expected in cases:
            browser = huw.Browser({'command': ['browser'], 'browser': 'test'})
            def action(*args):
                if args == ('get', 'url'):
                    return 'https://higgsfield.ai/'
                if args[0] == 'eval':
                    return json.dumps({'ready': ready, 'login': login})
                if args[:2] == ('get', 'count'):
                    return '0' if args[2] == huw.SELECTORS['unlimited'] else '1'
                raise AssertionError(args)
            with self.subTest(expected=expected), patch.object(browser, 'action', side_effect=action):
                with self.assertRaisesRegex(huw.GuardError, expected):
                    browser.inspect()

    def test_new_tab_off_is_not_inherited(self):
        self.assertTrue(huw.form_state(form())['unlimited_verified'])
        state = huw.form_state(form('false', 'Generate 90'))
        self.assertFalse(state['unlimited_verified'])


class RelayRegressionTests(unittest.TestCase):
    def test_started_false_is_uncertain_and_not_retried(self):
        with tempfile.TemporaryDirectory() as root:
            huw.demo(root)
            outputs = [subprocess.CompletedProcess([], 0, json.dumps({'address': 't1', 'state': 'live', 'status': {'app': 'codex', 'state': 'idle'}})),
                       subprocess.CompletedProcess([], 0, '{"started":false}')]
            with patch.object(huw.subprocess, 'run', side_effect=outputs) as run:
                first = huw.relay(root, 'hc', 't1', expected_app='codex', runner=run)
                self.assertEqual(first['state'], 'delivery_unknown')
                self.assertEqual(huw.relay(root, 'hc', 't1', expected_app='codex', runner=run), first)
                self.assertEqual(run.call_count, 2)

