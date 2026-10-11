import datetime
import hashlib
import json
import shutil
import stat
import subprocess
import sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
CLI = Path('/private/tmp/jarde-computed-for-final-cli-root-v2')
CLI_SHA256 = 'd1543cb69867d78f964873fe0e3d534f18923d3350d6dcf88b6a7e02738b56d1'
CLI_MODE = 0o555

if len(sys.argv) != 3:
    raise SystemExit('usage: audit.py OUTPUT_DIR EXPECTED_HEAD')
OUT = Path(sys.argv[1])
EXPECTED_HEAD = sys.argv[2]
if len(EXPECTED_HEAD) != 40 or any(ch not in '0123456789abcdef' for ch in EXPECTED_HEAD):
    raise SystemExit('EXPECTED_HEAD must be a full lowercase 40-character commit id')
OUT.mkdir(parents=True, exist_ok=False)
rows = []

def command(argv, cwd=ROOT, allowed=(0,)):
    proc = subprocess.run(argv, cwd=cwd, capture_output=True)
    index = len(rows)
    streams = {}
    for name, raw in [('stdout', proc.stdout), ('stderr', proc.stderr)]:
        filename = f'{index}.{name}.raw'
        (OUT / filename).write_bytes(raw)
        streams[name] = {'path': filename, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
    rows.append({'argv': argv, 'cwd': str(cwd), 'exit_code': proc.returncode, 'streams': streams})
    if proc.returncode not in allowed:
        raise RuntimeError(f'command {index} failed: {argv}')
    return proc.stdout.decode().strip()

result = {
    'schema': 'jarde-computed-for-clean-main-audit-root-v1',
    'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'expected_head': EXPECTED_HEAD,
    'commands': rows,
    'audit_script_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
}
try:
    head = command(['git', 'rev-parse', 'HEAD'])
    main = command(['git', 'rev-parse', 'main'])
    origin_main = command(['git', 'rev-parse', 'origin/main'])
    branch = command(['git', 'branch', '--show-current'])
    branches = command(['git', 'for-each-ref', '--format=%(refname:short)', 'refs/heads']).splitlines()
    origin_url = command(['git', 'remote', 'get-url', 'origin'])
    remote_heads = command(['git', 'ls-remote', '--heads', 'origin']).splitlines()
    worktrees_raw = command(['git', 'worktree', 'list', '--porcelain'])
    worktrees = []
    for block in worktrees_raw.split('\n\n'):
        if not block.strip():
            continue
        fields = {}
        for line in block.splitlines():
            key, sep, value = line.partition(' ')
            fields[key] = value if sep else True
        path = Path(fields['worktree'])
        status = command(['git', 'status', '--ignored', '--porcelain=v1', '--untracked-files=all'], path)
        status_lines = status.splitlines()
        ignored = [line for line in status_lines if line.startswith('!!')]
        changes = [line for line in status_lines if not line.startswith('!!')]
        command(['git', 'merge-base', '--is-ancestor', fields['HEAD'], 'main'], allowed=(0, 1))
        ancestor = rows[-1]['exit_code'] == 0
        worktrees.append({
            'path': str(path),
            'head': fields['HEAD'],
            'branch': fields.get('branch'),
            'detached': bool(fields.get('detached')),
            'locked': fields.get('locked') if 'locked' in fields else None,
            'prunable': fields.get('prunable') if 'prunable' in fields else None,
            'clean': not changes,
            'status_change_count': len(changes),
            'ignored_entry_count': len(ignored),
            'ignored_status_sha256': hashlib.sha256('\n'.join(ignored).encode()).hexdigest(),
            'main_ancestor': ancestor,
            'target_exists': (path / 'target').exists(),
            'fuzz_target_exists': (path / 'fuzz' / 'target').exists(),
        })
    cli_sha = hashlib.sha256(CLI.read_bytes()).hexdigest()
    cli_mode = stat.S_IMODE(CLI.stat().st_mode)
    free_bytes = shutil.disk_usage(ROOT).free
    result.update(
        head=head,
        main=main,
        origin_main=origin_main,
        branch=branch,
        local_branches=branches,
        origin_url=origin_url,
        remote_heads=remote_heads,
        worktree_count=len(worktrees),
        worktrees=worktrees,
        free_bytes=free_bytes,
        candidate_cli={'path': str(CLI), 'sha256': cli_sha, 'mode': oct(cli_mode)},
    )
    assert head == EXPECTED_HEAD, (head, EXPECTED_HEAD)
    assert head == main == origin_main and branch == 'main' and branches == ['main']
    assert origin_url and remote_heads == [head + '\trefs/heads/main'], remote_heads
    assert worktrees and any(w['path'] == str(ROOT) and w['branch'] == 'refs/heads/main' for w in worktrees)
    assert all(w['clean'] and w['main_ancestor'] and not w['locked'] and not w['prunable']
               and not w['target_exists'] and not w['fuzz_target_exists'] for w in worktrees)
    assert all(w['detached'] and w['branch'] is None for w in worktrees if w['path'] != str(ROOT))
    assert free_bytes >= 5 * 1024**3
    assert cli_sha == CLI_SHA256 and cli_mode == CLI_MODE
    result['status'] = 'accepted_clean_main'
except Exception as exc:
    result.update(status='failed', error=f'{type(exc).__name__}: {exc}')
finally:
    (OUT / 'execution.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({k: result.get(k) for k in ['status', 'expected_head', 'head', 'worktree_count', 'free_bytes', 'error']}))
sys.exit(0 if result['status'] == 'accepted_clean_main' else 1)
