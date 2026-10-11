from pathlib import Path
import datetime,hashlib,json,os,subprocess,sys
p=Path('/private/tmp/jarde-computed-for-own-ci-verifier-root-v1.py')
b=Path('/private/tmp/jarde-computed-for-own-ci-acceptance-root-v1')
argv=[sys.executable,str(p),'--product-commit','8bef1b76acfc93e1b1510e8cb71d7c7a25b0fdc6','--run-id','38131239404','--capture','/private/tmp/jarde-computed-for-own-ci-capture-root-v1','--workspace','/private/tmp/jarde-computed-for-workspace-root-v2','--out',str(b)+'.json','--checklist','/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-computed-for-initializers/results/final-local-product-validation-root-v1/ci-source-checklist.md','--final-cli-build','/private/tmp/jarde-computed-for-final-cli-build-root-v2/execution.json','--final-cli-runner','/private/tmp/jarde-final-cli-build-root-v1/build-final-cli.py','--final-cli-runner-sha256','a72b70a7b2699bf5f936f81b657df4073f105be3fceacf753e865f0ce2d4d79b','--build-source-base','d9e7e7b4085dd8dad5ce5abcf662c2ea4433eb65']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
started=datetime.datetime.now(datetime.timezone.utc).isoformat();before=sha(p)
env=os.environ.copy();env['PYTHONDONTWRITEBYTECODE']='1'
with Path(str(b)+'.stdout').open('xb') as out,Path(str(b)+'.stderr').open('xb') as err:
 r=subprocess.run(argv,cwd='/Users/lordcasser/workspace/projects/jarde',env=env,stdout=out,stderr=err)
receipt={'argv':argv,'cwd':'/Users/lordcasser/workspace/projects/jarde','started_at':started,'finished_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'exit_code':r.returncode,'verifier_sha256_before':before,'verifier_sha256_after':sha(p),'invoker_sha256':sha(Path(__file__)),'streams':{n:{'path':str(b)+'.'+n,'sha256':sha(Path(str(b)+'.'+n)),'bytes':Path(str(b)+'.'+n).stat().st_size} for n in ('stdout','stderr')}}
Path(str(b)+'.invocation.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'exit_code':r.returncode,'verifier_pins_unchanged':before==sha(p)}))
sys.exit(r.returncode)
