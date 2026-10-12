#!/usr/bin/env python3
"""Read-only main/worktree/source-closure audit for argv-selected product receipts."""
from __future__ import annotations
import argparse, datetime, hashlib, json, os, re, subprocess, sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
EXPECTED_AUX_WORKTREES = 14

ap = argparse.ArgumentParser(description=__doc__)
ap.add_argument('output_dir', type=Path)
ap.add_argument('product_commit')
ap.add_argument('docs_head')
ap.add_argument('--focused-receipt', type=Path, required=True)
ap.add_argument('--focused-receipt-sha256', required=True)
ap.add_argument('--own-ci-acceptance', type=Path, required=True)
ap.add_argument('--own-ci-acceptance-sha256', required=True)
args = ap.parse_args()
OUT = args.output_dir.resolve()


def need(ok, message):
    if not ok:
        raise AssertionError(message)


def sha(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def sha_file(path: Path) -> str:
    return sha(path.read_bytes())


def git(argv, cwd=ROOT, allowed=(0,)):
    proc = subprocess.run(['git', *argv], cwd=cwd, capture_output=True)
    record = {'argv': ['git', *argv], 'cwd': str(cwd), 'exit_code': proc.returncode,
              'stdout_sha256': sha(proc.stdout), 'stdout_bytes': len(proc.stdout),
              'stderr_sha256': sha(proc.stderr), 'stderr_bytes': len(proc.stderr)}
    commands.append(record)
    if proc.returncode not in allowed:
        raise RuntimeError(f'git command failed: {argv!r} ({proc.returncode})')
    return proc.stdout


def tree(commit):
    raw = git(['ls-tree', '-r', '-z', commit])
    result = {}
    for record in raw.split(b'\0'):
        if not record:
            continue
        header, sep, path = record.partition(b'\t')
        need(bool(sep), 'malformed git ls-tree record')
        mode, kind, oid = header.split()
        need(kind == b'blob', 'pinned source path is not a blob')
        result[path] = (mode, oid)
    return result, sha(raw)


def blobs_sha256(commit, paths, tree_map):
    """Resolve pinned paths through one cat-file batch and hash blob contents."""
    oids = []
    for path in paths:
        need(path in tree_map, f'pinned source missing from {commit}: {os.fsdecode(path)}')
        oids.append(tree_map[path][1])
    proc = subprocess.run(['git', 'cat-file', '--batch'], cwd=ROOT,
                          input=b''.join(oid + b'\n' for oid in oids), capture_output=True)
    record = {'argv': ['git', 'cat-file', '--batch'], 'cwd': str(ROOT), 'exit_code': proc.returncode,
              'stdin_bytes': sum(len(oid) + 1 for oid in oids),
              'stdin_sha256': sha(b''.join(oid + b'\n' for oid in oids)),
              'stdout_sha256': sha(proc.stdout), 'stdout_bytes': len(proc.stdout),
              'stderr_sha256': sha(proc.stderr), 'stderr_bytes': len(proc.stderr)}
    commands.append(record)
    need(proc.returncode == 0, f'git cat-file failed for {commit}')
    raw, pos, values = proc.stdout, 0, {}
    for path, oid in zip(paths, oids):
        end = raw.find(b'\n', pos)
        need(end >= 0, 'truncated git cat-file header')
        fields = raw[pos:end].split()
        need(len(fields) == 3 and fields[0] == oid and fields[1] == b'blob', 'unexpected git cat-file header')
        size = int(fields[2]); start = end + 1; stop = start + size
        need(stop < len(raw) and raw[stop:stop+1] == b'\n', 'truncated git cat-file blob')
        values[path] = sha(raw[start:stop]); pos = stop + 1
    need(pos == len(raw), 'unexpected trailing git cat-file output')
    return values


def ancestor(left, right):
    row = git(['merge-base', '--is-ancestor', left, right], allowed=(0, 1))
    return commands[-1]['exit_code'] == 0


commands = []
result = {'schema': 'structured-list-clean-main-audit-v2', 'status': 'running',
          'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
          'product_commit': args.product_commit, 'expected_docs_head': args.docs_head,
          'focused_receipt': str(args.focused_receipt.resolve()),
          'focused_receipt_sha256': args.focused_receipt_sha256,
          'own_ci_acceptance': str(args.own_ci_acceptance.resolve()),
          'own_ci_acceptance_sha256': args.own_ci_acceptance_sha256, 'commands': commands}
try:
    need(re.fullmatch(r'[0-9a-f]{40}', args.product_commit) is not None, 'product commit must be lowercase full Git SHA')
    need(re.fullmatch(r'[0-9a-f]{40}', args.docs_head) is not None, 'docs head must be lowercase full Git SHA')
    for label, value in (('focused receipt', args.focused_receipt_sha256),
                         ('own-CI acceptance', args.own_ci_acceptance_sha256)):
        need(re.fullmatch(r'[0-9a-f]{64}', value) is not None, f'{label} SHA-256 must be lowercase full digest')
    need(not OUT.exists() and not OUT.is_relative_to(ROOT), 'audit output must be new and outside repository')
    need(not args.focused_receipt.is_symlink() and args.focused_receipt.is_file(), 'focused receipt missing or symlink')
    need(sha_file(args.focused_receipt) == args.focused_receipt_sha256, 'focused receipt SHA differs from argv pin')
    focused = json.loads(args.focused_receipt.read_text(encoding='utf-8'))
    need(focused.get('status') == 'commands_passed_pending_independent_acceptance',
         'focused receipt does not report completed command checks')
    before, after = focused.get('source_manifest_before'), focused.get('source_manifest_after')
    need(isinstance(before, list) and before == after and len(before) > 0,
         'focused source manifests are missing, changed, or empty')
    pins = {}
    for row in before:
        need(isinstance(row, dict) and isinstance(row.get('path'), str), 'malformed source manifest entry')
        rel = row['path']; digest = row.get('sha256')
        need(rel not in pins and re.fullmatch(r'[0-9a-f]{64}', digest or '') is not None, 'duplicate path or invalid source digest')
        pure = Path(rel)
        need(not pure.is_absolute() and '..' not in pure.parts and pure.parts, 'unsafe source manifest path')
        pins[rel.encode()] = digest
    manifest_sha = sha(json.dumps(before, sort_keys=True, separators=(',', ':')).encode())
    for count_key in ('source_manifest_before_count', 'source_manifest_after_count'):
        if count_key in focused:
            need(focused[count_key] == len(before), f'{count_key} differs from dynamically enumerated manifest')
    for digest_key in ('source_manifest_before_sha256', 'source_manifest_after_sha256'):
        if digest_key in focused:
            need(focused[digest_key] == manifest_sha, f'{digest_key} differs from dynamically hashed manifest')

    need(not args.own_ci_acceptance.is_symlink() and args.own_ci_acceptance.is_file(),
         'own-CI acceptance reference missing or symlink')
    need(sha_file(args.own_ci_acceptance) == args.own_ci_acceptance_sha256,
         'own-CI acceptance SHA differs from argv pin')
    own_ci = json.loads(args.own_ci_acceptance.read_text(encoding='utf-8'))
    need(own_ci.get('schema') == 'ordered-field-own-ci-acceptance-v1' and
         own_ci.get('status') == 'accepted_local_own_ci_observation',
         'own-CI acceptance reference is not an accepted ordered-field observation')
    need(own_ci.get('product_commit') == args.product_commit,
         'own-CI acceptance product commit differs from argv')
    focus_ref = own_ci.get('focused_receipt')
    need(isinstance(focus_ref, dict) and focus_ref.get('sha256') == args.focused_receipt_sha256 and
         Path(focus_ref.get('path', '')).resolve() == args.focused_receipt.resolve(),
         'own-CI acceptance does not bind the argv focused receipt path and SHA')
    closure = own_ci.get('source_closure')
    need(isinstance(closure, dict) and closure.get('focused_workspace_product_git_live_equal') is True and
         closure.get('count') == len(pins),
         'own-CI acceptance source closure does not bind this dynamic focused manifest')

    head = git(['rev-parse', 'HEAD']).decode().strip()
    main = git(['rev-parse', 'main']).decode().strip()
    origin_main = git(['rev-parse', 'origin/main']).decode().strip()
    branch = git(['branch', '--show-current']).decode().strip()
    local_branches = git(['for-each-ref', '--format=%(refname:short)', 'refs/heads']).decode().splitlines()
    origin_url = git(['remote', 'get-url', 'origin']).decode().strip()
    remote_heads = git(['ls-remote', '--heads', 'origin']).decode().splitlines()
    need(head == main == origin_main == args.docs_head and branch == 'main', 'HEAD/main/origin/main/docs head or branch differs')
    need(local_branches == ['main'], f'local branch set is not exactly main: {local_branches}')
    need(origin_url and remote_heads == [args.docs_head + '\trefs/heads/main'], 'origin main does not equal expected docs head')
    need(ancestor(args.product_commit, args.docs_head), 'product commit is not an ancestor of docs head')

    product_tree, product_tree_sha = tree(args.product_commit)
    docs_tree, docs_tree_sha = tree(args.docs_head)
    paths = sorted(pins)
    product_hashes = blobs_sha256(args.product_commit, paths, product_tree)
    docs_hashes = blobs_sha256(args.docs_head, paths, docs_tree)
    live_hashes = {}
    for encoded, expected in pins.items():
        rel = os.fsdecode(encoded); path = ROOT.joinpath(*Path(rel).parts)
        need(not path.is_symlink() and path.is_file(), f'live source missing or symlink: {rel}')
        actual = sha_file(path)
        row = next(item for item in before if item['path'] == rel)
        need(type(row.get('bytes')) is int and path.stat().st_size == row['bytes'],
             f'live source byte count differs from focused manifest: {rel}')
        need(actual == expected == product_hashes[encoded] == docs_hashes[encoded], f'source bytes differ across focus/product/docs/live: {rel}')
        need(product_tree[encoded][0] in (b'100644', b'100755') and product_tree[encoded][0] == docs_tree[encoded][0],
             f'source mode differs or is unsupported: {rel}')
        live_hashes[rel] = actual

    wt_text = git(['worktree', 'list', '--porcelain']).decode()
    worktrees = []
    for block in wt_text.strip().split('\n\n'):
        fields = {}
        for line in block.splitlines():
            key, sep, value = line.partition(' '); fields[key] = value if sep else True
        path = Path(fields['worktree']).resolve()
        status = git(['status', '--ignored', '--porcelain=v1', '--untracked-files=all'], cwd=path).decode().splitlines()
        ignored = [line for line in status if line.startswith('!!')]
        changes = [line for line in status if not line.startswith('!!')]
        wt_head = fields['HEAD']
        is_root = path == ROOT
        wt = {'path': str(path), 'head': wt_head, 'branch': fields.get('branch'),
              'detached': bool(fields.get('detached')), 'clean': not changes,
              'status_change_count': len(changes), 'ignored_entry_count': len(ignored),
              'target_exists': (path/'target').exists(), 'fuzz_target_exists': (path/'fuzz'/'target').exists(),
              'main_ancestor': ancestor(wt_head, 'main'), 'is_root': is_root}
        worktrees.append(wt)
    roots = [w for w in worktrees if w['is_root']]
    aux = [w for w in worktrees if not w['is_root']]
    need(len({w['path'] for w in worktrees}) == len(worktrees), 'duplicate physical worktree path')
    need(len(roots) == 1 and len(aux) == EXPECTED_AUX_WORKTREES and
         roots[0]['branch'] == 'refs/heads/main' and not roots[0]['detached'],
         f'root/main and {EXPECTED_AUX_WORKTREES} auxiliary checkout topology differs')
    need(all(w['detached'] and w['branch'] is None for w in aux), 'an auxiliary worktree is not detached')
    need(all(w['clean'] and w['main_ancestor'] and not w['target_exists'] and not w['fuzz_target_exists'] for w in worktrees),
         'a worktree is dirty, not a main ancestor, or contains target output')
    result.update(status='accepted_clean_main', head=head, main=main, origin_main=origin_main,
                  branch=branch, local_branches=local_branches, origin_url=origin_url, remote_heads=remote_heads,
                  product_commit_is_ancestor_of_docs_head=True, worktree_count=len(worktrees), worktrees=worktrees,
                  source_pin_count=len(pins), source_manifest_before_sha256=manifest_sha,
                  product_tree_raw_sha256=product_tree_sha, docs_tree_raw_sha256=docs_tree_sha,
                  source_blob_match_count=len(pins), own_ci_acceptance_status=own_ci['status'])
except Exception as exc:
    result.update(status='failed', error=f'{type(exc).__name__}: {exc}')
finally:
    OUT.mkdir(parents=True, exist_ok=False)
    (OUT/'execution.json').write_text(json.dumps(result, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
    print(json.dumps({k:result.get(k) for k in ('status','product_commit','expected_docs_head','head','worktree_count','source_pin_count','error')}))
sys.exit(0 if result['status']=='accepted_clean_main' else 1)
