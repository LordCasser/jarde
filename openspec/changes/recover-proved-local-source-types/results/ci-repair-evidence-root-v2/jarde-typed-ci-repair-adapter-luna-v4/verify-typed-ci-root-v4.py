#!/usr/bin/env python3
"""Verify typed-local product CI logs plus exact source/build pins (private adapter draft)."""
from __future__ import annotations
import argparse, gzip, hashlib, json, os, re, stat, subprocess
from pathlib import Path
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
CHANGE=ROOT/'openspec/changes/recover-proved-local-source-types'
RESULTS=CHANGE/'results'
CAPTURE=Path('/private/tmp/jarde-typed-ci-capture-root-v4')
ACCEPT=RESULTS/'typed-ci-product-v4/acceptance-root-v4.json'
TEST_PATHS={'.github/workflows/ci.yml','crates/jarde-java/tests/cf12_proved_local_source_types.rs',
 'crates/jarde-java/tests/p3_proved_boolean_conditional_returns.rs','crates/jarde-java/tests/p3_patterns.rs',
 'tests/p3_declarations.rs','tests/p3_parameter_slots.rs','tests/p3_guard.rs',
 'tests/p3_popped_static_qualifier.rs','tests/p3_invocation_arguments.rs','tests/p3_array_slot_retype_locals.rs'}
PRODUCT_PATHS={'Cargo.toml','Cargo.lock','crates/jarde-jvm/Cargo.toml','crates/jarde-reader/Cargo.toml',
 'crates/jarde-query/Cargo.toml','crates/jarde-java/Cargo.toml','crates/jarde-cli/Cargo.toml',
 'crates/jarde-java/src/region.rs','crates/jarde-java/src/build.rs','crates/jarde-java/src/emit.rs',
 'crates/jarde-java/src/report.rs','crates/jarde-java/src/lib.rs','src/class_source.rs','src/facade.rs',
 'src/lib.rs','crates/jarde-cli/src/main.rs','crates/jarde-cli/src/task.rs'}
