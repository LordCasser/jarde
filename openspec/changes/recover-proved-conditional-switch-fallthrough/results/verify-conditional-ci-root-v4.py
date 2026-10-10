#!/usr/bin/env python3
"""Verify this conditional-switch product's exact CI run against frozen build/replay and a fresh local workspace baseline."""
from __future__ import annotations
import argparse, gzip, hashlib, json, os, re, stat, subprocess, sys
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
CHANGE=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough'
SCAN=CHANGE/'results/target-size-scan-root-v1.py'
SCAN_SHA='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
WORKSPACE_RUNNER=Path('/private/tmp/jarde-run-conditional-workspace-batches-root-v5.py')
WORKSPACE_RUNNER_SHA='15fb42dd044f2371fa8d0c75010f9b87b3474ad4a20c5f9d1007f15af299c76f'
RECORDER_SHA='4618af5287d895254fe08f359c65b9fe94e601f8ce3ba8b4a44e1ac5e2ba9869'
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SOURCE_BASE='2d70da515896c25ce022b8c28f4935ff2e105026'
CAPTURE_SCHEMA='recover-proved-conditional-switch-fallthrough-ci-capture-root-v2'
ACCEPT_SCHEMA='recover-proved-conditional-switch-fallthrough-ci-product-acceptance-root-v1'
EXPECTED_JOBS={'stable / test and specification','MSRV 1.88.0','supply chain','fuzz smoke'}
CI_SEEDS_RE=re.compile(r'PROPTEST_RNG_SEED:\s*"(\d+)"')
WORKSPACE_CMD='cargo test --workspace --all-targets --all-features --locked'
ANSI_RE=re.compile(r'(?:\x1b\[[0-9;]*m|\^\[\[[0-9;]*m)')
RUNNING_RE=re.compile(r'^\s*Running\s+(.+?)\s+\((target/[^)]+)\)\s*$')
RUNNING_TESTS_RE=re.compile(r'^\s*running (\d+) tests?\s*$')
OUTCOME_RE=re.compile(r'^test (.+) \.\.\. (ok|ignored(?:, .*)?|FAILED)$')
SUMMARY_RE=re.compile(r'^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out; finished in ([0-9]+(?:\.[0-9]+)?s)$')
INCLUDE_RE=re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')
REQUIRED_TESTS={
 'crates/jarde-java/src/region.rs':(
  'region::tests::cf12_switch_certificate_accepts_real_paths_and_rejects_edge_row_variants',
  'region::tests::real_multiple_exit_class_refuses_conditional_switch_certificate',
  'region::tests::real_nonadjacent_class_proves_one_map_entry',
  'region::tests::real_inner_loop_cycle_refuses_switch_certificate',
  'region::tests::terminal_case_unknown_terminal_fact_refuses_certificate',
  'region::tests::real_legacy_clone_target_with_different_path_refuses_switch_certificate'),
 'crates/jarde-java/src/build.rs':('build::tests::cf12_switch_break_builder_consumes_only_the_reader_leaf_it_proves',),
 'crates/jarde-java/tests/cf12_proved_local_source_types.rs':(
  'cf12_charat_stored_value_drives_char_source_type_and_full_origins',
  'cf12_null_first_direct_string_writes_drive_string_source_type_and_full_origins',
  'cf12_recovery_budget_stop_and_cancellation_publish_no_partial_source',
  'typed_boundary_four_char_seeds_have_specific_types_and_full_origins_in_both_debug_profiles',
  'typed_boundary_range_arithmetic_and_input_copy_remain_int_in_both_debug_profiles',
  'typed_boundary_null_first_exact_string_stays_string_in_both_debug_profiles',
  'typed_boundary_mixed_all_null_and_unknown_reference_stay_object_in_both_debug_profiles',
  'typed_boundary_real_slot_conflict_is_preserved_as_the_exact_fallback_in_default_and_all',
  'typed_boundary_proof_budget_stops_before_publication_at_observed_sites'),
 'crates/jarde-java/tests/p3_conditional_switch_fallthrough.rs':(
  'cf12_conditional_switch_keeps_exact_case_and_exit_sources',
  'cf12_recovery_with_pre_cancelled_public_budget_publishes_no_partial_artifact',
  'cf12_switch_proof_stops_at_observed_analysis_step_boundaries_without_an_artifact'),
 'crates/jarde-java/tests/p3_conditional_switch_boundary_rejection.rs':(
  'nonadjacent_conditional_switch_is_refused_by_the_public_caller',
  'pre_cancelled_public_recovery_publishes_no_partial_artifact'),
 'crates/jarde-java/tests/p3_conditional_switch_scope_boundaries.rs':(
  'loop_nested_switch_and_caught_edges_remain_conservative',
  'terminal_return_and_throw_keep_exact_text_and_physical_origins')}

def sha(raw:bytes)->str:return hashlib.sha256(raw).hexdigest()
def sha_file(path:Path)->str:return sha(path.read_bytes())
def need(ok,msg):
    if not ok: raise ValueError(msg)
def read(path):return Path(path).read_bytes()
def git_blob(commit,path):return subprocess.check_output(['git','show',f'{commit}:{path}'],cwd=ROOT)
def git_tree(commit):
    raw=subprocess.check_output(['git','ls-tree','-rz','--full-tree',commit],cwd=ROOT)
    result={}
    for item in raw.split(b'\0'):
        if not item:continue
        meta,rel=item.split(b'\t',1); mode,kind,oid=meta.decode().split(' ')
        if kind=='blob':result[rel.decode('utf-8')]=oid
    return result
