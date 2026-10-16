from pathlib import Path
import os,subprocess,json,hashlib,sys
D=Path(__file__).resolve().parent;R=Path('/home/lordcasser/.codex/worktrees/local-increment-finally/jarde');label=sys.argv[1];argv=['/home/lordcasser/.cache/jarde-validation/ordinary-try-list-20261014/tools/openspec-ci/node_modules/.bin/openspec']+sys.argv[2:]
assert not (D/(label+'.execution.json')).exists()
env=os.environ.copy();env['PATH']='/home/lordcasser/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin:'+env['PATH'];p=subprocess.run(argv,cwd=R,env=env,capture_output=True);streams={}
for k,b in [('stdout',p.stdout),('stderr',p.stderr)]:
 f=D/(label+'.'+k+'.raw');assert not f.exists();f.write_bytes(b);streams[k]=dict(path=str(f),bytes=len(b),sha256=hashlib.sha256(b).hexdigest())
(D/(label+'.execution.json')).write_text(json.dumps(dict(argv=argv,cwd=str(R),exit_code=p.returncode,streams=streams),indent=2)+'\n');sys.stdout.buffer.write(p.stdout);sys.stderr.buffer.write(p.stderr);sys.exit(p.returncode)