INCLUDE_RE=re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')
TEST_RE=re.compile(r'(?ms)^\s*#\[test\]\s*((?:#\[[^\]]+\]\s*)*)fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(')
SUMMARY_RE=re.compile(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')
WORKSPACE_COMMAND='cargo test --workspace --all-targets --all-features --locked'
SEEDS=('5350648285461741569','5350648285461741570')
ENV_APPEND=['results-receiver.actions.githubusercontent.com','productionresultssa1.blob.core.windows.net']
EXPECTED_JOBS={'stable / test and specification','MSRV 1.88.0','supply chain','fuzz smoke'}
JOBS=(('stable / test and specification','ci-stable-job-v1.log.gz'),('supply chain','ci-supply-job-v1.log.gz'))
REPAIR_TESTS=('tests/p3_meeting.rs','tests/p3_required_conversions.rs','tests/p3_reference_slot_lifetimes.rs')
REPAIR_ADDITIONAL_FIXED={'tests/p3_meeting.rs':'c4655b9fc0bfbe32ee02ba87f9c7bb36898cd740f29145b50f8c997045bd2caf','tests/fixtures/p3-meeting/v8/Meet.class':'0e3b9f78d79e49e345ccb163a01eeaa0d21f9ed7ca0b340c24d7901cd4b1c04c','tests/fixtures/p3-meeting/Meet.java':'2c4b5055998affb2ca1294184bb78aedecf92dbc7335b85b45dbc6a4fc10c132','tests/fixtures/p3-required-conversions/RequiredConversions.java':'7f9705b16412242c732af0ad2cb7cfb61b02e0b26c95fe077d2797898339c66d','tests/fixtures/p3-required-conversions/v8/RequiredConversions.class':'d71eeaa5dc3eb66eafffda9e0c509eae43e3dde566b67df0006348352778d1e4','tests/fixtures/p3-reference-slot-lifetimes/negative/unknown-null/NullThenBuilder.class':'93acdfe27b312e2c98666a00dbfcb5f61299b03f62ac0dbd916b385232f260b3','tests/fixtures/p3-reference-slot-lifetimes/negative/unknown-null/NullThenBuilder.baseline.java':'f04aad4af575953e14ce91084bc5e3e5fc23b3656f42dd21280701a81ccb8e1e'}
REPAIR_ADDITIONAL_DYNAMIC=('tests/p3_required_conversions.rs','tests/p3_reference_slot_lifetimes.rs')
REPAIR_ADDITIONAL_PATHS=set(REPAIR_ADDITIONAL_FIXED)|set(REPAIR_ADDITIONAL_DYNAMIC)
def sha(b:bytes)->str:return hashlib.sha256(b).hexdigest()
def read(p:Path)->bytes:return p.read_bytes()
def git_blob(commit:str,path:str)->bytes:return subprocess.check_output(['git','show',f'{commit}:{path}'],cwd=ROOT)
def strip_log(raw:bytes)->str:
 text=raw.decode('utf-8',errors='replace')
 text=re.sub(r'(?m)^[^\t\n]+\t[^\t\n]+\t(?=\d{4}-\d\d-\d\dT)','',text)
 text=re.sub(r'(?:\x1b\[[0-9;]*m|\^\[\[[0-9;]*m)','',text)
 return re.sub(r'(?m)^\d{4}-\d\d-\d\d[T ][^ ]+Z\s+','',text)
def command_groups(lines,command):
 needle='##[group]Run '+command
 out=[]
 for i,line in enumerate(lines):
  if needle in line:
   end=next((j for j in range(i+1,len(lines)) if '##[group]Run ' in lines[j]),len(lines))
   out.append((i,'\n'.join(lines[i:end])))
 return out
def test_counts(text):return [tuple(map(int,x)) for x in SUMMARY_RE.findall(text)]
def test_functions(raw:bytes):return [(name,'ignore' in attrs) for attrs,name in TEST_RE.findall(raw.decode('utf-8'))]
def repair_additional_pins(product):
 out=dict(REPAIR_ADDITIONAL_FIXED)
 for path in REPAIR_ADDITIONAL_DYNAMIC:out[path]=sha(git_blob(product,path))
 if set(out)!=REPAIR_ADDITIONAL_PATHS or len(out)!=9:raise RuntimeError('repair-only pin path set changed')
 return out
def canonical_inputs(product:str):
 out=set()
 for rel in TEST_PATHS:
  text=git_blob(product,rel).decode('utf-8')
  for p in INCLUDE_RE.findall(text):
   joined=(Path(rel).parent/Path(p)).as_posix()
   norm=os.path.normpath(joined)
   if norm=='..' or norm.startswith('../'):raise RuntimeError('include escapes repo: '+rel+' -> '+p)
   out.add(norm)
 return out
def expected_clippy(workflow:str):
 start=workflow.index('      - name: Run Clippy');end=workflow.index('      - name: Run workspace tests',start)
 lint=re.findall(r'(?m)^\s+-A clippy::([a-z0-9_]+)\s*$',workflow[start:end])
 if len(lint)!=29 or len(set(lint))!=29:raise RuntimeError('CI Clippy allowlist is not the expected 29 distinct lints')
 return ['cargo','clippy','--workspace','--all-targets','--all-features','--locked','--',*sum((['-A','clippy::'+x] for x in lint),[]),'-D','warnings']
def verify_ci_api(capture,product,run_id):
 doc=capture['validated_run_summary']
 if doc.get('headSha')!=product or doc.get('status')!='completed' or doc.get('conclusion')!='success':raise RuntimeError('captured CI summary head/status mismatch')
 match=re.search(r'/actions/runs/(\d+)(?:$|[/?#])',doc.get('url',''))
 if not match or int(match.group(1))!=int(run_id):raise RuntimeError('CI run ID does not match summary URL')
 jobs=doc.get('jobs',[])
 if len(jobs)!=4 or {j.get('name') for j in jobs}!=EXPECTED_JOBS:raise RuntimeError('CI job set differs from the four workflow jobs')
 for j in jobs:
  if j.get('status')!='completed' or j.get('conclusion')!='success':raise RuntimeError('CI job incomplete/failed: '+str(j.get('name')))
  if any(s.get('status')!='completed' or s.get('conclusion')!='success' for s in j.get('steps',[])):raise RuntimeError('CI step incomplete/failed in '+j['name'])
 steps=sum(len(j['steps']) for j in jobs)
 if steps!=52:raise RuntimeError(f'CI workflow step count differs from expected 52: {steps}')
 return steps
def verify_capture(capture,product,run_id,typed_names,typed_count):
 if capture.get('schema')!='recover-proved-local-source-types-ci-capture-root-v1' or capture.get('status')!='captured':raise RuntimeError('capture identity/status mismatch')
 if capture.get('product_commit')!=product or capture.get('run_id')!=int(run_id):raise RuntimeError('capture input mismatch')
 if capture.get('typed_test_names')!=sorted(typed_names) or capture.get('expected_test_counts')!={'lib':337,'typed':typed_count}:raise RuntimeError('capture typed-test input mismatch')
 steps=verify_ci_api(capture,product,run_id)
 rows=capture.get('commands',[])
 if len(rows)!=3:raise RuntimeError('capture must contain summary and two job-log commands')
 expected_summary=['gh','run','view',str(run_id),'--json','headSha,status,conclusion,jobs,url']
 if rows[0].get('argv')!=expected_summary:raise RuntimeError('captured summary argv mismatch')
 ci=capture['validated_run_summary']; ids={j['name']:str(j['databaseId']) for j in ci['jobs']}
 expected=[expected_summary,['gh','run','view',str(run_id),'--job',ids[JOBS[0][0]],'--log'],['gh','run','view',str(run_id),'--job',ids[JOBS[1][0]],'--log']]
 evidence=CAPTURE
 for i,(row,argv) in enumerate(zip(rows,expected)):
  if row.get('argv')!=argv or row.get('cwd')!=str(ROOT) or row.get('timeout_seconds')!=35 or row.get('timed_out') or row.get('exit_code')!=0 or row.get('spawn_error') is not None:raise RuntimeError('capture command record invalid at '+str(i))
  env=row.get('environment_override',{})
  if set(env)!={'NO_PROXY','no_proxy'} or any(v.get('appended')!=ENV_APPEND or not re.fullmatch(r'[0-9a-f]{64}',v.get('final_value_sha256','')) for v in env.values()):raise RuntimeError('capture proxy record invalid')
  if i==0:
   for field,path in (('stdout','ci-run-v1.json'),('stderr','ci-run-v1.stderr.raw')):
    rec=row[field];raw=read(evidence/path)
    if len(raw)!=rec['bytes'] or sha(raw)!=rec['sha256']:raise RuntimeError('summary stream mismatch')
   if json.loads(read(evidence/'ci-run-v1.json'))!=ci:raise RuntimeError('summary raw differs from recorded API object')
  else:
   filename=JOBS[i-1][1]; packed=read(evidence/filename);rec=row['stdout']
   if len(packed)!=rec['bytes'] or sha(packed)!=rec['sha256']:raise RuntimeError('compressed CI log mismatch')
   raw=gzip.decompress(packed)
   if len(raw)!=rec['uncompressed_bytes'] or sha(raw)!=rec['raw_sha256'] or gzip.compress(raw,mtime=0)!=packed:raise RuntimeError('noncanonical or mismatched gzip payload')
   raw_rec=row['stdout_raw'];saved=read(evidence/(filename.removesuffix('.gz')+'.stdout.raw'))
   if saved!=raw or len(saved)!=raw_rec['bytes'] or sha(saved)!=raw_rec['sha256']:raise RuntimeError('raw CI log mismatch')
   err=read(evidence/(filename+'.stderr.raw'));er=row['stderr']
   if len(err)!=er['bytes'] or sha(err)!=er['sha256']:raise RuntimeError('CI log stderr mismatch')
 return {'jobs':4,'steps':steps,'summary_sha256':sha(read(evidence/'ci-run-v1.json')),
  'stable_gzip_sha256':sha(read(evidence/JOBS[0][1])),'stable_raw_sha256':sha(read(evidence/'ci-stable-job-v1.log.stdout.raw')),
  'supply_gzip_sha256':sha(read(evidence/JOBS[1][1])),'supply_raw_sha256':sha(read(evidence/'ci-supply-job-v1.log.stdout.raw'))}
def exact_binary_result(lines, binary_path, names, require_header=True):
 expected=set(names);count=len(expected)
 headers=[i for i,line in enumerate(lines) if re.search(r'\bRunning .*'+re.escape(binary_path),line)]
 if require_header and len(headers)!=1:raise RuntimeError('integration binary header missing/duplicated: '+binary_path)
 if not require_header and headers:raise RuntimeError('local stdout unexpectedly contains Cargo target headers: '+binary_path)
 header_at=headers[0] if headers else -1
 matches=[]
 for i,line in enumerate(lines):
  if not re.fullmatch(rf'running {count} tests?',line.strip()):continue
  if i<=header_at:continue
  end=next((j for j in range(i+1,len(lines)) if re.search(r'test result: (?:ok\.|FAILED)',lines[j])),None)
  if end is None:continue
  block='\n'.join(lines[i:end+1]);summary=test_counts(block)
  outcomes=re.findall(r'(?m)^test ([A-Za-z_][A-Za-z0-9_]*) \.\.\. (ok|ignored|FAILED)$',block)
  if summary==[(count,0,0)] and len(outcomes)==count and {n for n,_ in outcomes}==expected and all(v=='ok' for _,v in outcomes):
   matches.append((summary[0],outcomes))
 if len(matches)!=1:raise RuntimeError(f'{binary_path} does not have one exact {count}/0/0 raw block')
 return list(matches[0][0]), sorted(expected)
def verify_workspace(product,typed_names,typed_count):
 workflow=git_blob(product,'.github/workflows/ci.yml').decode()
 seeds=re.findall(r'PROPTEST_RNG_SEED:\s*"(\d+)"',workflow)
 if tuple(seeds)!=SEEDS or workflow.count('run: '+WORKSPACE_COMMAND)!=2:raise RuntimeError('workflow no longer defines the exact two fixed workspace seed runs')
 raw=gzip.decompress(read(CAPTURE/'ci-stable-job-v1.log.gz'));text=strip_log(raw);lines=text.splitlines()
 groups=command_groups(lines,WORKSPACE_COMMAND)
 if len(groups)!=2:raise RuntimeError('stable job does not contain exactly two full workspace runs')
 rows=[]
 for (_,group),seed in zip(groups,seeds):
  if seed not in group or any(other in group for other in seeds if other!=seed):raise RuntimeError('seed run log boundary is wrong')
  if re.search(r'test result: FAILED|error: test failed',group):raise RuntimeError('workspace log contains a test failure')
  counts=test_counts(group)
  if not counts or any(f for _,f,_ in counts):raise RuntimeError('workspace run lacks successful summaries or has failures')
  lib_marks=[(i,int(m.group(1))) for i,line in enumerate(group.splitlines()) if (m:=re.search(r'\brunning (\d+) tests?\b',line)) and int(m.group(1))==337]
  if len(lib_marks)!=1:raise RuntimeError('expected one java library running-337 marker')
  ls=group.splitlines();li=lib_marks[0][0]
  header=next((ls[j] for j in range(li-1,-1,-1) if 'Running' in ls[j]),'')
  if 'src/lib.rs' not in header or 'target/debug/deps/jarde_java-' not in header:raise RuntimeError('337 test marker is not jarde-java library')
  lcounts=test_counts('\n'.join(ls[li+1:]))
  if not lcounts or lcounts[0]!=(337,0,0):raise RuntimeError('jarde-java library summary is not 337/0/0')
  typed_lines=[(i,line) for i,line in enumerate(ls) if re.search(r'Running .*cf12_proved_local_source_types\.rs',line)]
  if len(typed_lines)!=1:raise RuntimeError('typed local source test binary header missing/duplicated')
  start=typed_lines[0][0];end=next((j for j in range(start+1,len(ls)) if '     Running ' in ls[j] or '   Doc-tests ' in ls[j]),len(ls))
  block='\n'.join(ls[start:end]);binary_counts=test_counts(block)
  if binary_counts!=[(typed_count,0,0)]:raise RuntimeError(f'typed test summary mismatch: {binary_counts}')
  outcomes=re.findall(r'(?m)^test ([A-Za-z0-9_:]+) \.\.\. (ok|ignored|FAILED)$',block)
  if len(outcomes)!=typed_count or {n for n,_ in outcomes}!=set(typed_names) or any(v!='ok' for _,v in outcomes):raise RuntimeError('typed test names/status differ from exact expected set')
  meeting_names=[name for name,ignored in test_functions(git_blob(product,REPAIR_TESTS[0])) if not ignored]
  if len(meeting_names)!=6 or len(set(meeting_names))!=6:raise RuntimeError('meeting integration source does not define six distinct active tests')
  meeting_headers=[(i,line) for i,line in enumerate(ls) if re.search(r'\bRunning .*tests/p3_meeting\.rs',line)]
  if len(meeting_headers)!=1:raise RuntimeError('meeting integration binary header missing/duplicated in seed')
  # Cargo may print later `Running` headers before a subprocess flushes its test output.
  # Tie the unique test-name set to the unique 6/0/0 result block, rather than truncating at
  # the next executable header and risking attribution to an adjacent binary.
  running_blocks=[]
  for ri,line in enumerate(ls):
   if not re.search(r'(?m)^running 6 tests$',line): continue
   summary_at=next((j for j in range(ri+1,len(ls)) if re.search(r'test result: ok\.|test result: FAILED',ls[j])),None)
   if summary_at is None: continue
   block='\n'.join(ls[ri:summary_at+1]); rows_out=re.findall(r'(?m)^test ([A-Za-z_][A-Za-z0-9_]*) \.\.\. (ok|ignored|FAILED)$',block)
   totals=test_counts(block)
   if totals==[(6,0,0)] and len(rows_out)==6 and {name for name,_ in rows_out}==set(meeting_names) and all(result=='ok' for _,result in rows_out):
    running_blocks.append((totals[0],rows_out))
  if len(running_blocks)!=1:raise RuntimeError(f'meeting binary does not have one exact six-test successful raw block for seed {seed}')
  mcount,moutcomes=running_blocks[0]
  repair_binary_results={}
  for test_path in REPAIR_TESTS:
   raw_test=git_blob(product,test_path);all_tests=test_functions(raw_test)
   active=[name for name,ignored in all_tests if not ignored]
   if not active or len(set(active))!=len(active) or any(ignored for _,ignored in all_tests):raise RuntimeError('repair integration source has ignored/duplicate/empty test set: '+test_path)
   rel_binary=test_path
   result_counts,result_names=exact_binary_result(ls,rel_binary,active)
   repair_binary_results[test_path]={'counts':result_counts,'test_names':result_names}
  aggregate=(sum(p for p,_,_ in counts),sum(f for _,f,_ in counts),sum(i for _,_,i in counts))
  rows.append({'seed':seed,'library_counts':[337,0,0],'library_header':header,'typed_counts':list(binary_counts[0]),'typed_test_names':sorted(typed_names),
   'meeting_counts':list(mcount),'meeting_test_names':sorted(meeting_names),'repair_binary_results':repair_binary_results,'workspace_result_records':len(counts),'workspace_totals':list(aggregate)})
 if rows[0]['workspace_result_records']!=rows[1]['workspace_result_records'] or rows[0]['workspace_totals']!=rows[1]['workspace_totals']:
  raise RuntimeError('two fixed seed workspace runs disagree; no total count is assumed')
 return rows
def verify_repair(execution_path,execution_sha,product,meta_path):
 if not execution_path.is_file() or sha(read(execution_path))!=execution_sha:raise RuntimeError('repair execution SHA mismatch')
 repair=json.loads(read(execution_path))
 if repair.get('schema')!='typed-ci-repair-local-root-v1' or repair.get('status')!='passed':raise RuntimeError('repair focused gate is not a completed passing record')
 md=json.loads(read(meta_path));baseline={}
 for group in ('candidate_sources','test_sources','canonical_files'):
  for path,digest in md[group].items():
   if path in baseline and baseline[path]!=digest:raise RuntimeError('baseline pin groups conflict: '+path)
   baseline[path]=digest
 if len(baseline)!=76:raise RuntimeError('original typed closure no longer has 76 unique pinned paths')
 if repair.get('source_pins')!=baseline:raise RuntimeError('repair source pins differ from the exact original typed closure')
 additional_pins=repair_additional_pins(product)
 if repair.get('additional_test_pins')!=additional_pins:raise RuntimeError('repair additional source pins differ from the exact nine-path test/fixture set')
 if set(additional_pins)&set(baseline):raise RuntimeError('repair-only pins were folded into the historical typed closure')
 if repair.get('pins_unchanged') is not True:raise RuntimeError('repair runner did not verify unchanged pins')
 for path,digest in {**baseline,**additional_pins}.items():
  if sha(read(ROOT/path))!=digest or sha(git_blob(product,path))!=digest:raise RuntimeError('repair Git/live pin mismatch: '+path)
 test_names={}
 for test_path in REPAIR_TESTS:
  found=test_functions(git_blob(product,test_path));active=[name for name,ignored in found if not ignored]
  if not active or len(set(active))!=len(active) or any(ignored for _,ignored in found):raise RuntimeError('repair source contains ignored, duplicate, or empty active test set: '+test_path)
  test_names[test_path]=active
 commands=repair.get('commands',[])
 expected=[['cargo','fmt','--all','--','--check'],['cargo','test','--test','p3_meeting','--test','p3_required_conversions','--test','p3_reference_slot_lifetimes','--locked']]
 if len(commands)!=2:raise RuntimeError('repair gate must contain exactly fmt and the three-suite focused test command')
 streams_seen=0;repair_summaries=[]
 for index,(row,argv) in enumerate(zip(commands,expected)):
  if row.get('index')!=index or row.get('argv')!=argv or row.get('cwd')!=str(ROOT) or row.get('exit_code')!=0 or row.get('guard_stop') is not None:raise RuntimeError('repair command sequence/status mismatch at '+str(index))
  if row.get('peak_target_bytes',0)>1024**3 or row.get('free_bytes_after',0)<5*1024**3:raise RuntimeError('repair command resource telemetry crossed guard')
  if row.get('env_overrides')!={'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always'}:raise RuntimeError('repair command environment differs from pinned guard')
  streams=row.get('streams',{})
  if set(streams)!={'stdout','stderr'}:raise RuntimeError('repair raw streams missing')
  stream_data={}
  for label,record in streams.items():
   path=Path(record['path']);data=read(path)
   if not path.is_absolute() or len(data)!=record['bytes'] or sha(data)!=record['sha256']:raise RuntimeError(f'repair {index} {label} raw mismatch')
   stream_data[label]=data;streams_seen+=1
  if index==1:
   stdout=stream_data['stdout'].decode('utf-8',errors='replace');lines=stdout.splitlines()
   stderr_lines=strip_log(stream_data['stderr']).splitlines()
   headers=[line for line in stderr_lines if re.search(r'(?m)^\s*Running\s+tests/[^ ]+\.rs\s+\(target/debug/deps/[^)]+\)$',line)]
   expected_headers={path for path in REPAIR_TESTS}
   observed_headers=[]
   for line in headers:
    match=re.search(r'\bRunning\s+(tests/[^ ]+\.rs)\s+\(target/debug/deps/[^)]+\)$',line)
    if match:observed_headers.append(match.group(1))
   if len(observed_headers)!=3 or set(observed_headers)!=expected_headers or len(set(observed_headers))!=3:
    raise RuntimeError('repair stderr must contain exactly one Cargo Running header for each of the three test binaries')
   summaries=[]
   for test_path,active in test_names.items():
    counts,names=exact_binary_result(lines,test_path,active,require_header=False)
    summaries.append(counts);repair_summaries.append({'binary':test_path,'counts':counts,'test_names':names})
   check=row.get('test_summary_check')
   if check is not None:
    if check.get('ok') is not True or check.get('required') is not True:raise RuntimeError('repair runner summary check is present but not required and successful')
    actual=check.get('actual')
    if actual is not None and sorted(tuple(x) for x in actual)!=sorted(tuple(x) for x in summaries):raise RuntimeError('repair recorded summaries differ from the three raw binary results')
    expected_summaries=check.get('expected')
    if expected_summaries is not None and sorted(tuple(x) for x in expected_summaries)!=sorted(tuple(x) for x in summaries):raise RuntimeError('repair recorded expected summaries differ from source-derived actual results')
 return {'execution_path':str(execution_path),'execution_sha256':execution_sha,'schema':repair['schema'],'status':repair['status'],
  'focused_commands':len(commands),'raw_streams_verified':streams_seen,'test_summaries':repair_summaries,
  'additional_test_pins':additional_pins,'baseline_pin_count':len(baseline),'baseline_closure_unchanged':True}

def verify_pins(product,source_base,meta_path,meta_sha,cli_path,cli_sha,build_path,build_sha,runner_path,runner_sha,typed_names,typed_count):
 for p,h in ((meta_path,meta_sha),(cli_path,cli_sha),(build_path,build_sha),(runner_path,runner_sha)):
  if not p.is_file() or sha(read(p))!=h:raise RuntimeError('input SHA mismatch: '+str(p))
 md=json.loads(read(meta_path));b=json.loads(read(build_path))
 if md.get('schema')!='recover-proved-local-source-types-candidate-cli-root-v1' or md.get('cli_path')!=str(cli_path) or md.get('cli_sha256')!=cli_sha or md.get('metadata_path')!=str(meta_path) or md.get('build_result_sha256')!=build_sha or md.get('source_commit_base')!=source_base:raise RuntimeError('typed CLI metadata binding mismatch')
 if stat.S_IMODE(cli_path.stat().st_mode)!=0o555:raise RuntimeError('candidate CLI is not frozen 0555')
 if md.get('required_origin_tests')!=sorted(typed_names) or md.get('expected_origin_test_count')!=typed_count or md.get('expected_library_test_count')!=337:raise RuntimeError('metadata typed test/lib counts mismatch')
 if b.get('schema')!='recover-proved-local-source-types-validation-build-root-v1' or b.get('status')!='validation-passed-cli-frozen' or b.get('source_commit_base_expected')!=source_base:raise RuntimeError('typed validation build status/schema/base mismatch')
 if b.get('validation_runner')!={'path':str(runner_path),'sha256':runner_sha} or md.get('validation_runner')!=b['validation_runner']:raise RuntimeError('typed runner identity mismatch')
 if md.get('guarded_runner_template')!=b.get('guarded_runner_template'):raise RuntimeError('typed guard identity mismatch')
 if b.get('expected_origin_test_count')!=typed_count or b.get('required_origin_tests')!=sorted(typed_names) or b.get('expected_library_test_count')!=337 or b.get('uncommitted_local_source_types_product') is not True:raise RuntimeError('typed build inputs mismatch')
 if b.get('freeze',{}).get('cli_path')!=str(cli_path) or b['freeze'].get('cli_sha256')!=cli_sha or b['freeze'].get('metadata_path')!=str(meta_path) or b['freeze'].get('source_commit_base')!=source_base:raise RuntimeError('frozen CLI block mismatch')
 expected_tests=set(TEST_PATHS)
 if set(md.get('candidate_sources',{}))!=PRODUCT_PATHS or set(md.get('test_sources',{}))!=expected_tests:raise RuntimeError('pinned source path set mismatch')
 can=canonical_inputs(product)
 if set(md.get('canonical_files',{}))!=can:raise RuntimeError('canonical include path closure mismatch')
 before=b.get('preflight',{}).get('source_pins_before');after=b.get('preflight',{}).get('source_pins_after')
 expected={'candidate_sources':md['candidate_sources'],'test_sources':md['test_sources'],'canonical_files':md['canonical_files']}
 if before!=after or after!=expected:raise RuntimeError('build before/after pins differ from candidate metadata')
 if b['freeze'].get('product_paths')!=sorted(PRODUCT_PATHS) or b['freeze'].get('test_paths')!=sorted(expected_tests) or b['freeze'].get('canonical_paths')!=sorted(can):raise RuntimeError('freeze path sets do not match typed CI adapter')
 allpins=[]
 for cat,paths in (('candidate_sources',PRODUCT_PATHS),('test_sources',expected_tests),('canonical_files',can)):
  for path in sorted(paths):
   digest=md[cat][path];submitted=git_blob(product,path)
   if sha(submitted)!=digest or sha(read(ROOT/path))!=digest:raise RuntimeError(f'Git product blob/live source differs from frozen {cat}: {path}')
   allpins.append((cat,path,digest))
 # Verify the exact local validation command sequence, summaries, and every saved raw stream.
 workflow=git_blob(product,'.github/workflows/ci.yml').decode();clippy=expected_clippy(workflow)
 expected=[['cargo','fmt','--all','--','--check'],clippy,['cargo','test','-p','jarde-java','--lib','--locked'],
  ['cargo','test','-p','jarde-java','--test','cf12_proved_local_source_types','--locked','--','--nocapture'],
  ['cargo','test','-p','jarde','--test','p3_declarations','--locked','--','--nocapture'],
  ['cargo','test','-p','jarde','--test','p3_parameter_slots','--locked','--','--nocapture'],
  ['cargo','test','-p','jarde-java','--test','p3_proved_boolean_conditional_returns','--locked','--','--nocapture'],
  ['cargo','test','-p','jarde','--test','p3_guard','--locked','--','--nocapture'],
  ['cargo','test','-p','jarde','--test','p3_popped_static_qualifier','--locked','--','--nocapture'],
  ['cargo','test','-p','jarde','--test','p3_invocation_arguments','--locked','--','--nocapture'],
  ['cargo','test','-p','jarde','--test','p3_array_slot_retype_locals','--locked','--','--nocapture'],
  ['cargo','test','-p','jarde-java','--test','p3_patterns','--locked','--','--nocapture'],
  ['cargo','test','-p','jarde-reader','--lib','--locked'],['cargo','build','-p','jarde-cli','--locked']]
 commands=b.get('commands',[])
 if len(commands)!=len(expected):raise RuntimeError('build command count mismatch')
 summaries={2:(337,0,0),12:(178,0,0)}
 targets={3:('crates/jarde-java/tests/cf12_proved_local_source_types.rs',typed_names),
  4:('tests/p3_declarations.rs',None),5:('tests/p3_parameter_slots.rs',None),
  6:('crates/jarde-java/tests/p3_proved_boolean_conditional_returns.rs',None),7:('tests/p3_guard.rs',None),
  8:('tests/p3_popped_static_qualifier.rs',None),9:('tests/p3_invocation_arguments.rs',None),
  10:('tests/p3_array_slot_retype_locals.rs',None),11:('crates/jarde-java/tests/p3_patterns.rs',None)}
 for idx,(row,argv) in enumerate(zip(commands,expected)):
  if row.get('index')!=idx or row.get('argv')!=argv or row.get('cwd')!=str(ROOT) or row.get('exit_code')!=0 or row.get('guard_stop') is not None:raise RuntimeError(f'build command record mismatch at {idx}')
  streams=row.get('streams',{})
  if set(streams)!={'stdout','stderr'}:raise RuntimeError('missing build raw stream record')
  raw={}
  for name,rec in streams.items():
   path=Path(rec['path']);path=path if path.is_absolute() else ROOT/path;data=read(path)
   if len(data)!=rec['bytes'] or sha(data)!=rec['sha256']:raise RuntimeError(f'build raw stream mismatch {idx}/{name}')
   raw[name]=data
  if idx in summaries or idx in targets:
   actual=[tuple(map(int,x)) for x in SUMMARY_RE.findall(raw['stdout'].decode('utf-8',errors='replace'))]
   if idx in summaries:
    if actual!=[summaries[idx]]:raise RuntimeError(f'fixed library summary mismatch at {idx}: {actual}')
   else:
    source,names=targets[idx];tests=test_functions(git_blob(product,source))
    expected_row=(len([1 for _,ign in tests if not ign]),0,len([1 for _,ign in tests if ign]))
    if actual!=[expected_row]:raise RuntimeError(f'source-derived test count mismatch at {idx}: {actual} != {expected_row}')
    if names is not None:
     outcomes=re.findall(r'(?m)^test ([A-Za-z_][A-Za-z0-9_]*) \.\.\. (ok|ignored|FAILED)$',raw['stdout'].decode('utf-8',errors='replace'))
     if {n for n,_ in outcomes}!=set(names) or any(v!='ok' for _,v in outcomes):raise RuntimeError('local typed exact test names not all successful')
  check=row.get('test_summary_check')
  if idx in summaries or idx in targets:
   if not check or not check.get('ok') or check.get('required') is not True:raise RuntimeError(f'build test summary check failed at {idx}')
  else:
   if not check or check.get('expected') is not None or check.get('actual')!=[] or check.get('required') is not False or check.get('ok') is not True or check.get('failure_reason') is not None:raise RuntimeError(f'non-test command summary record mismatch at {idx}')
 if b.get('guards')!={'minimum_free_bytes':5*1024**3,'maximum_target_bytes':1024**3}:raise RuntimeError('build guard limits changed')
 if b.get('environment_overrides')!={'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always'}:raise RuntimeError('build environment not pinned')
 return {'candidate_cli_sha256':cli_sha,'metadata_sha256':meta_sha,'build_sha256':build_sha,'runner_sha256':runner_sha,
  'source_base':source_base,'product_pin_entries_verified':len(allpins),'build_commands':len(commands),
  'build_raw_streams_verified':2*len(commands),'typed_test_names':sorted(typed_names)}
def main():
 ap=argparse.ArgumentParser(description=__doc__)
 ap.add_argument('--product-commit',required=True);ap.add_argument('--run-id',required=True);ap.add_argument('--source-base',required=True)
 ap.add_argument('--metadata',type=Path,required=True);ap.add_argument('--metadata-sha256',required=True)
 ap.add_argument('--cli',type=Path,required=True);ap.add_argument('--cli-sha256',required=True)
 ap.add_argument('--build-execution',type=Path,required=True);ap.add_argument('--build-sha256',required=True)
 ap.add_argument('--validation-runner',type=Path,required=True);ap.add_argument('--runner-sha256',required=True)
 ap.add_argument('--typed-test-count',type=int,required=True);ap.add_argument('--typed-test-name',action='append',required=True)
 ap.add_argument('--repair-execution',type=Path,required=True);ap.add_argument('--repair-sha256',required=True)
 ap.add_argument('--capture',type=Path,default=CAPTURE);ap.add_argument('--acceptance',type=Path,default=ACCEPT)
 a=ap.parse_args()
 for value in (a.product_commit,a.source_base):
  if not re.fullmatch(r'[0-9a-f]{40}',value):raise SystemExit('commit values must be lowercase 40-hex')
 if not a.run_id.isdecimal():raise SystemExit('run ID must be decimal')
 for value in (a.metadata_sha256,a.cli_sha256,a.build_sha256,a.runner_sha256,a.repair_sha256):
  if not re.fullmatch(r'[0-9a-f]{64}',value):raise SystemExit('SHA-256 inputs must be lowercase 64-hex')
 names=set(a.typed_test_name)
 if a.typed_test_count<=0 or len(names)!=a.typed_test_count or len(names)!=len(a.typed_test_name):raise SystemExit('typed count and explicit distinct names must agree')
 if a.capture.resolve()!=CAPTURE.resolve():raise SystemExit('capture path must be the reviewed private capture directory')
 if a.acceptance.exists():raise SystemExit('refusing to overwrite acceptance output')
 capture=json.loads(read(CAPTURE/'capture-execution-root-v1.json'))
 ci_summary=verify_capture(capture,a.product_commit,a.run_id,names,a.typed_test_count)
 workspace=verify_workspace(a.product_commit,names,a.typed_test_count)
 build=verify_pins(a.product_commit,a.source_base,a.metadata.resolve(),a.metadata_sha256,a.cli.resolve(),a.cli_sha256,
  a.build_execution.resolve(),a.build_sha256,a.validation_runner.resolve(),a.runner_sha256,names,a.typed_test_count)
 repair=verify_repair(a.repair_execution.resolve(),a.repair_sha256,a.product_commit,a.metadata.resolve())
 result={'schema':'recover-proved-local-source-types-ci-product-acceptance-root-v4','status':'accepted',
  'product_commit':a.product_commit,'run_id':int(a.run_id),'source_commit_base':a.source_base,
  'ci':{**ci_summary,'seed_runs':workspace,'aggregate_counts_predeclared':False},'frozen_cli_and_build':build,'repair_gate':repair,
  'acceptance_scope':'CI API and all captured job steps completed successfully; both exact fixed-seed workspace runs were parsed from raw stable logs with no failed test records and equal observed totals; historical typed tests, p3_meeting tests, and the added conversion/reference-lifetime suites were derived from product sources and passed in each seed; 337 jarde-java lib tests remained intact; frozen typed metadata/build pins and the separate local repair gate plus its nine additional pins were checked against Git blobs and live files. Repair-only pins are not added to the historical typed build closure.'}
 a.acceptance.parent.mkdir(parents=True,exist_ok=True)
 with a.acceptance.open('x',encoding='utf-8') as f:f.write(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
 print(json.dumps({'status':'accepted','product_commit':a.product_commit,'run_id':int(a.run_id),'jobs':ci_summary['jobs'],'steps':ci_summary['steps'],'seed_totals':[r['workspace_totals'] for r in workspace],'typed_tests':sorted(names),'meeting_tests':workspace[0]['meeting_test_names'],'repair_test_summaries':repair['test_summaries'],'repair_commands':repair['focused_commands']}))
if __name__=='__main__':main()
