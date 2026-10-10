import datetime, hashlib, json, shutil, stat, subprocess, sys
from pathlib import Path
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
OUT=Path(sys.argv[1]); OUT.mkdir(parents=True,exist_ok=False)
rows=[]
def command(argv,cwd=ROOT,allowed=(0,)):
    proc=subprocess.run(argv,cwd=cwd,capture_output=True)
    index=len(rows); streams={}
    for name,raw in [('stdout',proc.stdout),('stderr',proc.stderr)]:
        filename=f'{index}.{name}.raw'; (OUT/filename).write_bytes(raw)
        streams[name]={'path':filename,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
    rows.append({'argv':argv,'cwd':str(cwd),'exit_code':proc.returncode,'streams':streams})
    if proc.returncode not in allowed: raise RuntimeError(f'command {index} failed: {argv}')
    return proc.stdout.decode().strip()
result={'schema':'jarde-clean-main-delivery-audit-root-v1','utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'commands':rows,'audit_script_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
try:
    head=command(['git','rev-parse','HEAD']); main=command(['git','rev-parse','main']); origin=command(['git','rev-parse','origin/main'])
    branch=command(['git','branch','--show-current']); branches=command(['git','for-each-ref','--format=%(refname:short)','refs/heads']).splitlines()
    worktrees_raw=command(['git','worktree','list','--porcelain'])
    worktrees=[]
    for block in worktrees_raw.split('\n\n'):
        item=dict(line.split(' ',1) if ' ' in line else (line,True) for line in block.splitlines()); path=Path(item['worktree'])
        status=command(['git','status','--porcelain=v1','--untracked-files=all'],path)
        ancestor=command(['git','merge-base','--is-ancestor',item['HEAD'],'main'],allowed=(0,1))
        ancestor_exit=rows[-1]['exit_code']
        worktrees.append({'path':str(path),'head':item['HEAD'],'branch':item.get('branch'),'detached':bool(item.get('detached')),'clean':status=='','main_ancestor':ancestor_exit==0,'target_exists':(path/'target').exists(),'fuzz_target_exists':(path/'fuzz/target').exists()})
    cli=Path('/private/tmp/jarde-return-arm-latch-cli-v1'); cli_sha=hashlib.sha256(cli.read_bytes()).hexdigest(); mode=stat.S_IMODE(cli.stat().st_mode)
    free=shutil.disk_usage(ROOT).free
    result.update(head=head,main=main,origin_main=origin,branch=branch,local_branches=branches,worktrees=worktrees,free_bytes=free,candidate_cli={'path':str(cli),'sha256':cli_sha,'mode':oct(mode)})
    assert head==main==origin and branch=='main' and branches==['main']
    assert len(worktrees)==15 and all(w['clean'] and w['main_ancestor'] and not w['target_exists'] and not w['fuzz_target_exists'] for w in worktrees)
    assert all(w['detached'] and w['branch'] is None for w in worktrees if w['path']!=str(ROOT))
    assert free>=5*1024**3 and cli_sha=='9ae5817d25749add0709b4f156c71ce23ac0e5ad41d66ee0136742024461a28f' and mode==0o555
    result['status']='accepted_clean_main'
except Exception as exc:
    result.update(status='failed',error=f'{type(exc).__name__}: {exc}')
finally:
    (OUT/'execution.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({k:result.get(k) for k in ['status','head','origin_main','free_bytes','error']}))
sys.exit(0 if result['status']=='accepted_clean_main' else 1)