def blob_sha256s(commit,paths):
    tree=git_tree(commit); resolved={}
    for path in paths:
        need(path in tree,'product commit has no pinned blob: '+path)
        resolved[path]=tree[path]
    out={}
    items=list(resolved.items())
    for start in range(0,len(items),100):
        part=items[start:start+100]
        proc=subprocess.run(['git','cat-file','--batch'],cwd=ROOT,input=(''.join(oid+'\n' for _,oid in part)).encode(),capture_output=True,check=True)
        raw=proc.stdout; pos=0
        for path,oid in part:
            end=raw.find(b'\n',pos); header=raw[pos:end].decode('ascii').split()
            need(len(header)==3 and header[0]==oid and header[1]=='blob','git cat-file returned a non-blob pin: '+path)
            size=int(header[2]); begin=end+1; payload=raw[begin:begin+size]
            need(len(payload)==size and raw[begin+size:begin+size+1]==b'\n','truncated git cat-file response: '+path)
            out[path]=sha(payload);pos=begin+size+1
        need(pos==len(raw),'unparsed bytes in git cat-file batch')
    return out

def verified_stream(rec,base=None):
    p=Path(rec['path'])
    if not p.is_absolute():
        need(base is not None,'relative stream path without evidence directory')
        p=Path(base)/p
    data=read(p)
    need(len(data)==rec['bytes'] and sha(data)==rec['sha256'],'raw command stream mismatch: '+str(p))
    return data

def clean(text):return ANSI_RE.sub('',text)
def parse_outcome_block(lines,start):
    m=RUNNING_TESTS_RE.fullmatch(clean(lines[start]))
    if not m:return None
    declared=int(m.group(1)); outcomes=[]; summary=None; end=None
    for j in range(start+1,len(lines)):
        line=clean(lines[j]).strip()
        sm=SUMMARY_RE.fullmatch(line)
        if sm:
            need(summary is None,'test block contains duplicate result summary')
            status=sm.group(1); summary=tuple(map(int,sm.groups()[1:4])); end=j
            measured=int(sm.group(5)); filtered=int(sm.group(6)); duration=sm.group(7)
            need((status=='ok')==(summary[1]==0),'test result status conflicts with failed count')
            break
        om=OUTCOME_RE.fullmatch(line)
        if om:
            state=om.group(2)
            outcomes.append((om.group(1), state))
        elif line.startswith('test result: FAILED') or line.startswith('test result: ok.'):
            raise ValueError('unrecognized test result summary line: '+line)
        elif line.startswith('test '):
            raise ValueError('unrecognized named test outcome line: '+line)
    need(summary is not None,'running test block has no result summary')
    need(declared==len(outcomes),'running N differs from number of named outcomes')
    need(len({name for name,_ in outcomes})==len(outcomes),'duplicate named test outcome inside target block')
    counts=(sum(v=='ok' for _,v in outcomes),sum(v=='FAILED' for _,v in outcomes),sum(v.startswith('ignored') for _,v in outcomes))
    need(counts==summary,'named outcomes differ from test result summary')
    return {'counts':counts,'outcomes':tuple(outcomes),'summary':summary,
            'measured':measured,'filtered':filtered,'duration':duration,'end':end}

def stdout_blocks_cargo_json(raw):
    plain=[]
    for line in raw.decode('utf-8',errors='replace').splitlines():
        if line.startswith('{'):
            try:
                obj=json.loads(line)
                if isinstance(obj,dict) and isinstance(obj.get('reason'),str):continue
            except json.JSONDecodeError:pass
        plain.append(line)
    lines=plain; blocks=[];i=0
    while i<len(lines):
        if RUNNING_TESTS_RE.fullmatch(clean(lines[i])):
            block=parse_outcome_block(lines,i);blocks.append(block);i=block['end']+1
        else:i+=1
    marker_count=sum(bool(RUNNING_TESTS_RE.fullmatch(clean(line))) for line in lines)
    result_count=sum(clean(line).strip().startswith(('test result: ok.','test result: FAILED.')) for line in lines)
    need(marker_count==len(blocks) and result_count==len(blocks),'unparsed running/result line in test output')
    return blocks

def stderr_headers(raw):
    rows=[]
    for line in clean(raw.decode('utf-8',errors='replace')).splitlines():
        m=RUNNING_RE.fullmatch(line)
        if m: rows.append({'display':m.group(1).strip(),'executable':m.group(2).strip()})
    return rows

def target_key(pkg_id,target):
    return (pkg_id,target['name'],target['kind'][0])
def metadata_targets(metadata):
    by_id={p['id']:p for p in metadata['packages']}; by_named={}
    rows=[]
    for p in metadata['packages']:
        if p['id'] not in metadata['workspace_members']:continue
        for t in p.get('targets',[]):
            key=(p['name'],t['name'],t['kind'][0])
            need(key not in by_named,'ambiguous package/name/kind Cargo target identity: '+repr(key))
            package_dir=Path(p['manifest_path']).resolve().parent
            src_path=Path(t['src_path']).resolve()
            by_named[key]={'package_id':p['id'],'package':p['name'],'name':t['name'],
                'kind':t['kind'][0],'source_path':src_path.relative_to(ROOT.resolve()).as_posix(),
                'src_path':str(src_path),'package_dir':str(package_dir),
                'header_path':src_path.relative_to(package_dir).as_posix()}
            rows.append(key)
    return by_id,by_named,rows

