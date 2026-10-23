#!/usr/bin/env python3
"""Inventory locked dependencies and collect available notices without changing licenses.

Run after `pnpm install --frozen-lockfile`. Registry metadata is used only for
locked npm packages absent on this platform. No package code is executed.
"""
import argparse
import concurrent.futures
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tomllib
import urllib.parse
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
PREFIXES = ('license', 'licence', 'notice', 'copying', 'copyright', 'unlicense')


def npm_keys(text):
    if not re.search(r"^lockfileVersion: ['\"]?9\.0", text):
        raise ValueError('Only pnpm lockfile v9 is supported')
    section = text.split('\npackages:\n', 1)[1].split('\nsnapshots:\n', 1)[0]
    keys = [key.strip("'\"") for key in re.findall(r'^  (\S.*):$', section, re.M)]
    result = [key.rsplit('@', 1) for key in keys]
    if not result or any(not n or not re.fullmatch(r'\d[^/]*', v) for n, v in result):
        raise ValueError('Unexpected package key; update parser before auditing')
    return result


def digest(data):
    return hashlib.sha256(data).hexdigest()


def notice_files(directory):
    # Collect nested vendored notices too, but never traverse dependency links.
    import os
    result = []
    for base, dirs, files in os.walk(directory, followlinks=False):
        dirs[:] = sorted(d for d in dirs if d not in {'node_modules', '.git', 'target'}
                         and not (Path(base) / d).is_symlink())
        for name in sorted(files):
            path = Path(base) / name
            if path.is_symlink():
                continue
            named_notice = name.lower().startswith(PREFIXES)
            license_directory = any(part.lower() in {'licenses', 'licences'}
                                    for part in path.relative_to(directory).parts[:-1])
            authors_license = name.lower().startswith('authors') and bool(re.search(
                r'Permission is hereby granted|Licensed under|SPDX-License-Identifier',
                path.read_text(encoding='utf-8', errors='replace'), re.IGNORECASE))
            if named_notice or license_directory or authors_license:
                result.append(path)
    return result


def registry_metadata(pair):
    name, version = pair
    url = 'https://registry.npmjs.org/' + urllib.parse.quote(name, safe='') + '/' + urllib.parse.quote(version, safe='')
    try:
        with urllib.request.urlopen(url, timeout=30) as response:
            data = json.load(response)
        if (data.get('name'), data.get('version')) != (name, version):
            raise ValueError('Registry identity mismatch')
        return pair, data, url, None
    except Exception as error:
        return pair, {}, url, type(error).__name__


