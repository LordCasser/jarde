#!/usr/bin/env python3
"""Build the fixed S0 CLI and existing measurement harness; remove only this target directory."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import time

p = argparse.ArgumentParser()
p.add_argument('--checkout', type=Path, required=True)
p.add_argument('--out', type=Path, required=True)
p.add_argument('--target', type=Path, required=True)
p.add_argument('--bin-dir', type=Path, required=True)
p.add_argument('--resource-source', type=Path)
p.add_argument('--resources-only', action='store_true')
p.add_argument('--negative-only', action='store_true')
p.add_argument('--lifetime-only', action='store_true')
a = p.parse_args()
assert not a.target.exists(), a.target
a.out.mkdir(parents=True, exist_ok=True)
a.bin_dir.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, CARGO_TARGET_DIR=str(a.target), CARGO_BUILD_JOBS='1',
           CARGO_INCREMENTAL='0', CARGO_PROFILE_RELEASE_DEBUG='0',
           CARGO_PROFILE_RELEASE_STRIP='none')
env.pop('RUST_MIN_STACK', None)
records = []
resource_target = a.checkout / 'tests/s0_resources.rs'
if a.resource_source:
    assert not resource_target.exists(), resource_target
    shutil.copy2(a.resource_source, resource_target)
try:
    commands = [
        ('cli', ['cargo', 'build', '--release', '--locked', '--offline', '-p', 'jarde-cli']),
        ('workloads', ['cargo', 'test', '--release', '--locked', '--offline',
                       '--test', 'p5_optimize_workloads', '--no-run', '--message-format=json']),
    ]
    if a.resources_only:
        assert a.resource_source
        commands = [('normal-check', ['cargo','check','--release','--locked','--offline','-p','jarde-cli']),
            ('support-check', ['cargo','check','--release','--locked','--offline','--features','test-support','--test','s0_resources','--test','s0_observation']),
            ('resources', ['cargo', 'test', '--release', '--locked', '--offline', '--features', 'test-support', '--test', 's0_resources', '--test', 's0_observation', '--no-run', '--message-format=json']),
            ('java-unit', ['cargo','test','--release','--locked','--offline','-p','jarde-java','--lib','--no-run','--message-format=json']),
            ('root-unit', ['cargo','test','--release','--locked','--offline','--features','test-support','--lib','--no-run','--message-format=json'])]
    if a.lifetime_only:
        commands = [('lifetime', ['cargo','test','--release','--locked','--offline','--features','test-support','--test','s0_observation','--no-run','--message-format=json'])]
    if a.negative_only:
        commands = [('negative-api', ['cargo','test','--release','--locked','--offline','--features','test-support','--test','class_source','--test','member_family_external_use','--test','recover_postfix_old_value_snapshot','--test','recover_static_generic_field_init_text','--no-run','--message-format=json']),
            ('negative-java', ['cargo','test','--release','--locked','--offline','-p','jarde-java','--lib','--no-run','--message-format=json'])]
    for name, command in commands:
        started = time.monotonic()
        with (a.out / (name + '.stdout')).open('wb') as stdout, (a.out / (name + '.stderr')).open('wb') as stderr:
            child = subprocess.Popen(command, cwd=a.checkout, env=env, stdout=stdout,
                                     stderr=stderr, start_new_session=True)
            stop = None
            peak_target = 0
            while child.poll() is None:
                size = 0
                for f in a.target.rglob('*'):
                    try:
                        if f.is_file():
                            size += f.stat().st_size
                    except FileNotFoundError:
                        pass  # rustc can rename/remove a temporary between directory and stat.
                peak_target = max(peak_target, size)
                if size > 1 << 30 or shutil.disk_usage(a.target.parent).free < 5 << 30:
                    stop = 'target_or_free_space_guard'
                if time.monotonic() - started > 900:
                    stop = 'build_timeout'
                if stop:
                    os.killpg(child.pid, signal.SIGTERM)
                    child.wait(timeout=15)
                    break
                time.sleep(1)
            rc = child.wait()
        records.append(dict(name=name, command=command, exit_code=rc, stop=stop,
                            seconds=time.monotonic()-started, peak_target_bytes=peak_target))
        assert rc == 0 and stop is None, records[-1]
    if not a.resources_only and not a.negative_only and not a.lifetime_only:
        shutil.copy2(a.target / 'release/jarde-cli', a.bin_dir / 'jarde-cli')
    if a.negative_only:
        candidates = []
        for name, _ in commands:
            for line in (a.out / (name+'.stdout')).read_text().splitlines():
                item=json.loads(line)
                if item.get('executable'):
                    candidates.append((item['target']['name'],Path(item['executable'])))
        assert len(candidates)==5,candidates
        for name,binary in candidates:
            shutil.copy2(binary,a.bin_dir/name)
    else:
        candidates = []
        target_name = 's0_observation' if a.lifetime_only else ('s0_resources' if a.resources_only else 'p5_optimize_workloads')
        for line in (a.out / ('lifetime.stdout' if a.lifetime_only else ('resources.stdout' if a.resources_only else 'workloads.stdout'))).read_text().splitlines():
            item = json.loads(line)
            if item.get('target', {}).get('name') == target_name and item.get('executable'):
                candidates.append(Path(item['executable']))
        assert len(candidates) == 1, candidates
        shutil.copy2(candidates[0], a.bin_dir / target_name)
        if a.resources_only:
            for output in ['resources.stdout','java-unit.stdout','root-unit.stdout']:
                for line in (a.out/output).read_text().splitlines():
                    item=json.loads(line)
                    if item.get('executable') and item['target']['name'] in ['s0_observation','jarde_java','jarde']:
                        shutil.copy2(item['executable'],a.bin_dir/item['target']['name'])

finally:
    if 'child' in globals() and child.poll() is None:
        os.killpg(child.pid, signal.SIGTERM)
        child.wait(timeout=15)
    shutil.rmtree(a.target, ignore_errors=True)
    if a.resource_source:
        resource_target.unlink(missing_ok=True)
    receipt = dict(checkout=str(a.checkout), target=str(a.target), commands=records,
                   target_absent=not a.target.exists(), binaries=[])
    for binary in a.bin_dir.glob('*'):
        if binary.is_file():
            receipt['binaries'].append(dict(path=str(binary), bytes=binary.stat().st_size,
                                           sha256=hashlib.sha256(binary.read_bytes()).hexdigest()))
    receipt['git_head'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=a.checkout, text=True).strip()
    receipt['resource_source'] = str(a.resource_source) if a.resource_source else None
    receipt['rustc'] = subprocess.check_output(['rustc', '-Vv'], text=True)
    (a.out / 'build-receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
print(json.dumps(receipt, indent=2))