def parse_artifacts(raw,by_id,by_named):
    artifacts=[]
    for line in raw.decode('utf-8',errors='replace').splitlines():
        if not line.startswith('{'):continue
        try:obj=json.loads(line)
        except json.JSONDecodeError:continue
        if obj.get('reason')!='compiler-artifact' or not obj.get('executable'):continue
        pkg=by_id.get(obj.get('package_id'));t=obj.get('target',{})
        if pkg is None:continue
        key=(pkg['name'],t.get('name'),(t.get('kind') or [''])[0])
        if key not in by_named:continue
        artifacts.append({'key':key,'package_id':obj['package_id'],'name':t['name'],
                          'kind':(t.get('kind') or [''])[0],
                          'src_path':str(Path(t['src_path']).resolve()),
                          'executable':str(Path(obj['executable']).resolve()),
                          'profile_test':obj.get('profile',{}).get('test')})
    return artifacts

def local_command_observation(row,by_id,by_named):
    stdout=verified_stream(row['streams']['stdout']);stderr=verified_stream(row['streams']['stderr'])
    arts=parse_artifacts(stdout,by_id,by_named)
    headers=stderr_headers(stderr);blocks=stdout_blocks_cargo_json(stdout)
    need(len(headers)==len(blocks),'local Cargo Running headers and stdout result blocks differ')
    artifact_by_exe={}
    for item in arts:
        exe=item['executable']
        need(exe not in artifact_by_exe,'duplicate compiler-artifact executable identity: '+exe)
        artifact_by_exe[exe]=item
    observed=[]
    for header,block in zip(headers,blocks):
        suffix=header['executable']
        matches=[x for exe,x in artifact_by_exe.items() if exe.endswith('/'+Path(suffix).name)]
        need(len(matches)==1,'Running executable does not map to one compiler-artifact: '+suffix)
        artifact=matches[0]
        key=artifact['key'];meta=by_named[key]
        need(str(Path(artifact['src_path']).resolve())==meta['src_path'],'compiler artifact source differs from metadata target')
        shown=header['display']
        shown_src=shown.removeprefix('unittests ').strip()
        need(shown_src==meta['header_path'],'Running source path does not identify compiler-artifact target relative to package manifest: '+shown_src+' != '+meta['header_path'])
        need(artifact['profile_test'] is True,'executed compiler artifact was not built as a test harness')
        observed.append({'key':key,'package_id':artifact['package_id'],'source_path':meta['source_path'],'header_path':meta['header_path'],
                         'counts':block['counts'],'outcomes':block['outcomes'],
                         'running_header':header['display'],'exe_basename':Path(artifact['executable']).name})
    recorded=row.get('executed_targets',[])
    need(len(recorded)==len(set(tuple(x) for x in recorded)),'duplicate executed target in runner row')
    recorded_keys=set()
    for pkg_id_or_name,name,kind in recorded:
        pkg_id=pkg_id_or_name
        if pkg_id not in by_id:
            matches=[pid for pid,p in by_id.items() if p['name']==pkg_id_or_name]
            need(len(matches)==1,'executed target package name is ambiguous')
            pkg_id=matches[0]
        pkg=by_id[pkg_id]; recorded_keys.add((pkg['name'],name,kind))
    observed_keys=[x['key'] for x in observed]
    need(len(observed_keys)==len(set(observed_keys)),'local command executed a target more than once')
    need(set(observed_keys)==recorded_keys,'compiler artifact/Running targets differ from recorded executed_targets')
    counts=[list(x['counts']) for x in observed]
    need(row.get('observed_counts')==counts,'recorded per-target summaries differ from named stdout blocks')
    need(all(c[1]==0 for c in counts),'local full-workspace baseline contains failed test')
    return observed

def check_workspace_argv(argv,observed,by_named):
    packages={x['key'][0] for x in observed}
    need(len(packages)==1,'one local batch executed targets from multiple packages')
    package=next(iter(packages))
    need(argv[:8]==['cargo','test','-p',package,'--all-features','--locked','--message-format','json'],
         'workspace batch package/features/message-format invocation mismatch')
    package_targets={key for key in by_named if key[0]==package}
    selected=set();i=8
    while i<len(argv):
        flag=argv[i]
        if flag in ('--lib','--examples','--tests','--bins','--benches','--all-targets'):
            kinds={'--lib':{'lib'},'--examples':{'example'},'--tests':{'test'},
                   '--bins':{'bin'},'--benches':{'bench'},
                   '--all-targets':{'lib','example','test','bin','bench'}}[flag]
            selected.update(k for k in package_targets if k[2] in kinds);i+=1
        elif flag in ('--test','--example','--bin','--bench'):
            need(i+1<len(argv),'workspace batch target selector lacks a value')
            kind={'--test':'test','--example':'example','--bin':'bin','--bench':'bench'}[flag]
            key=(package,argv[i+1],kind)
            need(key in package_targets,'workspace batch selector names a target outside Cargo metadata: '+repr(key))
            need(key not in selected,'workspace batch selects a target more than once: '+repr(key))
            selected.add(key);i+=2
        else:
            raise ValueError('unreviewed workspace batch argument: '+flag)
    observed_keys={x['key'] for x in observed}
    need(selected==observed_keys,'workspace argv target selectors differ from actual executed target identities')

