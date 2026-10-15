#!/usr/bin/env python3
"""Read frozen CLI observations only. Does not run CLI, Cargo/JDK or access target."""
from pathlib import Path
import datetime,difflib,hashlib,json,subprocess,zipfile
S=Path('/home/lordcasser/.cache/jarde-review/20261015/snapshot-proof-cli-probe-v1')
O=Path(__file__).resolve().parent
R=Path('/home/lordcasser/.codex/worktrees/core-integrity/jarde')
COMMIT='c814267719ebc6d8fe14535ebee1b85e41d444db'
inputs={}
def pin(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
def load(p):inputs[str(p)]=pin(p);return json.loads(Path(p).read_text())
def verify(p):assert pin(p['path'])==p;inputs[p['path']]=p
def normalized(x):
 if isinstance(x,dict):return {k:normalized(v) for k,v in x.items() if k!='elapsed_millis'}
 if isinstance(x,list):return [normalized(y) for y in x]
 return x
def codes(x):
 if isinstance(x,dict):return ([x['code']] if 'code' in x and 'severity' in x else [])+sum((codes(v) for v in x.values()),[])
 if isinstance(x,list):return sum((codes(y) for y in x),[])
 return []
e=load(S/'execution.json');extra=load(S/'execution-extra.json')
assert e['status']=='completed_observations_not_yet_classified' and e['product_commit']==COMMIT
assert extra['base_execution']==str(S/'execution.json') and extra['base_execution_sha256']==pin(S/'execution.json')['sha256']
assert e['inputs_unchanged_after'] and extra['inputs_unchanged_after'] and e['no_cargo_jdk_source_or_target_mutation']
cli,jar,*sourcepins=e['inputs'];verify(cli);verify(jar)
assert cli['bytes']==59602720 and cli['sha256']=='cdf06694fafcb4e8cef5a4b9452676489e8cf02495c0254ce38a21e44ee4c7d5'
assert Path(cli['path']).stat().st_mode&0o777==0o555
assert jar['bytes']==4469 and jar['sha256']=='1ee63f3836a5e514cea57b7349dd515ca3f98550154beff187fdca4d019105b0'
focus=load(Path('/home/lordcasser/.cache/jarde-validation/core-integrity-20261015/emitter-focused-v2/execution.json'))
assert focus['cli']==cli and focus['source_manifest_before']==focus['source_manifest_after']
focussrc={r['path']:(r['bytes'],r['sha256']) for r in focus['source_manifest_before']}
blobs={};source_bind=[]
for p in sourcepins:
 rel=Path(p['path']).relative_to(R).as_posix()
 b=subprocess.run(['git','show',COMMIT+':'+rel],cwd=R,check=True,capture_output=True).stdout
 assert (len(b),hashlib.sha256(b).hexdigest())==(p['bytes'],p['sha256'])==focussrc[rel]
 blobs[rel]=b.decode();source_bind.append({'path':rel,'bytes':len(b),'sha256':p['sha256'],'commit':COMMIT,'focused_manifest_match':True})
with zipfile.ZipFile(jar['path']) as z:
 assert z.testzip() is None
 entries=[{'name':x.filename,'bytes':x.file_size,'crc32':x.CRC} for x in z.infolist()]
 names={x['name'] for x in entries}
 assert {'H1.class','H1$Greet.class','H1$TwoLevel.class','H1$Mid.class','H1$Multi.class','H1$Other.class','H1$1.class','H1$ViaSub.class','H1$Sub.class','H1$Caller.class','H1$Fin.class'}<=names
 assert len(z.read('H1.class'))==1559
rows=e['commands']+extra['commands'];assert len(rows)==5
assert [x['label'] for x in rows]==['wide','class-headers-1','class-headers-2','class-headers-8','class-headers-14']
baseargv=rows[0]['argv'];assert baseargv==[cli['path'],'class-source','--input',jar['path'],'--class','H1','--evidence','essential','--format','json','--budget','elapsed_millis=10000','--budget','output_bytes=1048576']
reports={};summaries=[]
for row in rows:
 label=row['label'];assert row['cwd']==str(S) and 0<row['duration_seconds']<.03
 cap=4096 if label=='wide' else int(label.rsplit('-',1)[1])
 assert row['argv']==baseargv+([] if label=='wide' else ['--budget',f'class_headers={cap}'])
 for p in row['streams'].values():verify(p)
 assert row['streams']['stderr']['bytes']==0
 doc=json.loads(Path(row['streams']['stdout']['path']).read_text());reports[label]=doc
 assert doc['limits']['class_headers']==cap and doc['usage']['class_headers']<=cap
 assert doc['limits']['elapsed_millis']==10000 and doc['limits']['output_bytes']==1048576
 assert doc['outcome']=='performed' and doc['usage']==doc['execution']['usage'] and len(doc['methods'])==6
 members=[]
 for m in doc['methods']:
  outcome=m['outcome'];q=outcome.get('report',{});name=bytes(m['item']['identity']['name']).decode()
  entry={'index':m['item']['index'],'name':name,'descriptor':bytes(m['item']['identity']['descriptor']).decode(),'outcome_kind':outcome['kind'],'analysis_execution':outcome.get('analysis',{}).get('execution'),'recovery_execution':q.get('execution'),'refusal_execution':outcome.get('execution'),'content':q.get('content'),'quality':q.get('quality'),'diagnostic_codes':[d['code'] for d in q.get('diagnostics',outcome.get('diagnostics',[]))],'markers':m['markers'],'text_sha256':hashlib.sha256(m['text'].encode()).hexdigest()}
  members.append(entry)
  if name=='main':
   main=m;maintext=q.get('text',m['text']);(O/(label+'-main.txt')).write_text(maintext)
 if label=='wide':assert row['exit_code']==0 and doc['execution']['status']=='complete' and doc['usage']['class_headers']==16 and main['outcome']['report']['content']=='contains_statements' and '(H1$Greet) new H1$TwoLevel()' in maintext
 elif cap in (1,2):
  assert row['exit_code']==4 and doc['execution']['status']=='partial' and doc['usage']['class_headers']==cap
  assert main['outcome']['kind']=='refused' and main['outcome']['execution']['reason']=={'kind':'budget_exceeded','dimension':'class_headers'}
  assert 'budget_exceeded_class_headers' in codes(main)
 else:
  assert row['exit_code']==0 and doc['execution']['status']=='complete' and doc['usage']['class_headers']==cap
  q=main['outcome']['report'];assert q['execution']['status']=='complete' and q['content']=='explanation_only' and q['quality']=='fallback'
  assert main['outcome']['analysis']['execution']['status']=='complete'
  assert not any(x.startswith(('budget_exceeded','cancelled')) for x in codes(doc))
  assert 'no safe reference conversion evidence' in maintext
 summaries.append({'label':label,'argv':row['argv'],'exit_code':row['exit_code'],'duration_seconds':row['duration_seconds'],'limits':doc['limits'],'usage':doc['usage'],'execution':doc['execution'],'diagnostic_codes':codes(doc),'members':members})
wide=reports['wide']
for label,d in reports.items():
 assert d['class']==wide['class'] and d['view']==wide['view'] and d['coverage']==wide['coverage']
 assert {k:v for k,v in d['limits'].items() if k!='class_headers'}=={k:v for k,v in wide['limits'].items() if k!='class_headers'}
 assert [x['item'] for x in d['methods']]==[x['item'] for x in wide['methods']]
 for i in range(5):assert normalized(d['methods'][i])==normalized(wide['methods'][i])
 if label in ['class-headers-8','class-headers-14']:assert normalized(d['methods'][5]['outcome']['analysis'])==normalized(wide['methods'][5]['outcome']['analysis'])
 (O/(label+'-vs-wide-main.patch')).write_text(''.join(difflib.unified_diff((O/'wide-main.txt').read_text().splitlines(True),(O/(label+'-main.txt')).read_text().splitlines(True),fromfile='wide/main',tofile=label+'/main')))
facade=blobs['src/facade.rs'];task=blobs['crates/jarde-cli/src/task.rs'];budget=blobs['crates/jarde-reader/src/budget.rs']
assert 'task_budget(&overrides)?' in task and 'common.scope.as_deref(), &common.budget' in task
assert 'engine.class_source_with_evidence(' in task and 'fn class_source_plane(report: &ClassSourceReport) -> Plane {\n    plane_of(&report.execution)' in task
assert 'self.ensure_within(dimension, requested)?;' in budget and 'Some(total) if total <= limit => Ok(())' in budget
points=[]
for needle,why in [('fn reference_overload_calls_presented(', 'BudgetExceeded/Cancelled converted to empty certificate vector'),('match prove_direct_parent_accessor_calls(content, request, analyzed.ir(), budget)', 'BudgetExceeded/Cancelled discarded before recovery'),('let fields = match prove_direct_parent_field_writes(content, request, analyzed.ir(), budget)', 'BudgetExceeded/Cancelled converted to empty field proof vector'),('fn snapshot_hierarchy_widenings_presented(', 'BudgetExceeded/Cancelled converted to empty widening vector'),('fn resolve_class_source_dependency_read_raw(', 'resolver non-Complete execution merged into caller-local execution, then Ok(None)'),('fn selected_reference_header(', 'passes proof-local execution into dependency reader'),('fn prove_snapshot_hierarchy_widenings(', 'local execution exists but return type exposes only Vec'),('fn recover_analyzed', 'entry marker')]:
 if needle in facade:points.append({'line':facade[:facade.index(needle)].count('\n')+1,'needle':needle,'meaning':why})
assert len(points)>=7
for name in ['analysis-contracts','java8-recovery']:
 p=R/'openspec/specs'/name/'spec.md';inputs[str(p)]=pin(p)
result={'status':'independently_confirmed_public_budget_conditioned_proof_loss_with_limited_internal_attribution','recorded_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'product_commit_as_recorded':COMMIT,'source_binding':source_bind,'jar_entries':entries,'five_actual_commands':summaries,'facts':{'only_parameter_difference':'class_headers cap, with JSON limits confirming application','same_input_class_members_scope_profile':'verified identities,raw member items,view,coverage; same frozen JAR; all remaining limits equal','earlier_five_members':'identical after removing only elapsed_millis','main_analysis_wide_8_14':'identical after removing only elapsed_millis','caps_1_2':'explicit ClassHeaders stop with main Refused, class Partial, exit4','caps_8_14':'class and main recovery Complete, exit0, no budget diagnostic; proof-dependent conversion fallback changes main to explanation_only','wide':'class Complete,header usage16; main report snapshot headers15,structured statement text with reference casts'},'static_source_points':points,'interpretation':{'strength':'Actual end-to-end budget-conditioned loss of conversion evidence is proved. With reviewed charge/error and wrapper/local-execution loss paths, operation-level Stop hiding is strongly supported; exact private helper/denied charge not directly traced.','not_proved':'No raw captures an original Error inside snapshot_hierarchy_widenings_presented. The four earlier proof routes and report-bearing resolver stop local to a helper must not be collapsed into a snapshot-helper-only dynamic claim.','spec':'Explanation-only/Produced can validly complete when evidence is semantically absent; that content alone is no violation. A budget-refused proof in the requested run must retain truthful non-Complete execution/diagnostics under existing budget/execution and CLI contracts, rather than be replaced by ordinary absence. Structural coverage may remain complete.','next_isolation':'Use the existing real H1 fixture and actual public class-source path to isolate which prebody proof first receives BudgetExceeded or a non-Complete resolver execution; retain that original evidence and link it to the member/top-level execution. No new broad sweep, arbitrary higher cap, or helper-only success substitute needed.'},'input_pins':inputs,'limitations':['Five existing CLI observations only, each under0.03s as recorded; no new executions.','No body/source correctness, verification or new semantic recovery claim; wide text is presentation, not compile/execution validation.','Three relevant source files bind to c814 and focused manifest; this is not an exhaustive proof of every native executable/source linkage.','Input unchanged/no target mutations are original bounded observation assertions, not a retroactive full filesystem mutation audit. No current target examined, source/capture edited or product tools run.']}
(O/'audit.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'status':result['status'],'commands':5,'sources':len(source_bind),'static_points':len(points)},ensure_ascii=False))
