"""POSIX behavior tested on a real POSIX filesystem, never inferred from Windows."""
import json
import os
import stat
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock
from test_huw import huw, A, LIVE


@unittest.skipUnless(os.name == 'posix', 'requires POSIX permission semantics')
class PermissionTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.old_umask = os.umask(0o022)

    def tearDown(self):
        os.umask(self.old_umask)
        self.temp.cleanup()

    def test_every_state_artifact_is_private_under_umask_022(self):
        directory = self.root / 'new-parent' / 'job'
        values = iter(['pending', 'completed', 'completed'])
        clock = [0]
        def sleep(seconds): clock[0] += seconds
        huw.watch(lambda _: {'state': next(values)}, A, directory,
                  clock=lambda: clock[0], sleep=sleep)
        runner = Mock(return_value=subprocess.CompletedProcess([], 0, LIVE, ''))
        huw.relay(directory, 'hc', 't1', expected_app='copilot', runner=runner)
        with huw.lock(directory):
            for path in directory.iterdir():
                self.assertEqual(stat.S_IMODE(path.stat().st_mode), 0o600, path.name)
        self.assertEqual(stat.S_IMODE(directory.stat().st_mode), 0o700)
        self.assertEqual(stat.S_IMODE(directory.parent.stat().st_mode), 0o700)

    def test_existing_broad_directory_rejected_without_chmod(self):
        directory = self.root / 'existing'
        directory.mkdir(mode=0o755)
        with self.assertRaises(huw.GuardError):
            huw.atomic_json(directory / 'state.json', {'test': True})
        self.assertEqual(stat.S_IMODE(directory.stat().st_mode), 0o755)
        self.assertFalse((directory / 'state.json').exists())

    def test_atomic_replacement_and_journal_tighten_only_target_files(self):
        target = self.root / 'event.json'
        journal = self.root / 'transitions.jsonl'
        unrelated = self.root / 'unrelated.txt'
        for path in (target, journal, unrelated):
            path.write_text('{}\n')
            path.chmod(0o644)
        huw.atomic_json(target, {'updated': True})
        huw.append_transition(journal, {'updated': True})
        self.assertEqual(stat.S_IMODE(target.stat().st_mode), 0o600)
        self.assertEqual(stat.S_IMODE(journal.stat().st_mode), 0o600)
        self.assertEqual(stat.S_IMODE(unrelated.stat().st_mode), 0o644)
        self.assertEqual(json.loads(target.read_text()), {'updated': True})

    def test_symlink_directory_and_journal_rejected(self):
        link = self.root / 'link'
        link.symlink_to(self.root, target_is_directory=True)
        with self.assertRaises(huw.GuardError): huw.private_directory(link)
        target = self.root / 'original.txt'
        target.write_text('unchanged')
        journal = self.root / 'transitions.jsonl'
        journal.symlink_to(target)
        with self.assertRaises(huw.GuardError): huw.append_transition(journal, {})
        self.assertEqual(target.read_text(), 'unchanged')

    def test_temp_file_permissions_and_cleanup_on_write_failure(self):
        target = self.root / 'event.json'
        target.write_text('old')
        with self.assertRaises(TypeError):
            huw.atomic_json(target, {'not_json': object()})
        self.assertEqual(target.read_text(), 'old')
        self.assertEqual(list(self.root.glob('*.tmp')), [])


if __name__ == '__main__': unittest.main()
