from pathlib import Path
import os,json,subprocess,hashlib,datetime
D=Path(__file__).resolve().parent
R=Path('/home/lordcasser/.codex/worktrees/core-integrity/jarde')
N='preserve-reference-overload-dependency-proof-stops'
A=R/'openspec/changes/archive'/('2026-10-16-'+N)
S=R/'openspec/specs/java8-recovery/spec.md'
def pin(p):
 b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest())
assert not A.exists()
sync=json.loads((D/'sync-receipt.json').read_text())
assert pin(S)['sha256']==sync['after_sha256']
delta=R/'openspec/changes'/N/'specs/java8-recovery/spec.md'
assert pin(delta)['sha256']==sync['delta_sha256']
assert '- [ ]' not in (R/'openspec/changes'/N/'tasks.md').read_text()
assert json.loads((D/'apply-final.stdout.raw').read_text())['state']=='all_done'
B=Path('/home/lordcasser/.cache/jarde-validation/core-integrity-20261015')
assert json.loads((B/'reference-overload-fingerprint-own-ci-root-acceptance-v4.json').read_text())['errors']==[]
assert json.loads((B/'CORE/own-ci-reference-overload-fingerprint-independent-audit-v1/audit-result-v6.json').read_text())['status']=='accepted_independent_exact_product_ci_review'
env=os.environ.copy();env['PATH']='/home/lordcasser/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin:'+env['PATH']
commands=[]
def run(label,args):
 p=subprocess.run(args,cwd=R,env=env,capture_output=True)
 streams={}
 for k,b in [('stdout',p.stdout),('stderr',p.stderr)]:
  f=D/(label+'.'+k+'.raw');assert not f.exists();f.write_bytes(b);streams[k]=pin(f)
 commands.append(dict(argv=args,cwd=str(R),exit_code=p.returncode,streams=streams))
 (D/'archive-execution.json').write_text(json.dumps(dict(status='running',commands=commands),indent=2)+'\n')
 assert p.returncode==0,(label,p.stderr.decode(errors='replace'))
 if label.endswith('strict'):
  expected=365 if label.startswith('prearchive') else 364
  assert f'Totals: {expected} passed, 0 failed ({expected} items)' in p.stdout.decode(errors='replace'),label
 return p.stdout
run('prearchive-ci-strict',['/home/lordcasser/.cache/jarde-validation/ordinary-try-list-20261014/tools/openspec-ci/node_modules/.bin/openspec','validate','--all','--strict','--no-interactive'])
run('prearchive-local-strict',['/home/lordcasser/.local/bin/openspec','validate','--all','--strict','--no-interactive'])
run('archive',['/home/lordcasser/.local/bin/openspec','archive',N,'--yes','--skip-specs','--json'])
assert A.is_dir() and not (R/'openspec/changes'/N).exists()
assert pin(S)['sha256']==sync['after_sha256']
run('postarchive-ci-strict',['/home/lordcasser/.cache/jarde-validation/ordinary-try-list-20261014/tools/openspec-ci/node_modules/.bin/openspec','validate','--all','--strict','--no-interactive'])
run('postarchive-local-strict',['/home/lordcasser/.local/bin/openspec','validate','--all','--strict','--no-interactive'])
result=dict(status='accepted_archive_after_verified_agent_sync',archive_root=str(A),spec_sync_already_verified=True,cli_skip_specs_reason='Agent sync already applied and exact delta/main comparison found nothing remaining; preserve synced main bytes.',sync_receipt=pin(D/'sync-receipt.json'),main=pin(S),commands=commands)
(D/'archive-execution.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(dict(status=result['status'],archive_root=str(A)),ensure_ascii=False))
