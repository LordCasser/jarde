import sys, hashlib, json, importlib.util, shutil, subprocess, datetime
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde')
P=Path('/private/tmp/jarde-local-reference-write-candidate-complete-source-prep-root-v1')
O=Path('/private/tmp/jarde-local-reference-write-candidate-complete-source-launch-root-v1')
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
need=lambda ok,msg: (_ for _ in ()).throw(ValueError(msg)) if not ok else None
need(not O.exists() and not (R/'target').exists(),'fresh output/target preflight')
need(shutil.disk_usage(R).free>=5*1024**3,'5 GiB preflight')
need(H(P/'run-plan.json')=='617ccfaf977a82b915d2d2c5b4451d61a221b3cc7371660d85789624f07866f9','run plan pin')
need(H(Path('/private/tmp/jarde-local-reference-write-baseline-prep-luna-v2/collect_controls.py'))=='626865322096ff1ac7418dce672feb526e9ffddbda9fad53fe006482de47d4f3','collector pin')
C=Path('/private/tmp/jarde-iterator-assignment-full-validation-prep-luna-v2/workspace_runner.py')
need(H(C)=='63e52b55fe5d20d5f2ece98ac80676ccdb2de8dfd33d7199b688e075fb1e9673','closure helper pin')
spec=importlib.util.spec_from_file_location('local_reference_closure',C);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
before=m.pin_sources(m.source_paths());need(len(before)==4784,'source closure count')
F=Path('/private/tmp/jarde-local-reference-write-candidate-build-root-v1/execution.json')
need(H(F)=='0e08e135b87844a5e91bbb0d7ac9ca76240892bc080d793efc7193f6b9aacb67','candidate build receipt pin')
f=json.loads(F.read_text());need(before==f['source_pins_before_format']==f['source_pins_after_format']==f['source_pins_after'],'live source differs from candidate build')
plan=json.loads((P/'run-plan.json').read_text());O.mkdir()
x={'schema':'local-reference-write-candidate-complete-source-launch-root-v1','status':'running','started_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source_pins_before':before,'closure_helper':{'path':str(C),'sha256':H(C)},'run_plan':{'path':str(P/'run-plan.json'),'sha256':H(P/'run-plan.json')},'runner':{'path':str(Path(__file__)),'sha256':H(__file__)},'commands':[],'scope':'candidate controlled complete-class observation only'}
def save():(O/'execution.json').write_text(json.dumps(x,indent=2)+'\n')
save()
try:
 for run in plan['runs']:
  need(shutil.disk_usage(R).free>=5*1024**3 and not (R/'target').exists(),'profile resource/target preflight')
  inv=O/('invocation-'+run['profile'])
  argv=['python3','/private/tmp/jarde-cf11-invocations-root-v1/record.py',str(inv),*run['argv']]
  row={'profile':run['profile'],'argv':run['argv'],'invocation':str(inv)};x['commands'].append(row);save()
  row['exit_code']=subprocess.call(argv,cwd=R);save();need(row['exit_code']==0,'collector failed '+run['profile'])
  ex=Path(run['capture_root'])/'execution.json';capt=json.loads(ex.read_text());need(capt['status']=='observed' and capt['javac_debug_profile']==run['profile'],'capture not completed')
  row.update(capture_execution={'path':str(ex),'sha256':H(ex)},command_count=capt['actual_command_count']);save()
 x['status']='observed_complete_profiles'
except BaseException as e:
 x.update(status='failed',error=str(e));raise
finally:
 x.update(source_pins_after=m.pin_sources(m.source_paths()),target_absent=not (R/'target').exists(),free_bytes_after=shutil.disk_usage(R).free,finished_at=datetime.datetime.now(datetime.timezone.utc).isoformat());save()
need(x['source_pins_before']==x['source_pins_after'] and x['target_absent'],'source/target postcheck')
print(json.dumps({'status':x['status'],'commands':[{'profile':r['profile'],'capture_command_count':r['command_count']} for r in x['commands']],'source_count':len(before),'target_absent':x['target_absent']}))
