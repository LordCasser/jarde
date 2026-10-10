#!/usr/bin/env python3
"""Independent verification of discarded-call product pins and captured CI logs."""
from __future__ import annotations
import gzip, hashlib, importlib.util, json, posixpath, re, stat, subprocess, sys
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
HERE = ROOT / 'openspec/changes/preserve-proved-discarded-call-origins/results'
EVIDENCE = HERE / 'ci-product-v1'
RESULT = EVIDENCE / 'acceptance-proved-discarded-call-ci-root-v1.json'
METADATA = HERE / 'candidate-cli-v1.json'
CLI = Path('/private/tmp/jarde-proved-discarded-call-cli-v1')
BUILD = HERE / 'validation-build-root-v1/execution.json'
BUILD_SCHEMA = 'preserve-proved-discarded-call-origins-validation-build-root-v1'
BUILD_RUNNER = HERE / 'run-validation-build-root-v1.py'
TEMPLATE = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
TEMPLATE_SHA = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
STATIC = ROOT / 'openspec/changes/preserve-nonfinal-static-initializer-phase/results/verify-ci-product-root-v12.py'
STATIC_SHA = '806547956c91e1213b3549ea641497018b97970a5a7796c91c06d345f362d55d'
INTEGER = ROOT / 'openspec/changes/recover-int-array-constant-names/results/verify-int-array-ci-product-luna-v2.py'
INTEGER_SHA = 'e5534e0b2acc6066ba518a1b55f5d0f79b9e5e25a1c4dc5db768040b6d47b425'
PRIOR = ROOT / 'openspec/changes/preserve-proved-return-arm-loop-latch-origins/results/ci-product-v1/acceptance-proved-return-arm-loop-latch-ci-root-v1.json'
PRIOR_SHA = 'd04ac86e2f982783d37382b605a0c34fe1fefcb9a77fc707460e4bb5c3bfea47'
PRODUCT_PATHS = {
 'Cargo.toml','Cargo.lock','crates/jarde-jvm/Cargo.toml','crates/jarde-reader/Cargo.toml',
 'crates/jarde-query/Cargo.toml','crates/jarde-java/Cargo.toml','crates/jarde-cli/Cargo.toml',
 'crates/jarde-java/src/region.rs','crates/jarde-java/src/build.rs','crates/jarde-java/src/emit.rs',
 'crates/jarde-java/src/report.rs','crates/jarde-java/src/lib.rs','src/class_source.rs','src/facade.rs',
 'src/lib.rs','crates/jarde-cli/src/main.rs','crates/jarde-cli/src/task.rs'}
TEST_PATHS = {
 '.github/workflows/ci.yml','crates/jarde-java/tests/proved_discarded_call_origins.rs',
 'crates/jarde-java/tests/p3_loop_exit_gateways.rs','crates/jarde-java/tests/p3_patterns.rs',
 'tests/p3_twr_discarded_call.rs','tests/p3_popped_static_qualifier.rs',
 'tests/p3_loop_arm_join.rs','tests/p3_loop_boolean_exit.rs'}
NEW_TESTS = {
 'frozen_probe_has_exact_identity',
 'proved_static_virtual_and_interface_call_pops_map_to_complete_statements',
 'consumed_and_local_deferred_calls_keep_their_existing_body_and_map',
 'exact_pop_charge_and_cancel_publish_no_partial_statement',
 'wide_pop2_is_not_attached_to_a_call_statement'}
REGION_TESTS = (
 'region::tests::prefixed_loop_chain_certificate_rejects_extra_entries_exits_and_owners',
 'region::tests::prefixed_loop_header_cannot_reopen_claimed_outer_target_or_parent_scope')
REQUIRED_LIB = 'build::tests::exception_if_join_transfer_rejects_a_canonical_exception_edge'
ENV = {'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0',
       'CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always'}
