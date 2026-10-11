from pathlib import Path
import hashlib,json,subprocess,resource,shutil,sys,time
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
CLI=Path(sys.argv[1]).resolve(strict=True);OUT=Path(sys.argv[2]);OUT.mkdir(exist_ok=False)
BASE=Path('/private/tmp/jarde-lambda-depth-baseline-root-v1')
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def pin(p):
 p=Path(p);return {'path':str(p),'bytes':p.stat().st_size,'sha256':sha(p)}
def differences(a,b,path='$'):
 if type(a)!=type(b):return [{'path':path,'before':a,'after':b}]
 if isinstance(a,dict):
  if a.keys()!=b.keys():return [{'path':path,'before_keys':list(a),'after_keys':list(b)}]
  return sum((differences(a[k],b[k],path+'.'+k) for k in a),[])
 if isinstance(a,list):
  if len(a)!=len(b):return [{'path':path,'before_length':len(a),'after_length':len(b)}]
  return sum((differences(x,y,path+f'[{i}]') for i,(x,y) in enumerate(zip(a,b))),[])
 return [] if a==b else [{'path':path,'before':a,'after':b}]
rec={'schema':'arithmetic-stack-full-report-replay-root-v1','status':'running','runner':pin(__file__),'cli':pin(CLI),'commands':[],'allowed_variable_fields':['elapsed_millis'],'target_execution':False}
def save():(OUT/'execution.json').write_text(json.dumps(rec,indent=2)+'\n')
def run(argv,old,cwd,limit=None,reference=None):
 assert shutil.disk_usage(ROOT).free>=5*1024**3
 i=len(rec['commands']);input_path=Path(argv[argv.index('--input')+1]);before=pin(input_path)
 started=time.monotonic()
 c=subprocess.run(argv,cwd=cwd,capture_output=True,timeout=60,preexec_fn=(lambda:resource.setrlimit(resource.RLIMIT_STACK,(limit,limit))) if limit else None)
 stdout=OUT/f'{i}.stdout.raw';stderr=OUT/f'{i}.stderr.raw';stdout.write_bytes(c.stdout);stderr.write_bytes(c.stderr)
 row={'argv':argv,'cwd':str(cwd),'main_stack_limit_bytes':limit,'exit_code':c.returncode,'duration_seconds':time.monotonic()-started,'input':before,'reference_record':pin(reference) if reference else None,'old_stdout':pin(old),'streams':{'stdout':pin(stdout),'stderr':pin(stderr)}}
 rec['commands'].append(row);save();assert c.returncode==0 and not c.stderr,(i,c.stderr)
 assert pin(input_path)==before
 diff=differences(json.loads(Path(old).read_bytes()),json.loads(c.stdout));row['all_field_differences']=diff
 row['non_time_differences']=[d for d in diff if not(d['path'].endswith('.elapsed_millis') and isinstance(d.get('before'),(int,float)) and isinstance(d.get('after'),(int,float)))]
 save();assert not row['non_time_differences'],(i,row['non_time_differences'][:4])
try:
 argv=[str(CLI),'class-source','--input',str(BASE/'LambdaDeep.class'),'--class','LambdaDeep','--policy','single-class','--format','json','--evidence','all']
 run(argv,BASE/'cli-default-stdout.raw',ROOT,2097152,BASE/'execution.json')
 for name in ['jarde-compound-final-runtime-cli-v2','jarde-compound-cf12-final-cli-v2','jarde-compound-negative-render-final-cli-v2','jarde-candidate-continuation-final-cli-v2-attempt-v4','jarde-null-builder-final-cli-v2-replay']:
  record=Path('/private/tmp')/name/'execution.json';oldrec=json.loads(record.read_text())
  assert oldrec['status']=={'jarde-compound-final-runtime-cli-v2':'incomplete_or_mismatching_observations','jarde-compound-cf12-final-cli-v2':'observed_pending_root_independent_acceptance','jarde-compound-negative-render-final-cli-v2':'observations_collected','jarde-candidate-continuation-final-cli-v2-attempt-v4':'all_19_commands_observed','jarde-null-builder-final-cli-v2-replay':'all_19_commands_observed'}[name],oldrec['status']
  for row in oldrec['commands']:
   argv=row['argv']
   if len(argv)<2 or argv[1]!='class-source':continue
   assert row['exit_code']==0
   prior=row['streams']['stdout'];assert pin(prior['path'])['sha256']==prior['sha256']
   run([str(CLI)]+argv[1:],prior['path'],Path(row['cwd']),reference=record)
 rec['status']='passed';rec['full_report_pairs']=len(rec['commands']);rec['cli_unchanged']=pin(CLI)==rec['cli'];assert rec['cli_unchanged']
except BaseException as e:
 rec['status']='failed';rec['error']=repr(e)
finally:
 rec['target_absent']=not(ROOT/'target').exists();save();print(json.dumps({k:rec.get(k) for k in ['status','error','full_report_pairs','target_absent']}))
sys.exit(0 if rec['status']=='passed' else 1)