def verify_workspace_baseline(execution_path,execution_sha,metadata_path,metadata_sha,product):
    ex_raw=read(execution_path);md_raw=read(metadata_path)
    need(sha(ex_raw)==execution_sha and sha(md_raw)==metadata_sha,'explicit workspace execution/metadata SHA mismatch')
    ex=json.loads(ex_raw);md=json.loads(md_raw)
    need(ex.get('schema')=='conditional-full-workspace-batched-root-v1' and ex.get('status')=='passed','workspace baseline is not a completed passing run')
    need(ex.get('pins_unchanged') is True,'workspace baseline pins changed')
    need(ex.get('no_carried_execution') is True,'workspace baseline carried earlier command results')
    need(ex.get('metadata_command',{}).get('exit_code')==0,'workspace metadata command did not succeed')
    md_stdout=verified_stream(ex['metadata_command']['streams']['stdout'])
    need(md_stdout==md_raw and sha(md_stdout)==metadata_sha,'saved Cargo metadata differs from metadata command raw stdout')
    need(ex.get('metadata_command',{}).get('argv')==['cargo','metadata','--no-deps','--format-version','1','--locked'],'Cargo metadata command mismatch')
    need(ex.get('target_size_scan_adapter')=={'path':str(SCAN),'sha256':SCAN_SHA},'workspace execution target-size adapter pin mismatch')
    need(SCAN.is_file() and sha_file(SCAN)==SCAN_SHA and sha(git_blob(product,str(SCAN.relative_to(ROOT))))==SCAN_SHA,'target-size scanner Git/live source pin mismatch')
    need(ex.get('guard_template')=={'path':str(GUARD),'sha256':GUARD_SHA},'workspace runner guard pin mismatch')
    need(GUARD.is_file() and sha_file(GUARD)==GUARD_SHA and sha(git_blob(product,str(GUARD.relative_to(ROOT))))==GUARD_SHA,'guard template Git/live pin mismatch')
    expected_lib=ex.get('source_pins_before_format')
    pins=ex.get('source_pins')
    need(isinstance(expected_lib,dict) and pins==expected_lib,'workspace source pins differ before/after test run')
    need('tests/fixtures/corpus-fingerprint.json' in pins,
         'workspace source pins omit the runtime-read corpus fingerprint fixture')
    git_pins=blob_sha256s(product,pins)
    need(git_pins==pins,'workspace source pins differ from product commit blobs')
    live={p:sha_file(ROOT/p) for p in pins}
    need(live==pins,'workspace source pins differ from live tree')
    by_id,by_named,inventory=metadata_targets(md)
    declared=[tuple(x) for x in ex.get('target_inventory',[])]
    need(len(declared)==len(set(declared)) and set(declared)==set(inventory),'execution target inventory differs from Cargo metadata')
    commands=ex.get('commands',[]);batch_count=ex.get('expected_batch_count')
    need(type(batch_count) is int and batch_count==len(commands),'workspace batch count does not match completed command list')
    need([c.get('index') for c in commands]==list(range(2,2+len(commands))),'workspace batch indexes are incomplete or out of order')
    need(all(c.get('exit_code')==0 and c.get('guard_stop') is None for c in commands),'workspace command failed or guard stopped')
    target_rows=[];seen=[]
    for row in commands:
        obs=local_command_observation(row,by_id,by_named)
        check_workspace_argv(row.get('argv',[]),obs,by_named)
        target_rows.extend(obs);seen.extend(x['key'] for x in obs)
    need(len(seen)==len(set(seen)),'target executed in multiple workspace batches')
    need(set(seen)==set(inventory),'not every Cargo target was executed exactly once')
    need(ex.get('workspace_summary_count')==len(target_rows),'workspace per-target summary count mismatch')
    totals=[sum(row['counts'][i] for row in target_rows) for i in range(3)]
    need(ex.get('workspace_totals')==totals,'workspace totals differ from reconstructed target observations')
    runner=ex.get('runner')
    need(isinstance(runner,dict) and runner.get('path')==str(WORKSPACE_RUNNER) and
         runner.get('sha256')==WORKSPACE_RUNNER_SHA,
         'workspace runner source record absent or malformed')
    runner_path=Path(runner['path'])
    need(runner_path.is_file() and sha_file(runner_path)==runner['sha256'],
         'workspace runner source differs from recorded v4 runner SHA')
    return ex,md,target_rows,{'execution_sha256':execution_sha,'metadata_sha256':metadata_sha,
        'seed':ex.get('seed'),'target_count':len(inventory),'batch_count':len(commands),
        'workspace_totals':totals,'source_pin_count':len(pins),'target_observations':target_rows}

def parse_ci_header(line,targets):
    m=RUNNING_RE.fullmatch(clean(line))
    if not m:return None
    shown=m.group(1).strip().removeprefix('unittests ').strip()
    suffix=Path(m.group(2)).name
    # Cargo executable basename ends in -<16 lowercase hex>; use its prefix plus the
    # exact source path from the header so duplicate test function names stay scoped.
    em=re.fullmatch(r'(.+)-([0-9a-f]{16})',suffix)
    need(em is not None,'CI Running binary name has unexpected Cargo identity: '+suffix)
    cargo_name=em.group(1)
    matches=[t for t in targets if t['header_path']==shown and t['key'][1].replace('-','_')==cargo_name]
    need(len(matches)==1,'CI Running header does not resolve to exactly one baseline target: '+line)
    return matches[0]

def parse_ci_workspace(group,seed,targets):
    need(seed in group,'CI command group missing configured seed')
    headers=[]
    for line in group.splitlines():
        if 'Running ' in clean(line):
            item=parse_ci_header(line,targets)
            if item is not None:headers.append(item)
    blocks=stdout_blocks_cargo_json(group.encode())
    need(len(headers)==len(targets) and len(blocks)==len(targets),'CI target header/result block counts do not match observed full target set')
    keys=[x['key'] for x in headers]
    need(len(keys)==len(set(keys)) and set(keys)=={x['key'] for x in targets},'CI target headers omit/duplicate/introduce target')
    observed=[]
    by_key={x['key']:x for x in targets}
    for target,block in zip(headers,blocks):
        ref=by_key[target['key']]
        need(block['counts']==ref['counts'] and block['outcomes']==ref['outcomes'],
             'CI named outcomes/summary differ from local target '+repr(ref['key']))
        observed.append({'package':ref['key'][0],'name':ref['key'][1],'kind':ref['key'][2],
                         'source_path':ref['source_path'],'counts':list(block['counts']),
                         'test_outcomes':[{'name':n,'status':state} for n,state in block['outcomes']]})
    totals=[sum(x['counts'][i] for x in observed) for i in range(3)]
    return {'seed':seed,'target_count':len(observed),'workspace_totals':totals,'targets':observed}

