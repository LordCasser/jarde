#!/usr/bin/env python3
"""Audit newly saved debugger/normal observations without running any debugger or product."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess
S=Path('/home/lordcasser/.cache/jarde-review/20261015/snapshot-proof-cli-probe-v1')
O=Path(__file__).resolve().parent
def pin(p):
 p=Path(p);b=p.read_bytes();return {'path':str(p),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()}
def load(p):return json.loads(Path(p).read_text())
def norm(x):
 if isinstance(x,dict):return {k:norm(v) for k,v in x.items() if k!='elapsed_millis'}
 if isinstance(x,list):return [norm(v) for v in x]
 return x
inputs={};records={};traces={};rawdocs={}
original=load(O/'audit.json');inputs[str(O/'audit.json')]=pin(O/'audit.json')
cap8=load(S/'class-headers-8.stdout.raw');wide=load(S/'wide.stdout.raw')
for label in ['gdb-class-headers-8','gdb-class-headers-8-v2','gdb-class-headers-8-v3','gdb-class-headers-15','class-headers-15']:
 folder=S/label;d=load(folder/'execution.json');inputs[str(folder/'execution.json')]=pin(folder/'execution.json')
 assert d['exit_code']==0 and d['cwd']==str(folder)
 for p in d['streams'].values():assert pin(p['path'])==p;inputs[p['path']]=p
 if label.startswith('gdb'):
  script=Path(d['script']['path']);actual=pin(script);assert actual['sha256']==d['script']['sha256'];inputs[str(script)]=actual
  assert d['argv'][:6]==['/usr/bin/gdb','--batch','--nx','-x',str(script),'--args']
  intended=original['five_actual_commands'][0]['argv']+['--budget','class_headers=15' if label=='gdb-class-headers-15' else 'class_headers=8']
  assert d['argv'][6:]==d['inferior_argv']==intended
  commands=script.read_text().splitlines()
  allowed=('set debuginfod enabled off','set pagination off','set confirm off','set breakpoint pending off','set print thread-events off','set disable-randomization off',"break '<jarde_reader::budget::Budget>::exceeded'",'commands','silent','printf "JARDE_QUOTA_REFUSAL\\n"','continue','end')
  assert all(x in allowed or re.fullmatch(r'bt (14|16|30)',x) or x.startswith('run ') for x in commands)
  # No return/value/call/jump/argument rewriting commands; only breakpoint, print, bt, continue.
  trace=Path(d['streams']['gdb.stdout.raw']['path']).read_text();traces[label]=trace
  rawdocs[label]=load(d['streams']['inferior.stdout.raw']['path'])
 else:
  assert d['argv']==original['five_actual_commands'][0]['argv']+['--budget','class_headers=15'] and not d['debugger_enabled']
  rawdocs[label]=load(d['streams']['stdout.raw']['path'])
 records[label]=d
bad=rawdocs['gdb-class-headers-8'];badtrace=traces['gdb-class-headers-8']
assert bad['status']=='error' and bad['error']['code']=='cli_request_json' and 'EOF' in bad['error']['message']
assert 'exited with code 01' in badtrace and 'JARDE_QUOTA_REFUSAL' not in badtrace
assert any(x.startswith('run > ') for x in Path(records['gdb-class-headers-8']['script']['path']).read_text().splitlines())
for label in ['gdb-class-headers-8-v2','gdb-class-headers-8-v3','gdb-class-headers-15']:
 target=rawdocs['class-headers-15'] if label=='gdb-class-headers-15' else cap8
 assert rawdocs[label]['limits']==target['limits']
 assert norm(rawdocs[label])==norm(target)
 assert 'exited normally' in traces[label] and traces[label].count('JARDE_QUOTA_REFUSAL')==(1 if label=='gdb-class-headers-15' else 5)
 assert 'run class-source --input ' in Path(records[label]['script']['path']).read_text()
v3=traces['gdb-class-headers-8-v3'];blocks=v3.split('JARDE_QUOTA_REFUSAL\n')[1:];assert len(blocks)==5
stacks=[]
for i,block in enumerate(blocks):
 frames=re.findall(r'^#(\d+)\s+.*? in (.*?) \(\)',block,re.M)
 assert len(frames)==30 and [int(n) for n,_ in frames]==list(range(30))
 symbols=[name for _,name in frames]
 for needle in ['<jarde_reader::budget::Budget>::exceeded','<jarde_reader::budget::Budget>::ensure_within','<jarde_reader::budget::Budget>::charge','jarde_jvm::providers::charge_header_attempt','jarde::facade::resolve_class_source_dependency_read_raw']:assert needle in symbols
 if i<4:
  for needle in ['jarde::facade::selected_reference_header','jarde::facade::snapshot_header_chain_widens','jarde::facade::prove_snapshot_hierarchy_widenings','jarde::facade::snapshot_hierarchy_widenings_presented','jarde::facade::recovery_from_with_class_candidates']:assert needle in symbols
  family='snapshot_hierarchy_widening_in_member_recovery'
 else:
  assert '<jarde::facade::Engine>::prove_double_brace_allocation' in symbols and '<jarde::facade::Engine>::project_class_source_double_brace' in symbols
  family='double_brace_class_projection'
 stacks.append({'ordinal':i+1,'family':family,'frames':[{'frame':int(n),'symbol':f} for n,f in frames]})
fifteen=rawdocs['class-headers-15'];main15=fifteen['methods'][-1]
assert fifteen['execution']['status']=='complete' and fifteen['limits']['class_headers']==15 and fifteen['usage']['class_headers']==15
assert main15['outcome']['report']['execution']['status']=='complete' and main15['outcome']['report']['content']=='contains_statements'
assert norm(main15)==norm(wide['methods'][-1]) and fifteen['text']==wide['text']
assert norm(fifteen['methods'])==norm(wide['methods'])
assert fifteen['class']==wide['class'] and fifteen['view']==wide['view'] and fifteen['coverage']==wide['coverage']
assert {k:v for k,v in fifteen['limits'].items() if k!='class_headers'}=={k:v for k,v in wide['limits'].items() if k!='class_headers'}
assert not any('budget_exceeded' in str(m.get('outcome',{}).get('diagnostics',[])) for m in fifteen['methods'])
cap15trace=traces['gdb-class-headers-15'];assert 'snapshot_hierarchy' not in cap15trace
assert cap15trace.count('JARDE_QUOTA_REFUSAL')==1 and 'prove_double_brace_allocation' in cap15trace and 'project_class_source_double_brace' in cap15trace
cap15frames=re.findall(r'^#(\d+)\s+.*? in (.*?) \(\)',cap15trace,re.M)
assert len(cap15frames)==30 and [int(n) for n,_ in cap15frames]==list(range(30))
assert 'charge_header_attempt' in cap15trace
classification=load(S/'classification.json');inputs[str(S/'classification.json')]=pin(S/'classification.json')
assert pin(S/'classification.json')['sha256']=='24b3f6ea676a0d97dc851fe880839df1c7f7ce51625b2b9b6ff279c900b22042'
tools=classification['tools'];assert pin(tools['gdb_path'])['sha256']==tools['gdb_sha256'] and pin(tools['gdb_path'])['bytes']==tools['gdb_bytes']
inputs[tools['gdb_path']]=pin(tools['gdb_path']);inputs[str(S/'gdb-version.stdout.raw')]=pin(S/'gdb-version.stdout.raw')
assert pin(S/'gdb-version.stdout.raw')['sha256']==tools['gdb_version_sha256']
provider=Path('/home/lordcasser/.codex/worktrees/core-integrity/jarde')
source=subprocess.run(['git','show','c814267719ebc6d8fe14535ebee1b85e41d444db:crates/jarde-jvm/src/providers.rs'],cwd=provider,check=True,capture_output=True).stdout
text=source.decode();assert 'fn charge_header_attempt(budget: &mut Budget)' in text
body=text[text.index('fn charge_header_attempt'):text.index('fn charge_header_attempt')+650]
assert 'budget.charge(CountedBudgetDimension::ClassHeaders, 1)' in body
focused=load('/home/lordcasser/.cache/jarde-validation/core-integrity-20261015/emitter-focused-v2/execution.json')
p=[x for x in focused['source_manifest_before'] if x['path']=='crates/jarde-jvm/src/providers.rs'][0]
assert len(source)==p['bytes'] and hashlib.sha256(source).hexdigest()==p['sha256']
new={'status':'confirmed_dynamic_snapshot_hierarchy_classheaders_stop_hidden_and_additional_double_brace_stop','recorded_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'original_five_probe_audit':pin(O/'audit.json'),'inputs':inputs,'debugger_scope':{'invalid_v1':'Excluded: run redirection reset inferior args; cli_request_json EOF; inferior exit01; GDB exit0 is not product success. Original five normal probes unaffected.','partial_v2':'Five quota breakpoint hits, bt16 first four only reach selected_reference_header; lower attribution strength retained, not substituted for full v3.','valid_v3':'Five Budget::exceeded hits in real charge_header_attempt path; first4 stack through snapshot proof wrapper, fifth double-brace projection; bt30 only, no argument/error-local value capture. Inferior JSON equals normalcap8 after elapsed normalization and exits normally.','mutation_scope':'Read-only breakpoint/printf/bt/continue script; no return/value/function-call/jump mutation commands. Same absolute frozen binary/JAR; source/CLI hashes already bound by base audit, no invented extra historical inputpin receipt.','not_a_seven_path_matrix':'Two dynamic call families shown; no dynamic coverage claimed for other static wrappers.'},'actual_refusal_stacks':stacks,'classheaders_source_binding':{'commit':'c814267719ebc6d8fe14535ebee1b85e41d444db','providers':p,'charge_dimension':'ClassHeaders','requested':1},'cap15_normal':{'argv':records['class-headers-15']['argv'],'exit_code':0,'seconds':records['class-headers-15']['seconds'],'class_execution':fifteen['execution'],'main_report_execution':main15['outcome']['report']['execution'],'main_content':'contains_statements','main_normalized_equal_wide':True,'class_text_equal_wide':True,'trace_status':'No cap15 trace in reviewed evidence; do not assign its possible extra postbody refusal to a specific helper.'},'updated_proof_strength':{'observed':'Real ClassHeaders quota refusal now dynamically observed inside snapshot hierarchy proof chain and later double-brace proof in same cap8 inferior that returns class/main Complete without quota diagnostic. This is stronger than original five raw alone.','mechanism':'Static resolver stores non-Complete in helper-local execution and returnsNone, while proof returnsVec/Option; bt shows Budget::exceeded but not the subsequent Result variant at wrapper catch. Cannot claim four Err arms specifically executed.','spec':'A real requested proof budget stop is being hidden by Complete, contrary to existing truthful execution/termination and CLI noncomplete-exit contract. ExplanationOnly is a valid content classification but cannot substitute for the lost termination plane.','remaining_isolation':'Exact BCI/class arguments and Err-vs-Ok(None,localPartial) at each return remain unprinted; cap15 projection refusal and other wrappers remain untraced. No broad additional sweep implied.'},'scope':'Six normal CLI observations (original5+cap15), one failed debug attempt, one partial valid trace, one full valid trace. Not regression matrix/workspace/CI acceptance. No new product/debugger/toolchain runs, source/capture edits or target access by auditor.'}
assert records['class-headers-15']['seconds']<.03
assert classification['normal_cli_executions']==6 and len(classification['successful_debugger_runs'])==3
new['debugger_scope']['valid_cap15']='Exactly1 ClassHeaders refusal: full30frame double-brace projection chain, no snapshot frame; full inferior JSON equals normalcap15 after elapsed-only normalization, limits exactly equal, exited normally.'
new['actual_refusal_stacks'].append({'probe':'cap15','ordinal':1,'family':'double_brace_class_projection','frames':[{'frame':int(n),'symbol':f} for n,f in cap15frames]})
new['cap15_normal']['trace_status']='Dynamically confirmed by separate cap15 trace:1 double-brace postbody refusal,0 snapshot refusal, class stillComplete/exit0 with successful unchanged body.'
new['updated_proof_strength']['observed']='cap8 dynamically shows4 snapshot-proof refusals plus1 later double-brace refusal while main/class remainComplete/explanation_only/exit0. cap15 independently shows1 double-brace refusal without snapshot frames while main remainsComplete/contains_statements and classComplete/exit0.'
new['updated_proof_strength']['remaining_isolation']='Exact BCI/class arguments and subsequent Err-vs-Ok(None,localPartial) values were not printed. The snapshot and double-brace call families are dynamically identified; reference-overload/direct-parent-accessor/direct-parent-field wrappers remain static only; cap14 has no debugger trace.'
new['scope']='Six normal CLI observations,3 valid debugger runs(cap8 bt16,cap8 bt30,cap15 bt30),1 invalid debugger attempt. These are diagnostics, not seven-path regression/workspace/CI acceptance. No new product/debugger/toolchain runs, source/capture edits or target access by auditor.'
new['normalization']='Only elapsed_millis fields removed for report comparison; raw limits checked exactly equal first, including elapsed_millis=10000. No other field masked.'
new['gdb_tool_pin']=tools
(O/'trace-review.json').write_text(json.dumps(new,ensure_ascii=False,indent=2)+'\n')
print(json.dumps({'status':new['status'],'cap8_refusals':5,'cap8_snapshot':4,'cap8_double_brace':1,'cap15_refusals':1,'cap15_snapshot':0},ensure_ascii=False))
