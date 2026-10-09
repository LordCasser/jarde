#!/usr/bin/env python3
"""冻结期间串行执行，保存原始结果；失败即停止，不把未执行门禁当作成功。"""
from pathlib import Path
import hashlib
import json
import os
import shutil
import subprocess
import time

REPO = Path('/Users/lordcasser/workspace/projects/jarde')
OUT = Path(__file__).resolve().parent / 'focused-v5'
OUT.mkdir()
ENV = dict(os.environ, CARGO_BUILD_JOBS='1', CARGO_INCREMENTAL='0', RUST_TEST_THREADS='1',
           CARGO_PROFILE_DEV_DEBUG='0', CARGO_PROFILE_TEST_DEBUG='0')
recorded_env = {k: ENV[k] for k in ['CARGO_BUILD_JOBS', 'CARGO_INCREMENTAL', 'RUST_TEST_THREADS',
                                 'CARGO_PROFILE_DEV_DEBUG', 'CARGO_PROFILE_TEST_DEBUG']}
commands = [
    ('fmt', ['cargo', 'fmt', '--all']),
    ('init-focused', ['cargo', 'test', '-p', 'jarde-java', '--lib', 'init::tests', '--locked', '--', '--nocapture']),
    ('complete-family-integration', ['cargo', 'test', '--test', 'p3_constructed_reference_array_elements', '--locked', '--', '--nocapture']),
    ('cli-build', ['cargo', 'build', '-p', 'jarde-cli', '--locked']),
]
meta = {'environment': recorded_env, 'runner_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), 'stages': []}
for name, argv in commands:
    free = shutil.disk_usage(REPO).free
    if free < 20 * 1024 ** 3:
        meta['stopped_before_stage'] = name
        meta['disk_free_bytes'] = free
        (OUT / 'index.json').write_text(json.dumps(meta, indent=2) + '\n')
        raise SystemExit('可用空间低于20GiB停建线')
    start = time.monotonic()
    with (OUT / (name + '.stdout')).open('wb') as stdout, (OUT / (name + '.stderr')).open('wb') as stderr:
        result = subprocess.run(argv, cwd=REPO, env=ENV, stdout=stdout, stderr=stderr)
    row = {'name': name, 'argv': argv, 'cwd': str(REPO), 'exit': result.returncode,
           'seconds': time.monotonic() - start, 'free_bytes_after': shutil.disk_usage(REPO).free}
    for stream in ['stdout', 'stderr']:
        path = OUT / (name + '.' + stream)
        row[stream] = path.name
        row[stream + '_sha256'] = hashlib.sha256(path.read_bytes()).hexdigest()
    meta['stages'].append(row)
    (OUT / 'index.json').write_text(json.dumps(meta, indent=2) + '\n')
    print(name, result.returncode, flush=True)
    if result.returncode:
        raise SystemExit(result.returncode)