def strip_log(raw):
    text=raw.decode('utf-8',errors='replace')
    text=re.sub(r'(?m)^[^\t\n]+\t[^\t\n]+\t(?=\d{4}-\d\d-\d\dT)','',text)
    text=re.sub(r'(?m)^\d{4}-\d\d-\d\d[T ][^ ]+Z\s+','',text)
    return clean(text)
def command_groups(lines,command):
    needle='##[group]Run '+command;out=[]
    for i,line in enumerate(lines):
        if needle in line:
            end=next((j for j in range(i+1,len(lines)) if '##[group]Run ' in lines[j]),len(lines))
            out.append('\n'.join(lines[i:end]))
    return out

def verify_capture(capture_dir,product,run_id):
    path=capture_dir/'capture-execution-root-v2.json'; doc=json.loads(read(path))
    need(doc.get('schema')==CAPTURE_SCHEMA and doc.get('status')=='captured' and doc.get('jobs')==4 and doc.get('steps')==52,'CI capture schema/status/count mismatch')
    need(doc.get('product_commit')==product and doc.get('run_id')==int(run_id),'CI capture commit/run mismatch')
    collector=doc.get('collector',{}); cp=Path(collector.get('path',''))
    need(cp.is_file() and sha_file(cp)==collector.get('sha256') and
         cp.resolve()==(Path(__file__).resolve().parent/'capture-conditional-ci-root-v3.py').resolve(),
         'capture collector file/SHA mismatch')
    summary=doc.get('validated_run_summary',{})
    need(summary.get('headSha')==product and summary.get('status')=='completed' and summary.get('conclusion')=='success','CI exact head/status/conclusion mismatch')
    m=re.search(r'/actions/runs/(\d+)(?:$|[/?#])',summary.get('url',''))
    need(m and int(m.group(1))==int(run_id),'CI URL run id mismatch')
    jobs=summary.get('jobs',[])
    need(len(jobs)==4 and {j.get('name') for j in jobs}==EXPECTED_JOBS,'CI exact four-job inventory mismatch')
    need(all(j.get('status')=='completed' and j.get('conclusion')=='success' and all(s.get('status')=='completed' and s.get('conclusion')=='success' for s in j.get('steps',[])) for j in jobs),'CI failed/incomplete job or step')
    steps=sum(len(j.get('steps',[])) for j in jobs);need(steps==52,'CI step count differs from reviewed 52')
    cmds=doc.get('commands',[]);need(len(cmds)==3,'CI capture command count mismatch')
    ids={j['name']:str(j['databaseId']) for j in jobs}
    expected=[['gh','run','view',str(run_id),'--json','headSha,status,conclusion,jobs,url'],
      ['gh','run','view',str(run_id),'--job',ids['stable / test and specification'],'--log'],
      ['gh','run','view',str(run_id),'--job',ids['supply chain'],'--log']]
    for i,row in enumerate(cmds):
        need(row.get('argv')==expected[i] and row.get('cwd')==str(ROOT) and row.get('exit_code')==0 and not row.get('timed_out') and row.get('spawn_error') is None,'CI capture invocation mismatch/failure')
        env=row.get('environment_override',{})
        need(set(env)=={'NO_PROXY','no_proxy'} and all(x.get('appended')==['results-receiver.actions.githubusercontent.com','productionresultssa1.blob.core.windows.net'] and re.fullmatch(r'[0-9a-f]{64}',x.get('final_value_sha256','')) for x in env.values()),'CI capture proxy record mismatch')
        if i==0:
            raw=verified_stream(row['stdout'],capture_dir);verified_stream(row['stderr'],capture_dir);need(json.loads(raw)==summary,'captured raw summary differs from API record')
        else:
            packed=read(capture_dir/('ci-stable-job-v2.log.gz' if i==1 else 'ci-supply-job-v2.log.gz'))
            rec=row['stdout'];need(len(packed)==rec['bytes'] and sha(packed)==rec['sha256'],'captured job gzip pin mismatch')
            raw=gzip.decompress(packed);need(len(raw)==rec['uncompressed_bytes'] and sha(raw)==rec['raw_sha256'] and gzip.compress(raw,mtime=0)==packed,'captured job raw/gzip mismatch')
            raw_rec=row.get('stdout_raw',{})
            expected_raw_path=rec['path'].removesuffix('.gz')+'.stdout.raw'
            need(raw_rec.get('path')==expected_raw_path,'captured raw-log filename mismatch')
            saved=read(capture_dir/expected_raw_path)
            need(saved==raw and sha(saved)==row['stdout_raw']['sha256'] and len(saved)==row['stdout_raw']['bytes'],'captured raw log mismatch')
            verified_stream(row['stderr'],capture_dir)
    return summary,steps

def verify_required_tests(target_rows):
    by_source={}
    for target in target_rows:
        by_source.setdefault(target['source_path'],[]).append(target)
    checked=[]
    for source,required in REQUIRED_TESTS.items():
        target_source='crates/jarde-java/src/lib.rs' if source in ('crates/jarde-java/src/region.rs','crates/jarde-java/src/build.rs') else source
        rows=by_source.get(target_source,[])
        need(len(rows)==1,'required test source does not identify exactly one executed Cargo target: '+source)
        names={name for name,state in rows[0]['outcomes'] if state=='ok'}
        missing=[name for name in required if name not in names]
        need(not missing,'required conditional test missing from actual local output: '+source+': '+', '.join(missing))
        checked.append({'source_path':source,'required_test_count':len(required),'target':rows[0]['key']})
    return checked