def validate_cargo_metadata(metadata, lockfile):
    def identities(packages):
        return {(p['name'], p['version'], p['source']) for p in packages if p.get('source')}
    actual = identities(metadata['packages'])
    expected = identities(lockfile.get('package', []))
    if actual != expected:
        raise ValueError('Cargo metadata does not match Cargo.lock; regenerate without a target filter')
    if any(source != 'registry+https://github.com/rust-lang/crates.io-index' for _, _, source in actual):
        raise ValueError('Non-crates.io sources need explicit source URL handling before auditing')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cargo-metadata', type=Path, help='Previously generated cargo metadata JSON')
    parser.add_argument('--check', action='store_true', help='Check committed inventory input hashes without network access')
    parser.add_argument('--reuse-npm-metadata', action='store_true', help='Reuse exact-lock npm metadata from the existing inventory')
    parser.add_argument('--offline', action='store_true', help='Do not query npm metadata for missing packages')
    args = parser.parse_args()
    if args.check:
        report = json.loads((ROOT / 'docs/legal/dependency-licenses.json').read_text())
        for path, expected in report['lockfiles'].items():
            if digest((ROOT / path).read_bytes()) != expected:
                raise SystemExit(f'License inventory stale: {path}; regenerate and review notices')
        notice_path = ROOT / 'THIRD_PARTY_NOTICES.txt'
        if digest(notice_path.read_bytes()) != report['notices_sha256']:
            raise SystemExit('THIRD_PARTY_NOTICES.txt differs from the generated inventory')
        notices = notice_path.read_text()
        for package in report['packages']:
            for notice in package['notice_files']:
                if 'SHA256 ' + notice['sha256'] not in notices:
                    raise SystemExit('Collected notice missing from THIRD_PARTY_NOTICES.txt')
        print('License inventory input hashes and notice references match (not legal clearance).')
        return
    if args.cargo_metadata:
        metadata = json.loads(args.cargo_metadata.read_text())
    else:
        metadata = json.loads(subprocess.check_output([
            'cargo', 'metadata', '--locked', '--format-version', '1',
            '--manifest-path', str(ROOT / 'src-tauri/Cargo.toml')], cwd=ROOT))
    with (ROOT / "src-tauri/Cargo.lock").open("rb") as lockfile:
        validate_cargo_metadata(metadata, tomllib.load(lockfile))
    installed = {}
    store = ROOT / 'node_modules/.pnpm'
    for pattern in ('*/node_modules/*/package.json', '*/node_modules/@*/*/package.json'):
        for path in sorted(store.glob(pattern)):
            data = json.loads(path.read_text())
            installed[(data.get('name'), data.get('version'))] = (data, path.parent)
    locked = [tuple(pair) for pair in npm_keys((ROOT / 'pnpm-lock.yaml').read_text())]
    missing = [pair for pair in locked if pair not in installed]
    fetched = {}
    if args.reuse_npm_metadata:
        previous = json.loads((ROOT / 'docs/legal/dependency-licenses.json').read_text(encoding='utf-8'))
        if previous['lockfiles']['pnpm-lock.yaml'] != digest((ROOT / 'pnpm-lock.yaml').read_bytes()):
            raise SystemExit('Cannot reuse npm metadata after a lockfile change')
        for record in previous['packages']:
            pair = (record['name'], record['version'])
            if record['ecosystem'] == 'npm' and pair in missing and record['license']:
                fetched[pair] = ({'license': record['license']}, record['source'], None)
    if not args.offline:
        with concurrent.futures.ThreadPoolExecutor(max_workers=8) as pool:
            for pair, data, url, error in pool.map(registry_metadata, [pair for pair in missing if pair not in fetched]):
                fetched[pair] = (data, url, error)
    records, texts = [], {}
    supplement_path = ROOT / 'docs/legal/supplemental-notices.json'
    supplements = json.loads(supplement_path.read_text()) if supplement_path.exists() else []

    def add(ecosystem, name, version, license_expression, directory, source, error=None):
        notices = []
        if directory:
            for path in notice_files(directory):
                raw = path.read_bytes()
                key = digest(raw)
                try:
                    text = raw.decode('utf-8')
                except UnicodeDecodeError:
                    continue
                texts[key] = text
                notices.append({'path': path.relative_to(directory).as_posix(), 'sha256': key})
        package_id = f'{ecosystem}:{name}@{version}'
        for supplement in supplements:
            if package_id in supplement['packages']:
                key = digest(supplement['text'].encode('utf-8'))
                texts[key] = supplement['text']
                notices.append({'path': 'upstream/' + supplement['source'].rsplit('/', 1)[-1],
                                'source': supplement['source'], 'sha256': key})
        records.append({'ecosystem': ecosystem, 'name': name, 'version': version,
                        'license': license_expression, 'source': source,
                        'notice_files': notices, 'metadata_error': error,
                        'review_required': not license_expression or not notices})

    for name, version in sorted(locked):
        if (name, version) in installed:
            data, directory = installed[(name, version)]
            source = f'https://registry.npmjs.org/{urllib.parse.quote(name, safe="")}/{version}'
            error = None
        else:
            data, source, error = fetched.get((name, version), ({}, None, 'not-installed-offline'))
            directory = None
        add('npm', name, version, data.get('license'), directory, source, error)
    for package in sorted(metadata['packages'], key=lambda p: (p['name'], p['version'])):
        if not package['source']:
            continue
        add('cargo', package['name'], package['version'], package['license'],
            Path(package['manifest_path']).parent,
            f"https://crates.io/api/v1/crates/{package['name']}/{package['version']}/download")
    missing_metadata = [r for r in records if not r['license']]
    if missing_metadata:
        raise SystemExit('Missing license metadata; existing reports were not overwritten: ' +
                         ', '.join(r['name'] + '@' + r['version'] for r in missing_metadata))
    asset_notice_path = ROOT / 'docs/legal/asset-license-notices.json'
    asset_notices = json.loads(asset_notice_path.read_text(encoding='utf-8'))
    for notice in asset_notices:
        notice['sha256'] = digest(notice['text'].encode('utf-8'))
        texts[notice['sha256']] = notice['text']
    output = ROOT / 'docs/legal'
    output.mkdir(parents=True, exist_ok=True)
    report = {'scope': 'All pnpm v9 package entries and cargo metadata resolved packages, including dev/build/optional/platform dependencies; not a binary SBOM.',
              'lockfiles': {p: digest((ROOT / p).read_bytes()) for p in ['pnpm-lock.yaml', 'src-tauri/Cargo.lock', 'LICENSE', 'docs/legal/supplemental-notices.json', 'docs/legal/asset-license-notices.json', 'docs/legal/asset-provenance.json']},
              'packages': records,
              'asset_notices': [{k: v for k, v in n.items() if k != 'text'} for n in asset_notices]}
    out = ['THIRD-PARTY NOTICES — generated by scripts/audit-third-party-licenses.py',
           'See docs/legal/third-party-license-audit.md for coverage and unresolved items.',
           'Original CC-Gateway-Pro / cc-switch license:\n' + (ROOT / 'LICENSE').read_text(),
           'Package index (source URLs also locate unchanged MPL-covered source):']
    for r in records:
        out.append(f"{r['ecosystem']}:{r['name']}@{r['version']} | {r['license']} | {r['source']}")
        out.extend(f"  {n['path']} => SHA256 {n['sha256']}" for n in r['notice_files'])
        if not r['notice_files']:
            out.append('  NOTICE TEXT NOT COLLECTED: review required; metadata is not a substitute.')
    out.append('\nAsset library notices (source identity and rights limits: docs/legal/asset-provenance.json):')
    for notice in asset_notices:
        out.append(f"{notice['name']}@{notice['version']} | {notice['license']} | {notice['source']} | SHA256 {notice['sha256']}")
    for key, text in sorted(texts.items()):
        out.extend(['\n' + '=' * 72, 'SHA256 ' + key, text])
    (ROOT / 'THIRD_PARTY_NOTICES.txt').write_text('\n'.join(out) + '\n')
    report['notices_sha256'] = digest((ROOT / 'THIRD_PARTY_NOTICES.txt').read_bytes())
    (output / 'dependency-licenses.json').write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n')
    print(json.dumps({'packages': len(records), 'npm': len(locked), 'cargo': len(records)-len(locked),
                      'missing_metadata': sum(not r['license'] for r in records),
                      'without_notice_text': sum(not r['notice_files'] for r in records),
                      'unique_notice_texts': len(texts)}, indent=2))


if __name__ == '__main__':
    main()
