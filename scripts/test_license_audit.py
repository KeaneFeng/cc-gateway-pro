import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import json

spec = importlib.util.spec_from_file_location('audit', Path(__file__).with_name('audit-third-party-licenses.py'))
audit = importlib.util.module_from_spec(spec)
spec.loader.exec_module(audit)


class LicenseAuditTests(unittest.TestCase):
    def test_lockfile_keys_exclude_nested_peer_fields_and_snapshots(self):
        text = """lockfileVersion: '9.0'
packages:
  '@scope/pkg@1.2.3':
    peerDependencies:
      react: '*'
  simple@2.0.0:
    resolution: {integrity: sha512-test}
snapshots:
  unrelated@3.0.0: {}
"""
        self.assertEqual(audit.npm_keys(text), [['@scope/pkg', '1.2.3'], ['simple', '2.0.0']])

    def test_unknown_lock_format_fails_closed(self):
        with self.assertRaises(ValueError):
            audit.npm_keys("lockfileVersion: '8.0'\npackages:\n  example@1.0.0:\n")

    def test_collect_nested_vendor_notices_without_following_links(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'vendor').mkdir()
            (root / 'vendor/LICENSE-MIT').write_text('vendor copyright')
            (root / 'LICENSE').write_text('root copyright')
            (root / 'linked').symlink_to(root / 'vendor', target_is_directory=True)
            (root / 'node_modules').mkdir()
            (root / 'node_modules/LICENSE').write_text('not part of this package')
            self.assertEqual({p.relative_to(root).as_posix() for p in audit.notice_files(root)},
                             {'LICENSE', 'vendor/LICENSE-MIT'})

    def test_collect_authors_license_and_spdx_license_directory(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'AUTHORS').write_text('LICENSE: MIT\nPermission is hereby granted to any person...')
            (root / 'AUTHORS.md').write_text('Alice and Bob')
            (root / 'LICENSES').mkdir()
            (root / 'LICENSES/MIT.txt').write_text('Permission is hereby granted...')
            self.assertEqual({p.relative_to(root).as_posix() for p in audit.notice_files(root)},
                             {'AUTHORS', 'LICENSES/MIT.txt'})

    def test_missing_metadata_does_not_overwrite_existing_inventory(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'docs/legal').mkdir(parents=True)
            report = root / 'docs/legal/dependency-licenses.json'
            report.write_text('previous reviewed inventory')
            (root / 'pnpm-lock.yaml').write_text("lockfileVersion: '9.0'\npackages:\n  absent@1.0.0:\nsnapshots:\n")
            metadata = root / 'metadata.json'
            metadata.write_text('{"packages": []}')
            (root / 'src-tauri').mkdir()
            (root / 'src-tauri/Cargo.lock').write_text('version = 3\n')
            with patch.object(audit, 'ROOT', root), patch('sys.argv', ['audit', '--offline', '--cargo-metadata', str(metadata)]):
                with self.assertRaisesRegex(SystemExit, 'not overwritten'):
                    audit.main()
            self.assertEqual(report.read_text(), 'previous reviewed inventory')

    def test_cached_metadata_rejected_after_lockfile_change(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'docs/legal').mkdir(parents=True)
            (root / 'docs/legal/dependency-licenses.json').write_text(json.dumps({'lockfiles': {'pnpm-lock.yaml': 'old'}}))
            (root / 'pnpm-lock.yaml').write_text("lockfileVersion: '9.0'\npackages:\n  absent@1.0.0:\nsnapshots:\n")
            metadata = root / 'metadata.json'
            metadata.write_text('{"packages": []}')
            (root / 'src-tauri').mkdir()
            (root / 'src-tauri/Cargo.lock').write_text('version = 3\n')
            with patch.object(audit, 'ROOT', root), patch('sys.argv', ['audit', '--offline', '--reuse-npm-metadata', '--cargo-metadata', str(metadata)]):
                with self.assertRaisesRegex(SystemExit, 'lockfile change'):
                    audit.main()

    def test_cargo_metadata_must_match_locked_package_identities(self):
        locked = {'package': [{'name': 'sample', 'version': '2.0.0', 'source': 'registry+https://github.com/rust-lang/crates.io-index'}]}
        metadata = {'packages': [{'name': 'sample', 'version': '1.0.0', 'source': 'registry+https://github.com/rust-lang/crates.io-index'}]}
        with self.assertRaisesRegex(ValueError, 'Cargo metadata'):
            audit.validate_cargo_metadata(metadata, locked)

    def test_freshness_check_rejects_changed_inputs_and_notice_text(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / 'docs/legal').mkdir(parents=True)
            (root / 'LICENSE').write_text('original')
            notices = root / 'THIRD_PARTY_NOTICES.txt'
            notices.write_text('notice text')
            report = {'lockfiles': {'LICENSE': audit.digest(b'original')},
                      'packages': [], 'notices_sha256': audit.digest(b'notice text')}
            (root / 'docs/legal/dependency-licenses.json').write_text(json.dumps(report))
            with patch.object(audit, 'ROOT', root), patch('sys.argv', ['audit', '--check']):
                audit.main()
                (root / 'LICENSE').write_text('changed')
                with self.assertRaisesRegex(SystemExit, 'inventory stale'):
                    audit.main()
                (root / 'LICENSE').write_text('original')
                notices.write_text('changed')
                with self.assertRaisesRegex(SystemExit, 'differs'):
                    audit.main()


if __name__ == '__main__':
    unittest.main()
