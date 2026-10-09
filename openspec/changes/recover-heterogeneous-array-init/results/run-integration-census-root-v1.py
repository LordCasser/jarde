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
    ('focused-integration-v4', ['cargo', 'test', '--test', 'p3_heterogeneous_array_initializers', '--locked', '--', '--nocapture'], {}),
    ('reader-census-v1', ['cargo', 'test', '-p', 'jarde-reader', '--lib', 'repository_class_fixtures_validate_without_false_target_rejections', '--locked', '--', '--nocapture'], {}),

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
