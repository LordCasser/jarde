#!/usr/bin/env python3
import datetime
import hashlib
import json
import os
import shlex
import shutil
import subprocess
import sys
from pathlib import Path

root = Path(__file__).resolve().parents[4]
out = Path(__file__).resolve().parent / sys.argv[1]
out.mkdir()
env = dict(os.environ, CARGO_BUILD_JOBS='1', CARGO_INCREMENTAL='0', RUST_TEST_THREADS='1')
ci = (root / '.github/workflows/ci.yml').read_text()
lines = ci.splitlines()
begin = next(i for i, line in enumerate(lines) if line.strip().startswith('cargo clippy '))
end = next(i for i in range(begin, len(lines)) if lines[i].strip() == '-D warnings')
clippy = shlex.split(' '.join(line.strip() for line in lines[begin:end + 1]))
stages = [
    ('focused-varargs', ['cargo', 'test', '-p', 'jarde-java', '--test', 'varargs_ctor_argument_sites', '--locked'], {}),
    ('workspace-seed1', ['cargo', 'test', '--workspace', '--all-targets', '--all-features', '--locked', '--no-fail-fast'], {'PROPTEST_RNG_SEED': '5350648285461741569'}),
    ('workspace-seed2', ['cargo', 'test', '--workspace', '--all-targets', '--all-features', '--locked', '--no-fail-fast'], {'PROPTEST_RNG_SEED': '5350648285461741570'}),
    ('msrv-188', ['rustup', 'run', '1.88.0', 'cargo', 'check', '--workspace', '--all-targets', '--locked'], {}),
    ('fmt', ['cargo', 'fmt', '--all', '--', '--check'], {}),
    ('ci-clippy', clippy, {}),
    ('ignored-p3', ['cargo', 'test', '--test', 'p3_execution_comparison', '--locked', '--', '--ignored'], {}),
    ('ignored-constructor', ['cargo', 'test', '-p', 'jarde-cli', '--test', 'json_cli', '--locked', '--', '--ignored', '--exact', 'functional_constructor_arguments_replay_the_complete_class_on_both_jdks'], {}),
    ('strict-openspec', ['openspec', 'validate', '--all', '--strict', '--no-interactive'], {}),
    ('diff-check', ['git', 'diff', '--check'], {}),
]
records = []
for name, argv, extra in stages:
    free = shutil.disk_usage(root).free
    if free < 20 * 2**30:
        raise SystemExit('Disk free below 20 GiB before ' + name)
    print('START', name, 'free_GiB', free / 2**30, flush=True)
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    stdout = out / (name + '.stdout')
    stderr = out / (name + '.stderr')
    with stdout.open('wb') as so, stderr.open('wb') as se:
        result = subprocess.run(argv, cwd=root, env=dict(env, **extra), stdout=so, stderr=se)
    records.append({'name': name, 'argv': argv, 'cwd': str(root), 'env': {k: dict(env, **extra)[k] for k in ('CARGO_BUILD_JOBS', 'CARGO_INCREMENTAL', 'RUST_TEST_THREADS', *extra)},
                    'started_utc': started, 'exit': result.returncode,
                    'stdout': stdout.name, 'stdout_sha256': hashlib.sha256(stdout.read_bytes()).hexdigest(),
                    'stderr': stderr.name, 'stderr_sha256': hashlib.sha256(stderr.read_bytes()).hexdigest(),
                    'free_bytes_after': shutil.disk_usage(root).free})
    (out / 'index.json').write_text(json.dumps({'runner_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                                            'ci_yml_sha256': hashlib.sha256(ci.encode()).hexdigest(), 'stages': records}, indent=2) + '\n')
    print('DONE', name, 'exit', result.returncode, flush=True)
    if result.returncode:
        raise SystemExit(result.returncode)