def verify_candidate(product,metadata_path,metadata_sha,cli,cli_sha,build_path,build_sha,runner,runner_sha):
    for p,h in ((metadata_path,metadata_sha),(cli,cli_sha),(build_path,build_sha),(runner,runner_sha)):
        need(Path(p).is_file() and sha_file(Path(p))==h,'conditional candidate input SHA mismatch: '+str(p))
    md=json.loads(read(metadata_path));b=json.loads(read(build_path))
    need(md.get('schema')=='recover-proved-conditional-switch-fallthrough-candidate-cli-root-v1','candidate metadata schema mismatch')
    need(md.get('metadata_path')==str(metadata_path.resolve()) and md.get('cli_path')==str(cli.resolve()) and md.get('cli_sha256')==cli_sha and md.get('build_result_sha256')==build_sha and md.get('source_commit_base')==SOURCE_BASE,'candidate metadata CLI/build/source-base binding mismatch')
    need(stat.S_IMODE(Path(cli).stat().st_mode)==0o555,'frozen CLI mode is not 0555')
    need(b.get('schema')=='recover-proved-conditional-switch-fallthrough-validation-build-root-v2' and b.get('status')=='validation-passed-cli-frozen','validation build schema/status mismatch')
    need(b.get('source_commit_base_expected')==SOURCE_BASE and b.get('uncommitted_local_conditional_switch_product') is True,'validation build base/product identity mismatch')
    need(b.get('validation_runner')=={'path':str(runner.resolve()),'sha256':runner_sha} and md.get('validation_runner')==b.get('validation_runner'),'validation runner identity mismatch')
    need(sha(git_blob(product,str(Path(runner).resolve().relative_to(ROOT.resolve()))))==runner_sha,
         'validation runner product Git blob mismatch')
    scanpin={'path':str(SCAN),'sha256':SCAN_SHA}
    need(SCAN.is_file() and sha_file(SCAN)==SCAN_SHA and sha(git_blob(product,str(SCAN.relative_to(ROOT))))==SCAN_SHA,'target-size scan adapter Git/live pin mismatch')
    need(all(x.get('target_size_scan_adapter')==scanpin for x in (b,b.get('freeze',{}),md)),'target-size scan adapter absent/mismatched in build/freeze/metadata')
    need(b.get('freeze',{}).get('cli_path')==str(cli.resolve()) and b['freeze'].get('cli_sha256')==cli_sha and b['freeze'].get('metadata_path')==str(metadata_path.resolve()),'frozen build CLI/metadata binding mismatch')
    groups=('candidate_sources','test_sources','canonical_files'); live={}
    for group in groups:
        pins=md.get(group);need(isinstance(pins,dict) and pins,'candidate pin group absent: '+group)
        for rel,digest in pins.items():
            need(sha_file(ROOT/rel)==digest and sha(git_blob(product,rel))==digest,'conditional source Git/live pin mismatch: '+rel)
        live[group]=pins
    required_source_set=set(REQUIRED_TESTS)
    need(required_source_set <= (set(live['candidate_sources'])|set(live['test_sources'])),
         'candidate metadata omits one or more sources whose test outcomes are required')
    need(set(REQUIRED_TESTS)-{'crates/jarde-java/src/region.rs','crates/jarde-java/src/build.rs'} <= set(live['test_sources']),
         'candidate metadata omits a required integration-test source')
    closure=set()
    for rel in sorted(set(live['candidate_sources'])|set(live['test_sources'])):
        if rel.endswith('.rs'):
            source=git_blob(product,rel).decode('utf-8')
            for include in INCLUDE_RE.findall(source):
                canonical=os.path.normpath((Path(rel).parent/include).as_posix())
                need(canonical!='..' and not canonical.startswith('../'),'literal Rust include escapes repo')
                closure.add(canonical)
    need(closure==set(live['canonical_files']),'conditional literal include closure mismatch')
    pins=b.get('preflight',{}).get('source_pins_before');after=b.get('preflight',{}).get('source_pins_after')
    need(pins==after and pins==live,'validation build before/after pins differ from candidate metadata')
    return {'metadata_sha256':metadata_sha,'cli_sha256':cli_sha,'build_sha256':build_sha,'runner_sha256':runner_sha,
      'candidate_source_count':len(live['candidate_sources']),'test_source_count':len(live['test_sources']),
      'canonical_file_count':len(live['canonical_files']),'target_size_scan_adapter':scanpin}