INCLUDE_RE = re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')
SUMMARY_RE = re.compile(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')

sha = lambda b: hashlib.sha256(b).hexdigest()
def read(p: Path) -> bytes: return p.read_bytes()
def git_blob(commit: str, path: str) -> bytes:
    return subprocess.check_output(['git','show',f'{commit}:{path}'],cwd=ROOT)
def load(name: str,path: Path):
    spec=importlib.util.spec_from_file_location(name,path); assert spec and spec.loader
    mod=importlib.util.module_from_spec(spec);sys.modules[name]=mod;spec.loader.exec_module(mod);return mod
def canonical_paths(product: str) -> set[str]:
    result=set()
    for rel in TEST_PATHS:
        for included in INCLUDE_RE.findall(git_blob(product,rel).decode()):
            p=posixpath.normpath(posixpath.join(posixpath.dirname(rel),included))
            assert p!='..' and not p.startswith('../');result.add(p)
    base='openspec/changes/preserve-proved-discarded-call-origins/results/'
    result.update(base+x for x in (
      'original-inputs-root-v1/DiscardedCallSourceProbe.java',
      'original-inputs-root-v1/DiscardedCallSourceProbeRunner.java',
      'original-inputs-root-v1/ProbePop2.java','baseline-root-v1/acceptance-root-v1.json',
      'baseline-root-v1/execution.json','run-baseline-root-v1.py'))
    result.add('openspec/changes/preserve-proved-return-arm-loop-latch-origins/results/ci-product-v1/acceptance-proved-return-arm-loop-latch-ci-root-v1.json')
    for jdk in ('javac8','javac23'):
        for binary in ('DiscardedCallSourceProbe.class','DiscardedCallSourceProbeRunner.class','ProbePop2.class'):
            result.add(base+f'baseline-root-v1/{jdk}/original/discardprobe/{binary}')
    assert len(result)==47, f'expected exact 47 canonical inputs, got {len(result)}'
    return result

def expected_clippy(workflow: str) -> list[str]:
    start=workflow.index('      - name: Run Clippy');end=workflow.index('      - name: Run workspace tests',start)
    names=re.findall(r'(?m)^\s+-A clippy::([a-z0-9_]+)\s*$',workflow[start:end])
    assert len(names)==29 and len(set(names))==29
    return ['cargo','clippy','--workspace','--all-targets','--all-features','--locked','--',
      *sum((['-A',f'clippy::{n}'] for n in names),[]),'-D','warnings']

def load_helpers(evidence: Path):
    assert sha(read(STATIC))==STATIC_SHA and sha(read(INTEGER))==INTEGER_SHA
    h=load('discarded_static_ci_v12',STATIC);h.CI_EVIDENCE=evidence;h.METADATA_PATH=METADATA
    orig_strip=h.strip_terminal
    def strip(raw):
        text=raw.decode('utf-8',errors='strict')
        text=re.sub(r'(?m)^[^\t\n]+\t[^\t\n]+\t(?=\d{4}-\d\dT)','',text)
        text=re.sub(r'(?:\x1b\[[0-9;]*m|\^\[\[[0-9;]*m)','',text)
        return re.sub(r'(?m)^\d{4}-\d\d-\d\d[T ][^ ]+Z[ \t]','',text)
    h.strip_terminal=strip
    original=h.binary_group
    def group(lines,binary,count,ignored=0):
        if binary=='tests/p5_corpus_fingerprint.rs':
            starts=[i for i,line in enumerate(lines) if (f'Running {binary}' in line if binary != 'tests/proved_discarded_call_origins.rs' else re.search(r'Running .*proved_discarded_call_origins\.rs',line))];assert len(starts)==1
            marks=[(i,re.search(r'\brunning (\d+) tests?\b',line)) for i,line in enumerate(lines[starts[0]+1:],starts[0]+1) if re.search(r'\brunning (\d+) tests?\b',line)]
            assert marks and int(marks[0][1].group(1))==count+ignored
            begin=marks[0][0];summaries=[(i,h.test_counts(line)) for i,line in enumerate(lines[begin+1:],begin+1) if 'test result:' in line]
            assert summaries;end,actual=summaries[0];block='\n'.join(lines[begin:end+1])
            expected={'corpus_files_match_the_recorded_fingerprint':'ok','every_acceptance_row_is_indexed_against_existing_corpus':'ok','every_dimension_is_carried_by_existing_corpus':'ok','regenerate_corpus_fingerprint':'ignored','the_indexed_rows_are_the_rows_the_acceptance_table_names':'ok','the_manifest_matches_its_rendered_classification':'ok'}
            outcomes=re.findall(r'\btest ([A-Za-z0-9_:]+) \.\.\. (ok|ignored|FAILED)',block)
            assert count==5 and ignored==1 and actual==[(5,0,1)] and dict(outcomes)==expected and len(outcomes)==6
            return block,actual
        if binary=='tests/interface_initializer_proof.rs':
            starts=[i for i,line in enumerate(lines) if (f'Running {binary}' in line if binary != 'tests/proved_discarded_call_origins.rs' else re.search(r'Running .*proved_discarded_call_origins\.rs',line))];assert len(starts)==1
            marks=[(i,re.search(r'\brunning (\d+) tests?\b',line)) for i,line in enumerate(lines[starts[0]+1:],starts[0]+1) if re.search(r'\brunning (\d+) tests?\b',line)]
            assert marks and int(marks[0][1].group(1))==count+ignored
            begin=marks[0][0]; summaries=[(i,h.test_counts(line)) for i,line in enumerate(lines[begin+1:],begin+1) if 'test result:' in line]
            assert summaries;end,actual=summaries[0];block='\n'.join(lines[begin:end+1])
            outcomes=re.findall(r'\btest ([A-Za-z0-9_:]+) \.\.\. (ok|ignored|FAILED)',block)
            assert actual==[(count,0,ignored)] and len(outcomes)==count+ignored
            assert {n for n,_ in outcomes}==set(h.INTERFACE_TESTS) and all(v=='ok' for _,v in outcomes)
            return block,actual
        if binary not in ('tests/proved_discarded_call_origins.rs','tests/p3_loop_exit_gateways.rs'):
            return original(lines,binary,count,ignored)
        starts=[i for i,line in enumerate(lines) if (f'Running {binary}' in line if binary != 'tests/proved_discarded_call_origins.rs' else re.search(r'Running .*proved_discarded_call_origins\.rs',line))]
        assert len(starts)==1,(binary,starts)
        markers=[(i,re.search(r'\brunning (\d+) tests?\b',line)) for i,line in enumerate(lines[starts[0]+1:],starts[0]+1)
                 if re.search(r'\brunning (\d+) tests?\b',line)]
        assert markers and int(markers[0][1].group(1))==count+ignored,(binary,markers[:1])
        begin=markers[0][0]
        summaries=[(i,h.test_counts(line)) for i,line in enumerate(lines[begin+1:],begin+1) if 'test result:' in line]
        assert summaries,(binary,'missing summary')
        end,actual=summaries[0]
        block='\n'.join(lines[begin:end+1])
        outcomes=re.findall(r'\btest ([A-Za-z0-9_:]+) \.\.\. (ok|ignored|FAILED)',block)
        if binary.endswith('proved_discarded_call_origins.rs'):
            assert count==5 and ignored==0 and actual==[(5,0,0)]
            assert len(outcomes)==5 and {n for n,_ in outcomes}==NEW_TESTS and all(v=='ok' for _,v in outcomes),outcomes
        else:
            assert actual==[(count,0,ignored)] and len(outcomes)==count+ignored
            assert {n for n,_ in outcomes}==GATEWAY_NAMES and all(v=='ok' for _,v in outcomes),outcomes
        return block,actual
    h.binary_group=group
    i=load('discarded_accepted_integer_verifier_v2',INTEGER);i.CI_EVIDENCE=evidence
    return h,i

GATEWAY_NAMES=set()
def verify_build(product: str, source_base: str, meta_sha: str, cli_sha: str, build_sha: str, runner_sha: str) -> dict:
    meta_raw=read(METADATA);assert sha(meta_raw)==meta_sha
    meta=json.loads(meta_raw)
    assert meta['schema']=='preserve-proved-discarded-call-origins-candidate-cli-v1'
    assert meta['cli_path']==str(CLI) and meta['cli_sha256']==cli_sha and sha(read(CLI))==cli_sha
    assert stat.S_IMODE(CLI.stat().st_mode)==0o555
    assert meta['source_commit_base']==source_base and meta['uncommitted_discarded_call_product'] is True
    assert meta['required_origin_tests']==sorted(NEW_TESTS) and meta['expected_origin_test_count']==5
    assert set(meta['candidate_sources'])==PRODUCT_PATHS and set(meta['test_sources'])==TEST_PATHS
    gateway_source=git_blob(product,'crates/jarde-java/tests/p3_loop_exit_gateways.rs').decode()
    gateway_names=set(re.findall(r'(?ms)^\s*#\[test\]\s*(?:#\[[^\]]+\]\s*)*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(',gateway_source))
    assert gateway_names==GATEWAY_NAMES,(gateway_names,GATEWAY_NAMES)
    test_source=git_blob(product,'crates/jarde-java/tests/proved_discarded_call_origins.rs').decode()
    actual_names=set(re.findall(r'(?ms)^\s*#\[test\]\s*(?:#\[[^\]]+\]\s*)*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(',test_source))
    assert actual_names==NEW_TESTS,(actual_names,NEW_TESTS)
    can=canonical_paths(product);assert set(meta['canonical_files'])==can
    for cat,expected in (('candidate_sources',PRODUCT_PATHS),('test_sources',TEST_PATHS),('canonical_files',can)):
        assert set(meta[cat])==expected
        for path,digest in meta[cat].items():
            assert sha(git_blob(product,path))==digest and sha(read(ROOT/path))==digest,(cat,path)
    assert len(PRODUCT_PATHS|TEST_PATHS|can)==72
    braw=read(BUILD);assert sha(braw)==build_sha==meta['build_result_sha256'];b=json.loads(braw)
    assert b['schema']==BUILD_SCHEMA and b['status']=='validation-passed-cli-frozen'
    assert b['source_commit_base_expected']==source_base and b['uncommitted_discarded_call_product'] is True
    assert b['required_origin_tests']==sorted(NEW_TESTS) and b['expected_origin_test_count']==5
    assert b['expected_library_test_count']==337 and b['environment_overrides']==ENV
    assert b['guards']=={'minimum_free_bytes':5*1024**3,'maximum_target_bytes':1024**3}
    assert b['validation_runner']==meta['validation_runner']=={'path':str(BUILD_RUNNER),'sha256':runner_sha}
    assert sha(read(BUILD_RUNNER))==runner_sha
    assert b['guarded_runner_template']==meta['guarded_runner_template']=={'path':str(TEMPLATE),'sha256':TEMPLATE_SHA}
    assert sha(read(TEMPLATE))==TEMPLATE_SHA
    before=b['preflight']['source_pins_before'];after=b['preflight']['source_pins_after']
    mp={k:meta[k] for k in ('candidate_sources','test_sources','canonical_files')};assert before==after==mp
    head=b['preflight']['git_head'];assert head['matches_expected'] and head['value']==source_base
    assert head['argv']==['git','rev-parse','HEAD'] and head['exit_code']==0
    for rec in (head['stdout'],head['stderr']):
        p=Path(rec['path']);p=p if p.is_absolute() else ROOT/p
        p.resolve().relative_to(ROOT.resolve());raw=read(p)
        assert len(raw)==rec['bytes'] and sha(raw)==rec['sha256']
    for rec in b['preflight']['source_pins_before'].values():
        assert isinstance(rec,dict)
    expected=[['cargo','fmt','--all','--','--check'], expected_clippy(git_blob(product,'.github/workflows/ci.yml').decode()),
      ['cargo','test','-p','jarde-java','--lib','--locked'],
      ['cargo','test','-p','jarde-java','--test','proved_discarded_call_origins','--locked','--','--nocapture'],
      ['cargo','test','-p','jarde','--test','p3_twr_discarded_call','--locked'],
      ['cargo','test','-p','jarde','--test','p3_popped_static_qualifier','--locked'],
      ['cargo','test','-p','jarde-java','--test','p3_loop_exit_gateways','--locked','--','--nocapture'],
      ['cargo','test','-p','jarde','--test','p3_loop_arm_join','--locked'],
      ['cargo','test','-p','jarde','--test','p3_loop_boolean_exit','--locked'],
      ['cargo','test','-p','jarde-java','--test','p3_patterns','--locked'],
      ['cargo','test','-p','jarde-reader','--lib','--locked'],['cargo','build','-p','jarde-cli','--locked']]
    assert len(b['commands'])==12
    summaries={2:[(337,0,0)],3:[(5,0,0)],4:[(8,0,0)],5:[(4,0,0)],6:[(15,0,0)],7:[(4,0,0)],8:[(3,0,0)],9:[(85,0,0)],10:[(178,0,0)]}
    raw_count=0; recomputed={}
    for idx,(row,argv) in enumerate(zip(b['commands'],expected)):
        assert row['index']==idx and row['argv']==argv and row['cwd']==str(ROOT)
        assert row['exit_code']==0 and row['guard_stop'] is None and row['env_overrides']==ENV
        assert row['peak_target_bytes']<=1024**3 and row['free_bytes_after']>=5*1024**3
        streams={}
        for name,rec in row['streams'].items():
            p=Path(rec['path']);p=p if p.is_absolute() else ROOT/p
            p.resolve().relative_to(ROOT.resolve())
            data=read(p);assert len(data)==rec['bytes'] and sha(data)==rec['sha256'];streams[name]=data;raw_count+=1
        if idx in summaries:
            check=row['test_summary_check'];assert check and check['ok'] and check['required']
            text=streams['stdout'].decode('utf-8',errors='strict')
            actual=[tuple(map(int,x)) for x in SUMMARY_RE.findall(text)]
            assert actual==summaries[idx] and [tuple(x) for x in check['actual']]==actual
            assert check['required_test_names_present'] is True
            if idx==3:
                assert set(check['required_test_names'])==NEW_TESTS and len(check['required_test_names'])==5
                for n in NEW_TESTS: assert re.search(r'^test '+re.escape(n)+r' \.\.\. ok$',text,re.M),n
            if idx==2: assert check['required_test_names']==[]
            recomputed[idx]=[list(x) for x in actual]
        else:
            check=row['test_summary_check'];assert check['expected'] is None and check['actual']==[]
            assert check['required'] is False and check['ok'] is True and check['failure_reason'] is None
    assert raw_count==24
    freeze=b['freeze'];assert freeze['cli_path']==str(CLI) and freeze['cli_sha256']==cli_sha
    assert freeze['metadata_path']==str(METADATA) and freeze['cli_mode']=='0o555'
    assert freeze['source_commit_base']==source_base and freeze['uncommitted_discarded_call_product'] is True
    assert freeze['required_origin_tests']==sorted(NEW_TESTS) and freeze['expected_library_test_count']==337
    return {'metadata_sha256':meta_sha,'cli':{'path':str(CLI),'sha256':cli_sha,'mode':'0o555'},
      'build_sha256':build_sha,'runner_sha256':runner_sha,'source_base':source_base,'pins_verified':72,
      'build_commands':12,'build_streams_verified':raw_count,'recomputed_test_summaries':recomputed}

def verify_capture_streams(capture: dict, run_id: str, evidence: Path, ci_document: dict) -> dict:
    rows=capture['commands']; assert len(rows)==3
    assert rows[0]['argv']==['gh','run','view',run_id,'--json','headSha,status,conclusion,jobs,url']
    for row in rows:
        assert row['cwd']==str(ROOT) and row['timeout_seconds']==35 and row['timed_out'] is False
        assert row['exit_code']==0 and row['spawn_error'] is None and row['duration_seconds']>=0
        assert re.fullmatch(r'\d{4}-\d\d-\d\dT.*\+00:00',row['started_at'])
        env=row['environment_override'];assert set(env)=={'NO_PROXY','no_proxy'}
        for val in env.values():
            assert val['appended']==['results-receiver.actions.githubusercontent.com','productionresultssa1.blob.core.windows.net']
            assert re.fullmatch(r'[0-9a-f]{64}',val['final_value_sha256'])
    jobs={j['name']:str(j['databaseId']) for j in ci_document['jobs']}
    expected=(('stable / test and specification','ci-stable-job-v1.log.gz'),('supply chain','ci-supply-job-v1.log.gz'))
    assert [(row['argv'][5],row['stdout']['path']) for row in rows[1:]]==[(jobs[name],filename) for name,filename in expected]
    records=[]
    def check(rec, rel=None):
        path=evidence/(rel or rec['path']); raw=read(path)
        assert len(raw)==rec['bytes'] and sha(raw)==rec['sha256'],str(path)
        return raw
    api_raw=check(rows[0]['stdout'],'ci-run-v1.json')
    check(rows[0]['stderr'],'ci-run-v1.stderr.raw')
    assert json.loads(api_raw)==capture['validated_run_summary']
    for row,(_,filename) in zip(rows[1:],expected):
        compressed=check(row['stdout'],filename)
        plain=gzip.decompress(compressed)
        assert len(plain)==row['stdout']['uncompressed_bytes'] and sha(plain)==row['stdout']['raw_sha256']
        assert gzip.compress(plain,mtime=0)==compressed
        check(row['stdout_raw'],filename.removesuffix('.gz')+'.stdout.raw')
        assert read(evidence/(filename.removesuffix('.gz')+'.stdout.raw'))==plain
        check(row['stderr'],filename+'.stderr.raw')
        records.append({'argv':row['argv'],'stdout_gzip_sha256':sha(compressed),'stdout_raw_sha256':sha(plain),
                        'stderr_sha256':row['stderr']['sha256'],'exit_code':row['exit_code']})
        assert row['exit_code']==0 and not row['timed_out']
    return {'api_sha256':sha(api_raw),'job_logs':records}

def main():
    import argparse
    ap=argparse.ArgumentParser(description=__doc__)
    for arg in ('product-commit','run-id','source-base','metadata-sha256','cli-sha256','build-sha256','runner-sha256'):
        ap.add_argument('--'+arg,required=True)
    args=ap.parse_args()
    assert re.fullmatch('[0-9a-f]{40}',args.product_commit) and re.fullmatch('[0-9a-f]{40}',args.source_base)
    assert args.run_id.isdecimal()
    for name in ('metadata_sha256','cli_sha256','build_sha256','runner_sha256'): assert re.fullmatch('[0-9a-f]{64}',getattr(args,name))
    assert not RESULT.exists() and EVIDENCE.is_dir()
    prior_raw=read(PRIOR);assert sha(prior_raw)==PRIOR_SHA;prior=json.loads(prior_raw)
    assert prior['schema']=='preserve-proved-return-arm-loop-latch-origins-ci-product-root-acceptance-v1'
    assert prior['status']=='accepted' and prior['product_commit']=='0ae30a7c8d217522f2e5f5b954aa86c9b59356dd' and prior['run_id']==38073837512
    seed_rows=prior['workspace_regressions']['workspace_seed_runs']
    assert len(seed_rows)==2 and all(r['library_test_count']==337 and r['workspace_expected_totals']==[3393,0,97] and r['workspace_result_records']==354 for r in seed_rows)
    prior_gateways=set(seed_rows[0]['gateway_test_names']);assert len(prior_gateways)==15 and all(set(r['gateway_test_names'])==prior_gateways for r in seed_rows)
    cap=json.loads(read(EVIDENCE/'capture-execution-root-v1.json'))
    assert cap['schema']=='preserve-proved-discarded-call-origins-ci-capture-root-v1' and cap['status']=='captured'
    assert cap['run_id']==args.run_id and cap['expected_head_sha']==args.product_commit
    assert cap['expected_test_counts']=={'lib':337,'gateway':15,'workspace_passed':3398}
    assert set(cap['new_test_names'])==NEW_TESTS and len(cap['new_test_names'])==5
    assert cap['prior_return_acceptance']=={'path':str(PRIOR),'sha256':PRIOR_SHA}
    helper,integer=load_helpers(EVIDENCE)
    global GATEWAY_NAMES
    GATEWAY_NAMES=prior_gateways
    ci=helper.verify_ci(args.product_commit,int(args.run_id));assert ci['jobs']==4 and ci['step_count']==52 and ci['all_jobs_and_steps_success'] is True
    capture_streams=verify_capture_streams(cap,args.run_id,EVIDENCE,cap['validated_run_summary'])
    stable=helper.strip_terminal(gzip.decompress(read(EVIDENCE/'ci-stable-job-v1.log.gz')))
    groups=helper.command_groups(stable.splitlines(),'cargo test --workspace --all-targets --all-features --locked')
    seeds=re.findall(r'PROPTEST_RNG_SEED:\s*"(\d+)"',git_blob(args.product_commit,'.github/workflows/ci.yml').decode())
    assert seeds==['5350648285461741569','5350648285461741570'] and len(groups)==2
    ws=[]
    for (_,text),seed in zip(groups,seeds):
        assert seed in text and all(other not in text for other in seeds if other!=seed)
        lines=text.splitlines()
        # Require the exact lib target marker and following running count/summary in each seed.
        libmarkers=[(i,re.search(r'\brunning (\d+) tests?\b',l)) for i,l in enumerate(lines) if re.search(r'\brunning (\d+) tests?\b',l) and int(re.search(r'\brunning (\d+) tests?\b',l).group(1))==337]
        assert len(libmarkers)==1,(seed,libmarkers)
        lib_header=next((lines[j] for j in range(libmarkers[0][0]-1,-1,-1) if 'Running' in lines[j]),'')
        assert 'src/lib.rs' in lib_header and 'target/debug/deps/jarde_java-' in lib_header,(seed,lib_header)
        libsummary=[helper.test_counts(l) for l in lines[libmarkers[0][0]+1:] if 'test result:' in l]
        assert libsummary and libsummary[0]==[(337,0,0)]
        new_headers=[l for l in lines if re.search(r'Running .*proved_discarded_call_origins\.rs',l)]
        assert len(new_headers)==1,(seed,new_headers)
        iblock,icount=helper.binary_group(lines,'tests/proved_discarded_call_origins.rs',5)
        for name in NEW_TESTS: assert re.search(r'(?m)^test '+re.escape(name)+r' \.\.\. ok$',iblock), (seed,name)
        gwblock,gwcount=helper.binary_group(lines,'tests/p3_loop_exit_gateways.rs',15)
        for name in prior_gateways: assert re.search(r'(?m)^test '+re.escape(name)+r' \.\.\. ok$',gwblock),(seed,name)
        totals=helper.test_counts(text);assert len(totals)==355
        aggregate=(sum(x[0] for x in totals),sum(x[1] for x in totals),sum(x[2] for x in totals))
        assert aggregate==(3398,0,97) and all(f==0 for _,f,_ in totals)
        integer_runs=integer.verify_integer_tests_in_both_seeds if False else None
        ws.append({'seed':seed,'library_binary_header':lib_header,'lib_counts':[337,0,0],
          'new_binary_header':new_headers[0],'new_binary_counts':[5,0,0],
          'new_test_names':sorted(NEW_TESTS),'gateway_counts':[15,0,0],
          'gateway_test_names':sorted(prior_gateways),'workspace_records':355,
          'workspace_totals':[3398,0,97]})
    integer.CI_EVIDENCE=EVIDENCE
    integer_runs=integer.verify_integer_tests_in_both_seeds(helper,args.product_commit)
    assert len(integer_runs)==2 and len(integer.WORKSPACE_TESTS)==11
    build=verify_build(args.product_commit,args.source_base,args.metadata_sha256,args.cli_sha256,args.build_sha256,args.runner_sha256)
    result={'schema':'preserve-proved-discarded-call-origins-ci-product-root-acceptance-v1','status':'accepted',
      'product_commit':args.product_commit,'run_id':int(args.run_id),'source_commit_base':args.source_base,
      'prior_return_acceptance':{'path':str(PRIOR),'sha256':PRIOR_SHA,'baseline_workspace':[3393,0,97]},
      'shared_ci_verification':{'static_helper_sha256':STATIC_SHA,'integer_verifier_sha256':INTEGER_SHA,
                                'jobs':ci['jobs'],'steps':ci['step_count'],'capture_streams':capture_streams},
      'ci_workspace':{'seeds':ws,'prior_integer_tests':integer_runs,'workspace_records_per_seed':355,
                      'expected_totals_per_seed':[3398,0,97]},'frozen_cli_and_build':build,
      'acceptance_scope':'Exact product Git blobs and live 17 product, 8 test, and 47 canonical pins; frozen CLI/build/runner; CI head and complete four-job/52-step success; both fixed seeds with direct logs proving lib337, gateway15, all five new integration test names, 355 workspace records, 3398 passed/0 failed/97 ignored.'}
    with RESULT.open('x',encoding='utf-8') as f:f.write(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'status':'accepted','product_commit':args.product_commit,'run_id':int(args.run_id),'jobs':4,'steps':52,'workspace_passed_per_seed':3398,'new_tests':5}))
if __name__=='__main__': main()