def verify_replay_invocation(product,inv_path,inv_sha,argv_sha,execution_path,execution_sha,cli,cli_sha,metadata,metadata_sha,build,build_sha,source_base,jdk8,jdk23,stdout_sha):
    need(sha(read(execution_path))==execution_sha,'replay execution SHA mismatch')
    invraw=read(inv_path);need(sha(invraw)==inv_sha,'replay verifier invocation record SHA mismatch')
    inv=json.loads(invraw);argv=inv.get('argv',[])
    need(inv.get('schema')=='conditional-ci-invocation-record-root-v2' and inv.get('exit_code')==0 and
         inv.get('spawn_error') is None and inv.get('cwd')==str(ROOT) and isinstance(argv,list) and argv,
         'replay verifier invocation record schema/cwd/exit mismatch')
    recorder=Path(__file__).resolve().parent/'record-invocation-root-v2.py'
    recorder_row=inv.get('recorder',{})
    need(recorder_row.get('path')==str(recorder) and recorder.is_file() and
         sha_file(recorder)==recorder_row.get('sha256')==RECORDER_SHA,'replay invocation recorder source pin mismatch')
    computed_argv_sha=sha(json.dumps(argv,separators=(',',':')).encode())
    need(computed_argv_sha==argv_sha==inv.get('argv_sha256'),'replay verifier argv SHA mismatch')
    verifier=CHANGE/'results/verify-conditional-replay.py'
    expected_prefix=['uv','run','--offline','--with','blake3','python','-B',str(verifier.resolve())]
    need(argv[:len(expected_prefix)]==expected_prefix,'replay command must use the reviewed offline uv/blake3 invocation')
    verifier_sha=sha_file(verifier)
    need(verifier_sha==sha(git_blob(product,str(verifier.relative_to(ROOT)))),'replay verifier Git/live source mismatch')
    values={}
    arg_vector=argv[len(expected_prefix):]
    for i,item in enumerate(arg_vector[:-1]):
        if item.startswith('--'):
            need(item not in values,'duplicate replay verifier argument: '+item)
            values[item]=arg_vector[i+1]
    expected={'--execution':str(execution_path.resolve()),'--cli':str(cli.resolve()),'--cli-sha256':cli_sha,
      '--metadata':str(metadata.resolve()),'--metadata-sha256':metadata_sha,
      '--validation-execution':str(build.resolve()),'--validation-sha256':build_sha,
      '--source-base':source_base,'--jdk8-home':str(jdk8.resolve()),'--jdk23-home':str(jdk23.resolve())}
    need(all(values.get(k)==v for k,v in expected.items()),'replay verifier argv does not bind exact execution/CLI/metadata/build/JDK inputs')
    typed_expected={'--typed-execution':'/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-local-source-types/results/complete-source-root-v4/execution.json',
      '--typed-execution-sha256':'27afad91dc4e5c426d90f6ff4253b4244541490161752b759d1f5ada19847ed5',
      '--typed-acceptance':'/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-local-source-types/results/complete-source-root-v4/acceptance.json',
      '--typed-acceptance-sha256':'d9e336b8b6eb33b76c0d866eb83040540586eba9df5ba629311f2371f577812f'}
    need(values.get('--typed-execution')==typed_expected['--typed-execution'] and
      values.get('--typed-execution-sha256')=='27afad91dc4e5c426d90f6ff4253b4244541490161752b759d1f5ada19847ed5' and
      values.get('--typed-acceptance')=='/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-local-source-types/results/complete-source-root-v4/acceptance.json' and
      values.get('--typed-acceptance-sha256')==typed_expected['--typed-acceptance-sha256'],'replay verifier typed baseline arguments drifted')
    need(set(values)==set(expected)|set(typed_expected),'replay invocation contains unverified arguments')
    need(jdk8.is_dir() and jdk23.is_dir(),'recorded replay JDK home is missing')
    need(set(inv.get('streams',{}))=={'stdout','stderr'},'replay invocation stream set mismatch')
    output=verified_stream(inv['streams']['stdout'],inv_path.parent)
    need(sha(output)==stdout_sha,'replay verifier supplied stdout SHA differs from invocation raw stdout')
    err=verified_stream(inv['streams']['stderr'],inv_path.parent)
    stderr_sha=sha(err)
    stderr_bytes=len(err)
    doc=json.loads(output)
    need(doc.get('schema')=='recover-proved-conditional-switch-fallthrough-complete-class-replay-root-v1' and doc.get('status')=='verified-conditional-switch-replay-observations' and doc.get('cf12_complete') is False,'independent replay verifier did not accept the replay')
    ex=json.loads(read(execution_path));c=ex.get('candidate',{})
    need(ex.get('status')=='candidate-replay-recorded' and ex.get('schema')==doc.get('schema'),'conditional replay execution identity/status mismatch')
    need(c.get('cli')==str(cli.resolve()) and c.get('cli_sha256')==cli_sha and c.get('metadata')==str(metadata.resolve()) and c.get('validation_execution')==str(build.resolve()) and c.get('validation_sha256')==build_sha and c.get('source_base')==source_base,'replay execution candidate tuple differs from verifier argv')
    return {'execution_path':str(execution_path.resolve()),'execution_sha256':execution_sha,'invocation_path':str(inv_path.resolve()),'invocation_sha256':inv_sha,'argv_sha256':argv_sha,'verifier_path':str(verifier),'verifier_sha256':verifier_sha,'stdout_sha256':stdout_sha,'stderr_sha256':stderr_sha,'stderr_bytes':stderr_bytes,'status':doc['status']}

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--product-commit',required=True);ap.add_argument('--run-id',required=True)
    ap.add_argument('--capture',type=Path,required=True)
    ap.add_argument('--capture-execution-sha256',required=True)
    ap.add_argument('--workspace-execution',type=Path,required=True);ap.add_argument('--workspace-execution-sha256',required=True)
    ap.add_argument('--workspace-metadata',type=Path,required=True);ap.add_argument('--workspace-metadata-sha256',required=True)
    ap.add_argument('--source-base',required=True);ap.add_argument('--metadata',type=Path,required=True);ap.add_argument('--metadata-sha256',required=True)
    ap.add_argument('--cli',type=Path,required=True);ap.add_argument('--cli-sha256',required=True)
    ap.add_argument('--build-execution',type=Path,required=True);ap.add_argument('--build-sha256',required=True)
    ap.add_argument('--validation-runner',type=Path,required=True);ap.add_argument('--runner-sha256',required=True)
    ap.add_argument('--replay-execution',type=Path,required=True);ap.add_argument('--replay-execution-sha256',required=True)
    ap.add_argument('--replay-verifier-invocation',type=Path,required=True);ap.add_argument('--replay-invocation-sha256',required=True)
    ap.add_argument('--replay-argv-sha256',required=True);ap.add_argument('--replay-stdout-sha256',required=True)
    ap.add_argument('--jdk8-home',type=Path,required=True);ap.add_argument('--jdk23-home',type=Path,required=True)
    ap.add_argument('--acceptance',type=Path,required=True)
    a=ap.parse_args()
    need(re.fullmatch(r'[0-9a-f]{40}',a.product_commit) is not None,'product commit must be lowercase 40-hex')
    need(a.run_id.isdecimal(),'run ID must be decimal')
    for v in (a.capture_execution_sha256,a.workspace_execution_sha256,a.workspace_metadata_sha256,a.metadata_sha256,a.cli_sha256,a.build_sha256,a.runner_sha256,a.replay_execution_sha256,a.replay_invocation_sha256,a.replay_argv_sha256,a.replay_stdout_sha256):
        need(re.fullmatch(r'[0-9a-f]{64}',v) is not None,'SHA-256 inputs must be lowercase 64-hex')
    need(a.source_base==SOURCE_BASE,'source-base does not match conditional candidate base')
    need(not a.acceptance.exists(),'refusing to overwrite acceptance output')
    capture_path=a.capture.resolve()/'capture-execution-root-v2.json'
    need(sha_file(capture_path)==a.capture_execution_sha256,'explicit CI capture execution SHA mismatch')
    capture_summary,steps=verify_capture(a.capture.resolve(),a.product_commit,a.run_id)
    workspace_ex,workspace_md,targets,workspace=verify_workspace_baseline(
      a.workspace_execution.resolve(),a.workspace_execution_sha256,a.workspace_metadata.resolve(),a.workspace_metadata_sha256,a.product_commit)
    required_tests=verify_required_tests(targets)
    candidate=verify_candidate(a.product_commit,a.metadata.resolve(),a.metadata_sha256,a.cli.resolve(),a.cli_sha256,
      a.build_execution.resolve(),a.build_sha256,a.validation_runner.resolve(),a.runner_sha256)
    replay=verify_replay_invocation(a.product_commit,a.replay_verifier_invocation.resolve(),a.replay_invocation_sha256,a.replay_argv_sha256,
      a.replay_execution.resolve(),a.replay_execution_sha256,a.cli.resolve(),a.cli_sha256,a.metadata.resolve(),a.metadata_sha256,
      a.build_execution.resolve(),a.build_sha256,a.source_base,a.jdk8_home,a.jdk23_home,a.replay_stdout_sha256)
    workflow=git_blob(a.product_commit,'.github/workflows/ci.yml').decode('utf-8')
    seeds=CI_SEEDS_RE.findall(workflow)
    need(len(seeds)==2 and workflow.count('run: '+WORKSPACE_CMD)==2,'product workflow does not define exactly two workspace seed runs')
    need(workspace['seed']==seeds[0],'fresh workspace baseline seed does not match the first product workflow seed')
    stable=gzip.decompress(read(a.capture.resolve()/'ci-stable-job-v2.log.gz'))
    lines=strip_log(stable).splitlines();groups=command_groups(lines,WORKSPACE_CMD)
    need(len(groups)==2,'stable CI log does not contain exactly two workspace runs')
    seed_runs=[parse_ci_workspace(group,seed,targets) for group,seed in zip(groups,seeds)]
    need(seed_runs[0]['workspace_totals']==seed_runs[1]['workspace_totals'],'two CI seed totals differ')
    need(seed_runs[0]['target_count']==workspace['target_count']==seed_runs[1]['target_count'],'CI/local observed target inventory count differs')
    acceptance={'schema':ACCEPT_SCHEMA,'status':'accepted','product_commit':a.product_commit,'run_id':int(a.run_id),
      'ci':{'jobs':4,'steps':steps,'seed_runs':seed_runs,'aggregate_counts_predeclared':False},
      'local_full_workspace_baseline':{k:v for k,v in workspace.items() if k!='target_observations'},
      'candidate_build':candidate,'conditional_replay':replay,'required_test_observations':required_tests,
      'evidence':{'capture_execution':str((a.capture.resolve()/'capture-execution-root-v2.json')),
        'capture_sha256':sha_file(a.capture.resolve()/'capture-execution-root-v2.json'),
        'workspace_execution':str(a.workspace_execution.resolve()),'workspace_execution_sha256':a.workspace_execution_sha256,
        'workspace_metadata':str(a.workspace_metadata.resolve()),'workspace_metadata_sha256':a.workspace_metadata_sha256},
      'acceptance_scope':'Exact conditional product commit and run; all four CI jobs and 52 steps succeeded; each of the product workflow’s two workspace seeds reproduced every actual target and named outcome observed in the fresh complete local workspace baseline; exact product Git/live pins, included-file closure, scan helper, validation/CLI metadata, and independently invoked CF12 replay verification are bound.'}
    a.acceptance.parent.mkdir(parents=True,exist_ok=True)
    with a.acceptance.open('x',encoding='utf-8') as f:f.write(json.dumps(acceptance,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'status':'accepted','product_commit':a.product_commit,'run_id':int(a.run_id),
      'jobs':4,'steps':steps,'target_count':workspace['target_count'],'seed_totals':[x['workspace_totals'] for x in seed_runs]}))

if __name__=='__main__':
    try:main()
    except Exception as e:
        print('conditional CI verification failed: '+str(e),file=sys.stderr);raise
