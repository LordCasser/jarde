#!/usr/bin/env python3
"""Prepare deterministic dual-JDK whole-class candidate observations; root runs after review."""
from __future__ import annotations
import argparse, ast, datetime, hashlib, json, os, re, shutil, stat, subprocess, sys, zipfile
from pathlib import Path
from blake3 import blake3

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
HERE=Path('/private/tmp/jarde-return-arm-latch-controls-v1')
RESULTS=ROOT/'openspec/changes/preserve-proved-return-arm-loop-latch-origins/results'
OUT=RESULTS/'old-controls-replay-root-v1'
ACCEPTANCE=RESULTS/'old-controls-exact-acceptance-root-v2.json'
FOR_RESULTS=ROOT/'openspec/changes/preserve-proved-for-latch-origins/results'
FOR_BUNDLE=FOR_RESULTS/'candidate-whole-classes-root-v1'
FOR_ACCEPTANCE=FOR_RESULTS/'candidate-whole-classes-acceptance-root-v1.json'
FOR_ACCEPTANCE_SHA='1f7a318a0c7da89744814414ec4a8cbdc22efbe6920ff601ede98f7b6c0b0866'
FOR_MANIFEST_SHA='38e80cb85429e0236f91b9828fafc145eebf23e990e79b4ef02e9609dc7c5525'
FOR_INVENTORY_SHA='4324037c2fdfa492c3a228c50bd08d77a03b78741c3e008eadb1d6e14453f5fd'
PREVIOUS_RESULTS=ROOT/'openspec/changes/preserve-proved-if-arm-join-origins/results'
PREVIOUS_BUNDLE=PREVIOUS_RESULTS/'candidate-whole-classes-root-v1'
PREVIOUS_ACCEPTANCE=PREVIOUS_RESULTS/'old-controls-exact-acceptance-root-v1.json'
PREVIOUS_ACCEPTANCE_SHA='fc277edd77738d6ec5d3dfcce5f9254c2e68f491f74895dae989432dc8309082'
PREVIOUS_MANIFEST_SHA='89953659d9e59eddf5ff72f5e1b7cae6fc7d9ad80603ed415a9e9a38047c053a'
PREVIOUS_INVENTORY_SHA='804843867b9b311bd2a5d20e24e6e69314e493b956a71df0fc8e34223fab83f9'
PREVIOUS_WHOLE_ACCEPTANCE=PREVIOUS_RESULTS/'candidate-whole-classes-acceptance-root-v1.json'
PREVIOUS_WHOLE_ACCEPTANCE_SHA='187db36694949caf956c8b2c78dd6806b50e2540ed57e00fa378f90e9116c1ec'
COLLECTOR_PATH=HERE/'prepare-controls-root-v1.py'
CLI_PATH=Path('/private/tmp/jarde-return-arm-latch-cli-v1')
METADATA_PATH=RESULTS/'candidate-cli-v1.json'
METADATA_SCHEMA='preserve-proved-return-arm-loop-latch-origins-candidate-cli-v1'
SOURCE_BASE_EXPECTED='41fe336462448eae3547cafa2a141d52e85a08e0'
CLI_SHA_EXPECTED='9ae5817d25749add0709b4f156c71ce23ac0e5ad41d66ee0136742024461a28f'
METADATA_SHA_EXPECTED='9913bb51cfa4acc91ed10708f94acdcedb6b4eb85dd4ea31d827febbd05ad6ef'
BUILD_SHA_EXPECTED='d81a2918b1a55e7d910ebd59d82956c14155b6968f4bfb602e85c99d88ab70de'

SOURCE_BASE=None
EVID=ROOT/'openspec/evidence/java-syntax-2026-10-10'
JDK_MANIFEST=ROOT/'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json'
JDK_MANIFEST_SHA256='ec27b5a0e8a9d2345d289b4626bac516e055e0f86d70ab3b2f744c336f961aec'
JADX=Path('/opt/homebrew/bin/jadx'); JADX_SHA256='64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7'; JADX_VERSION='1.5.6'
GUARD_SOURCE=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA256='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
GUARD_FREE=5*1024**3; GUARD_TARGET=1024**3
PRODUCT_PATHS={"Cargo.toml","Cargo.lock","crates/jarde-jvm/Cargo.toml","crates/jarde-reader/Cargo.toml","crates/jarde-query/Cargo.toml","crates/jarde-java/Cargo.toml","crates/jarde-cli/Cargo.toml","crates/jarde-java/src/region.rs","crates/jarde-java/src/build.rs","crates/jarde-java/src/emit.rs","crates/jarde-java/src/report.rs","crates/jarde-java/src/lib.rs","src/class_source.rs","src/facade.rs","src/lib.rs","crates/jarde-cli/src/main.rs","crates/jarde-cli/src/task.rs"}
TEST_PATHS={".github/workflows/ci.yml","crates/jarde-java/tests/p3_loop_exit_gateways.rs","crates/jarde-java/tests/p3_loop_body_double_jumps.rs","crates/jarde-java/tests/p3_loop_terminal_return.rs","crates/jarde-java/tests/p3_effectful_exits.rs","tests/p3_loop_arm_join.rs","tests/p3_loop_boolean_exit.rs","tests/p5_corpus_fingerprint.rs"}
INCLUDE_RE=re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"\s*,?\s*\)')
SUITES={
 'postfix':{'base':EVID/'em23-variable-postfix-loop/baseline-root-v1','collector':EVID/'em23-variable-postfix-loop/results/prepare-baseline-luna-v1.py','collector_sha':'2e35326ebe7b2b60bd47b60419375088bc36142e053ea41f086f3cf534743089','manifest_sha':'e8aec7964455ab84ffb12d984140e18bffe94e5cc0cf7f1eb5de06c21cc23a2d','inventory_sha':'89af5a986896541c5adb0dd8bed30c89d61038017f05d57634142e8416cce783','source':'VariablePostfixLoop.java','source_sha':'17147219c9e524d18d62063f932b1ebb3ae976cb6f3bd3fddd2bfd8a58889199','runner_sha':'4408a0d8dea6b761b35546e314dc9961cf515ddeb31920eb5665524598f03102','class':'VariablePostfixLoop','old_accept_sha':'946409416021061c868d671b929279937997efcf479da7c1bef7ab7ccabe49eb','old_accept':EVID/'em23-variable-postfix-loop/results/independent-acceptance-luna-v3.json'},
 'plain':{'base':EVID/'one-arm-loop-controls/baseline-root-v1','collector':EVID/'one-arm-loop-controls/prepare-baseline-luna-v1.py','collector_sha':'a1ed035789f96f2a945bc9e8499bb4d1d089639082ccb59ce61933c97660879c','manifest_sha':'6dfccb5ceeb854edf41b90e4136c23ce503699bb83bb17767444cd6fd6cfc18d','inventory_sha':'96d487211c0bcbb71b6fb06a19894fb3a8d0b59a44b5376f8a5a3a48aa7c1b96','source':'PlainOneArmLoops.java','source_sha':'8f5ccc668017113caf93002d3e07853206412ca581a23dfb98950ad7393805b4','runner_sha':'240b28968c2a0f466660e2b191b6c08cc801e7024362c0dccbe2f47427bc2f9b','class':'PlainOneArmLoops','old_accept_sha':'f95666277091007f178cd4a74170f22eddd7699a49b2e5db02c08fa27bbad450','old_accept':EVID/'one-arm-loop-controls/results/independent-acceptance-luna-v5.json'}}
LEGS=('javac8','javac23'); STRIPPED=('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH')
def sha(b): return hashlib.sha256(b).hexdigest()
def writej(p,v): p.parent.mkdir(parents=True,exist_ok=True); p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def sha_file(p):
 h=hashlib.sha256()
 with p.open('rb') as f:
  for block in iter(lambda:f.read(1<<20),b''): h.update(block)
 return h.hexdigest()
def target_bytes():
 p=ROOT/'target'; return sum(x.stat().st_size for x in p.rglob('*') if x.is_file()) if p.exists() else 0
def load_guard():
 raw=GUARD_SOURCE.read_bytes()
 if sha(raw)!=GUARD_SHA256: raise RuntimeError('pinned v9 guarded runner changed')
 tree=ast.parse(raw); names={'sha_file','target_bytes','command_stream','expected_test_summaries','run_command'}
 nodes=[n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name in names]
 ns={'__name__':'_pinned_guard_helpers','__file__':str(GUARD_SOURCE),'ROOT':ROOT,'OUT':OUT/'streams','FREE_LIMIT':GUARD_FREE,'TARGET_LIMIT':GUARD_TARGET,'shutil':shutil,'datetime':datetime,'time':__import__('time'),'subprocess':subprocess,'os':os,'signal':__import__('signal'),'re':re,'hashlib':hashlib,'SUMMARY_RE':re.compile(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored'),'ENV_VALUES':{'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always'},'EXPECTED_SUMMARIES':{},'TEST_COMMANDS':set(),'REQUIRED_GATEWAY_TESTS':(),'REQUIRED_BOOLEAN_LOOP_TESTS':()}
 exec(compile(ast.fix_missing_locations(ast.Module(body=nodes,type_ignores=[])),str(GUARD_SOURCE),'exec'),ns)
 return ns['run_command']
def load_helpers(conf):
 raw=conf['collector'].read_bytes()
 if sha(raw)!=conf['collector_sha']: raise RuntimeError('pinned baseline collector changed')
 tree=ast.parse(raw); wanted={'file_record','inventory_rows','package_of','package_of_text','copy_runner','adapt_runner','class_files','compile_run','parse_javap','parse_javap_methods','method_key','verify_report_map','b3','sha'}
 nodes=[n for n in tree.body if isinstance(n,(ast.FunctionDef,ast.ClassDef)) and n.name in wanted]
 imports=[n for n in tree.body if isinstance(n,ast.ImportFrom) and n.module=='blake3']
 ns={'__name__':'_pinned_baseline_helpers','__file__':str(conf['collector']),'Path':Path,'re':re,'hashlib':hashlib,'blake3':blake3,'OUT':OUT,'ROOT':ROOT,'RUNNER':conf['base']/'original-sources/Runner.java','CLASS_NAME':conf['class']}
 exec(compile(ast.fix_missing_locations(ast.Module(body=imports+nodes,type_ignores=[])),str(conf['collector']),'exec'),ns)
 ns['METHOD_KEYS']=[('<init>','()V'),('countEmpty','(Ljava/util/List;)I')] if conf['class']=='VariablePostfixLoop' else [('<init>','()V'),('prefixWhile','(ZI)I'),('noPrefix','(ZI)I'),('loopAndTail','(ZI)I'),('takenArm','(ZI)I')]
 if conf['class']=='VariablePostfixLoop': ns['METHODS']=set(ns['METHOD_KEYS'])
 else: ns['METHOD_FLAGS']={'<init>()V':1,'prefixWhile(ZI)I':9,'noPrefix(ZI)I':9,'loopAndTail(ZI)I':9,'takenArm(ZI)I':9}
 return ns
def closed_baseline(c):
 base=c['base']; mb=(base/'manifest.json').read_bytes(); ib=(base/'file-inventory.json').read_bytes()
 if sha(mb)!=c['manifest_sha'] or sha(ib)!=c['inventory_sha']: raise RuntimeError('baseline manifest/inventory pin mismatch')
 rows=json.loads(ib); actual={x.relative_to(base).as_posix():x for x in base.rglob('*') if x.is_file() and x!=base/'file-inventory.json'}
 if set(actual)!={r['path'] for r in rows}: raise RuntimeError('baseline inventory is not closed')
 for r in rows:
  b=actual[r['path']].read_bytes()
  if len(b)!=r['bytes'] or sha(b)!=r['sha256']: raise RuntimeError('baseline file pin mismatch '+r['path'])
 if sha(c['old_accept'].read_bytes())!=c['old_accept_sha'] or json.loads(c['old_accept'].read_bytes()).get('verified') is not True: raise RuntimeError('independent baseline acceptance mismatch')
 return {'manifest_sha256':sha(mb),'inventory_sha256':sha(ib),'file_count':len(actual),'closed':True}
def accepted_for_bundle_baseline():
 acceptance_raw=FOR_ACCEPTANCE.read_bytes()
 if sha(acceptance_raw)!=FOR_ACCEPTANCE_SHA: raise RuntimeError('accepted For bundle acceptance SHA mismatch')
 accepted=json.loads(acceptance_raw)
 if accepted.get('schema')!='preserve-proved-for-latch-origins-candidate-acceptance-root-v1' or accepted.get('verified') is not True or accepted.get('full_bci_acceptance') is not True or accepted.get('status')!='accepted': raise RuntimeError('previous For bundle is not fully accepted')
 manifest_path=FOR_BUNDLE/'manifest.json'; inventory_path=FOR_BUNDLE/'file-inventory.json'
 manifest_raw=manifest_path.read_bytes(); inventory_raw=inventory_path.read_bytes()
 if sha(manifest_raw)!=FOR_MANIFEST_SHA or accepted.get('manifest_sha256')!=FOR_MANIFEST_SHA: raise RuntimeError('accepted For manifest SHA mismatch')
 if sha(inventory_raw)!=FOR_INVENTORY_SHA: raise RuntimeError('accepted For inventory SHA mismatch')
 inventory=json.loads(inventory_raw); actual={p.relative_to(FOR_BUNDLE).as_posix():p for p in FOR_BUNDLE.rglob('*') if p.is_file() and p!=inventory_path}
 if set(actual)!={row['path'] for row in inventory}: raise RuntimeError('accepted For bundle inventory is not closed')
 for row in inventory:
  raw=actual[row['path']].read_bytes()
  if len(raw)!=row['bytes'] or sha(raw)!=row['sha256']: raise RuntimeError('accepted For bundle member mismatch '+row['path'])
 manifest=json.loads(manifest_raw)
 if manifest.get('schema')!='preserve-proved-for-latch-origins-candidate-observation-v1' or manifest.get('status')!='observations-recorded' or manifest.get('failures'): raise RuntimeError('accepted For observation manifest is not clean')
 return {'acceptance_path':str(FOR_ACCEPTANCE),'acceptance_sha256':FOR_ACCEPTANCE_SHA,'manifest_path':str(manifest_path),'manifest_sha256':FOR_MANIFEST_SHA,'inventory_path':str(inventory_path),'inventory_sha256':FOR_INVENTORY_SHA,'files':len(inventory),'full_bci_acceptance':True}

def accepted_previous_controls_baseline():
 acceptance_raw=PREVIOUS_ACCEPTANCE.read_bytes()
 if sha(acceptance_raw)!=PREVIOUS_ACCEPTANCE_SHA: raise RuntimeError('previous old-controls acceptance SHA mismatch')
 accepted=json.loads(acceptance_raw)
 if (accepted.get('schema')!='preserve-proved-if-arm-old-controls-exact-acceptance-root-v1'
     or accepted.get('status')!='accepted' or accepted.get('generated_class_profiles')!=8
     or accepted.get('method_profiles')!=28 or accepted.get('body_and_all_maps_byte_facts_equal') is not True
     or accepted.get('previous_manifest_sha256')!=FOR_MANIFEST_SHA
     or accepted.get('candidate_manifest_sha256')!=PREVIOUS_MANIFEST_SHA
     or accepted.get('independent_whole_class_acceptance_sha256')!=PREVIOUS_WHOLE_ACCEPTANCE_SHA):
  raise RuntimeError('previous accepted old-controls record does not certify the full matrix')
 previous_whole_raw=PREVIOUS_WHOLE_ACCEPTANCE.read_bytes()
 if sha(previous_whole_raw)!=PREVIOUS_WHOLE_ACCEPTANCE_SHA: raise RuntimeError('previous full-class acceptance SHA mismatch')
 whole=json.loads(previous_whole_raw)
 if (whole.get('schema')!='preserve-proved-if-arm-join-origins-candidate-acceptance-root-v1'
     or whole.get('verified') is not True or whole.get('full_bci_acceptance') is not True
     or whole.get('status')!='accepted' or whole.get('commands')!=57
     or whole.get('generated_whole_classes')!=8 or whole.get('original_whole_classes')!=4
     or whole.get('jadx_whole_classes')!=8): raise RuntimeError('previous whole-class acceptance is incomplete')
 manifest_path=PREVIOUS_BUNDLE/'manifest.json'; inventory_path=PREVIOUS_BUNDLE/'file-inventory.json'
 manifest_raw=manifest_path.read_bytes(); inventory_raw=inventory_path.read_bytes()
 if sha(manifest_raw)!=PREVIOUS_MANIFEST_SHA or whole.get('manifest_sha256')!=PREVIOUS_MANIFEST_SHA or whole.get('manifest_path')!=str(manifest_path): raise RuntimeError('previous manifest SHA mismatch')
 if sha(inventory_raw)!=PREVIOUS_INVENTORY_SHA: raise RuntimeError('previous output inventory SHA mismatch')
 inventory=json.loads(inventory_raw); actual={p.relative_to(PREVIOUS_BUNDLE).as_posix():p for p in PREVIOUS_BUNDLE.rglob('*') if p.is_file() and p!=inventory_path}
 if set(actual)!={row['path'] for row in inventory}: raise RuntimeError('previous controls bundle inventory is not closed')
 for row in inventory:
  raw=actual[row['path']].read_bytes()
  if len(raw)!=row['bytes'] or sha(raw)!=row['sha256']: raise RuntimeError('previous controls bundle member mismatch '+row['path'])
 manifest=json.loads(manifest_raw)
 if manifest.get('schema')!='preserve-proved-if-arm-join-origins-candidate-observation-v1' or manifest.get('status')!='observations-recorded' or manifest.get('failures'):
  raise RuntimeError('previous accepted controls manifest is not clean')
 return {'acceptance_path':str(PREVIOUS_ACCEPTANCE),'acceptance_sha256':PREVIOUS_ACCEPTANCE_SHA,
         'whole_class_acceptance_path':str(PREVIOUS_WHOLE_ACCEPTANCE),'whole_class_acceptance_sha256':PREVIOUS_WHOLE_ACCEPTANCE_SHA,
         'manifest_path':str(manifest_path),'manifest_sha256':PREVIOUS_MANIFEST_SHA,
         'inventory_path':str(inventory_path),'inventory_sha256':PREVIOUS_INVENTORY_SHA,
         'generated_class_profiles':8,'method_profiles':28,'closed':True}

def actual_source_pins():
 canonical=set()
 for rel in sorted(TEST_PATHS):
  if rel.endswith('.rs'):
   for inc in INCLUDE_RE.findall((ROOT/rel).read_text(encoding='utf-8')):
    canonical.add((ROOT/rel).parent.joinpath(inc).resolve().relative_to(ROOT).as_posix())
 canonical.add('openspec/changes/preserve-proved-if-arm-join-origins/results/exception-join-case-root-v1/classes/ifjoin/ExceptionIfJoin.class')
 groups={'candidate_sources':PRODUCT_PATHS,'test_sources':TEST_PATHS,'canonical_files':canonical}
 return {g:{x:sha_file(ROOT/x) for x in sorted(paths)} for g,paths in groups.items()}, {g:set(paths) for g,paths in groups.items()}
def main():
 global SOURCE_BASE
 ap=argparse.ArgumentParser(); ap.add_argument('--source-base',required=True); ap.add_argument('--cli',required=True); ap.add_argument('--cli-sha256',required=True); ap.add_argument('--metadata',required=True); ap.add_argument('--metadata-sha256',required=True); ap.add_argument('--build',required=True); a=ap.parse_args()
 SOURCE_BASE=a.source_base
 if SOURCE_BASE!=SOURCE_BASE_EXPECTED: raise SystemExit('--source-base does not match the frozen return-arm candidate base')
 cli=Path(a.cli).resolve(); meta=Path(a.metadata).resolve(); build=Path(a.build).resolve()
 if cli.resolve()!=CLI_PATH or meta.resolve()!=METADATA_PATH.resolve(): raise SystemExit('candidate CLI/metadata paths do not identify this change; no fallback')
 results_root=RESULTS.resolve(); build_resolved=build.resolve()
 if build_resolved!= (RESULTS/'validation-build-root-v1/execution.json').resolve() or not build_resolved.is_relative_to(results_root): raise SystemExit('--build must name this slice validation-build-root-v1/execution.json')
 if sha_file(build_resolved)!=BUILD_SHA_EXPECTED: raise SystemExit('validation build execution SHA mismatch')
 if OUT.exists() or ACCEPTANCE.exists(): raise SystemExit('refusing to overwrite candidate result/acceptance')
 if not cli.is_file() or not meta.is_file() or not build.is_file(): raise SystemExit('frozen CLI, metadata, or validation-build execution is missing; no fallback')
 if a.cli_sha256!=CLI_SHA_EXPECTED or a.metadata_sha256!=METADATA_SHA_EXPECTED or sha_file(cli)!=CLI_SHA_EXPECTED or sha_file(meta)!=METADATA_SHA_EXPECTED: raise SystemExit('candidate CLI or metadata hash mismatch')
 if stat.S_IMODE(cli.stat().st_mode)!=0o555: raise SystemExit('candidate CLI mode must be 0555')
 md=json.loads(meta.read_bytes()); exec_bytes=build.read_bytes(); execution=json.loads(exec_bytes)
 if md.get('cli_path')!=str(cli) or md.get('cli_sha256')!=a.cli_sha256 or md.get('build_result_sha256')!=sha(exec_bytes): raise SystemExit('metadata CLI/build-result binding mismatch')
 schema=execution.get('schema',''); match=re.fullmatch(r'preserve-proved-return-arm-loop-latch-origins-validation-build-root-v([1-9][0-9]*)',schema)
 if not match: raise SystemExit('build execution schema is outside this change validation-build family')
 version=match.group(1)
 if build_resolved.parent.name!=f'validation-build-root-v{version}' or build_resolved.parent.parent!=results_root: raise SystemExit('--build directory does not match its validation schema version')
 runner=(build_resolved.parent.parent/f"run-validation-build-root-v{version}.py").resolve()
 runner_row=execution.get('validation_runner',{})
 if runner_row.get('path')!=str(runner) or not runner.is_file() or runner_row.get('sha256')!=sha_file(runner): raise SystemExit('validation runner path/SHA does not match build schema and live file')
 template=execution.get('guarded_runner_template',{})
 if template.get('path')!=str(GUARD_SOURCE) or template.get('sha256')!=GUARD_SHA256 or sha_file(GUARD_SOURCE)!=GUARD_SHA256: raise SystemExit('validation guarded-runner template pin mismatch')
 freeze=execution.get('freeze',{})
 if (md.get('schema')!=METADATA_SCHEMA or md.get('metadata_path')!=str(meta)
     or md.get('uncommitted_return_arm_loop_latch_product') is not True
     or md.get('source_commit_base')!=SOURCE_BASE
     or execution.get('status')!='validation-passed-cli-frozen'
     or freeze.get('cli_path')!=str(cli) or freeze.get('cli_sha256')!=a.cli_sha256
     or freeze.get('metadata_path')!=str(meta) or freeze.get('cli_mode')!='0o555'
     or freeze.get('source_commit_base')!=SOURCE_BASE
     or freeze.get('source_commit_base')!=md.get('source_commit_base')
     or freeze.get('uncommitted_return_arm_loop_latch_product') is not True
     or execution.get('uncommitted_return_arm_loop_latch_product') is not True): raise SystemExit('build execution/metadata did not freeze this candidate and source base')
 pins,sets=actual_source_pins()
 for group,paths in sets.items():
  if set(md.get(group,{}))!=paths or md[group]!=pins[group]: raise SystemExit(f'{group} path set/content pins differ from validation root')
 before=execution.get('preflight',{}).get('source_pins_before'); after=execution.get('preflight',{}).get('source_pins_after')
 if before!=after or before!=pins: raise SystemExit('validation source pins before/after do not match metadata')
 if len(sets['candidate_sources'])!=17 or len(sets['test_sources'])!=8: raise SystemExit('frozen product/test path count mismatch')
 if ACCEPTANCE.exists(): raise SystemExit('acceptance already exists')
 if not OUT.parent.is_dir(): raise SystemExit('change results directory missing')
 if shutil.disk_usage(ROOT).free<GUARD_FREE or target_bytes()>GUARD_TARGET: raise SystemExit('5GiB/1GiB resource guard failed before replay')
 guard_run=load_guard(); OUT.mkdir(); (OUT/'streams').mkdir(); (OUT/'cases').mkdir()
 configs={k:{**c,'helpers':load_helpers(c),'source_path':c['base']/'original-sources'/c['source'],'runner_path':c['base']/'original-sources/Runner.java'} for k,c in SUITES.items()}
 baselines={k:closed_baseline(c) for k,c in configs.items()}
 for_bundle_baseline=accepted_for_bundle_baseline()
 previous_controls_baseline=accepted_previous_controls_baseline()
 jraw=JDK_MANIFEST.read_bytes()
 if sha(jraw)!=JDK_MANIFEST_SHA256: raise RuntimeError('JDK manifest pin mismatch')
 jm=json.loads(jraw); legs={}; env_base=os.environ.copy(); stripped=[k for k in STRIPPED if k in env_base]
 for k in STRIPPED: env_base.pop(k,None)
 env_base.update({'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always'})
 for row in jm['legs']:
  tools={}; checks={}
  for n,f in row['jdk_tools'].items():
   p=Path(f['path']); actual=sha_file(p) if p.is_file() else None
   tools[n]=p; checks[n]={'path':str(p),'expected_sha256':f['sha256'],'actual_sha256':actual,'ok':actual==f['sha256']}
  legs[row['leg']]={'tools':tools,'home':tools['java'].parent.parent,'checks':checks}
 if set(legs)!=set(LEGS): raise RuntimeError('frozen JDK leg set mismatch')
 if any(not all(v['ok'] for v in leg['checks'].values()) for leg in legs.values()): raise RuntimeError('actual JDK tool hash mismatch')
 if not JADX.is_file() or sha_file(JADX)!=JADX_SHA256: raise RuntimeError('fixed JADX launcher hash mismatch')
 manifest={'schema':'preserve-proved-return-arm-loop-latch-origins-old-controls-observation-v1','status':'running','collector':{'path':str(COLLECTOR_PATH),'sha256':sha_file(Path(__file__).resolve())},'cli':{'path':str(cli),'sha256':a.cli_sha256,'mode':oct(stat.S_IMODE(cli.stat().st_mode))},'metadata':{'path':str(meta),'sha256':a.metadata_sha256,'schema':md['schema'],'source_commit_base':md['source_commit_base'],'uncommitted_return_arm_loop_latch_product':md['uncommitted_return_arm_loop_latch_product'],'build_result_sha256':sha(exec_bytes),'build_result_path':str(build_resolved),'source_pins':pins,'source_pins_before':before,'source_pins_after':after},'baselines':baselines,'accepted_for_bundle_baseline':for_bundle_baseline,'accepted_previous_controls_baseline':previous_controls_baseline,'fixtures':{k:{'source_path':str(v['source_path']),'source_sha256':sha_file(v['source_path']),'runner_path':str(v['runner_path']),'runner_sha256':sha_file(v['runner_path'])} for k,v in configs.items()},'jdk':{'manifest_sha256':JDK_MANIFEST_SHA256,'legs':{k:v['checks'] for k,v in legs.items()}},'jadx':{'path':str(JADX),'sha256':JADX_SHA256,'version':JADX_VERSION},'commands':[],'suites':{},'failures':[],'preflight':{'stripped_environment_keys':stripped,'target_bytes':target_bytes(),'free_bytes':shutil.disk_usage(ROOT).free}}
 counter=0
 class Recorder:
  def run(self,label,argv,home=None):
   nonlocal counter
   counter+=1; env=env_base.copy()
   if home: env['JAVA_HOME']=str(home); env['PATH']=str(home/'bin')+os.pathsep+env.get('PATH','')
   row=guard_run(counter,[str(v) for v in argv],env)
   row['label']=label; row['java_home']=str(home) if home else None; row['stripped_environment_keys']=stripped
   # Keep raw stream references relative to this exclusive candidate bundle.
   for nm in ('stdout','stderr'):
    q=ROOT/row['streams'][nm]['path']; row['streams'][nm]['path']=q.relative_to(OUT).as_posix()
   manifest['commands'].append(row)
   writej(OUT/'manifest.json',manifest)
   return row['exit_code'],(OUT/row['streams']['stdout']['path']).read_bytes(),(OUT/row['streams']['stderr']['path']).read_bytes(),row
 recorder=Recorder()
 # Fixed version and tool identity are measured through guarded raw commands.
 jc,jo,je,jrow=recorder.run('jadx-version',[JADX,'--version'],legs['javac23']['home']); manifest['jadx']['version_observed']=jo.decode(errors='replace').strip(); manifest['jadx']['version_command']=jrow
 if jc!=0 or jo.strip()!=JADX_VERSION.encode(): manifest['failures'].append('JADX version mismatch')
 for suite,c in configs.items():
  h=c['helpers']; cls=c['class']; source=c['source_path']; runner=c['runner_path']; srow={'cases':[],'renders':[]}; manifest['suites'][suite]=srow
  if sha_file(source)!=c['source_sha'] or sha_file(runner)!=c['runner_sha']: raise RuntimeError('frozen input SHA mismatch '+suite)
  originals={}; oracle={}; javaps={}; docs={}
  for legname in LEGS:
   leg=legs[legname]; case=OUT/'cases'/suite/legname/'original'; case.mkdir(parents=True)
   sc=case/source.name; rc=case/'Runner.java'; shutil.copyfile(source,sc); shutil.copyfile(runner,rc)
   row,rt,outs=h['compile_run'](recorder,f'{suite}-{legname}-original',[sc,rc],leg,'Runner')
   clsfile=next((p for p in outs if p.name==cls+'.class'),None); originals[legname]=clsfile; oracle[legname]=rt
   row.update({'kind':'original','jdk_leg':legname,'complete_class_set':row['class_set_exact']})
   if clsfile:
    code,jout,jerr,jcmd=recorder.run(f'{suite}-{legname}-javap',[leg['tools']['javap'],'-p','-c','-s','-v',clsfile],leg['home'])
    jp=case/'javap.txt'; jp.write_bytes(jout); parser=h.get('parse_javap',h.get('parse_javap_methods')); javaps[legname]=parser(jout.decode('utf-8',errors='replace'))
    row['javap']={'command':jcmd,'path':jp.relative_to(OUT).as_posix(),'sha256':sha(jout),'exit':code}
   srow['cases'].append({'kind':'original','leg':legname,'row':row})
   if not row['compile_success'] or not row['runtime_success'] or not row['class_set_exact']: manifest['failures'].append(f'{suite}/{legname}: original oracle compile/runtime failure')
  jar=OUT/'cases'/suite/'input.jar'
  with zipfile.ZipFile(jar,'w',compression=zipfile.ZIP_STORED) as z:
   info=zipfile.ZipInfo(cls+'.class',date_time=(1980,1,1,0,0,0));info.compress_type=zipfile.ZIP_STORED;z.writestr(info,originals['javac23'].read_bytes())
  for profile in ('default','none'):
   dest=OUT/'cases'/suite/'jadx'/profile
   argv=[JADX,'--no-res','--config','none','--threads-count','1']+(['--rename-flags','none'] if profile=='none' else [])+['-d',dest,jar]
   code,_,_,cmd=recorder.run(f'{suite}-jadx-{profile}',argv,legs['javac23']['home']); srcs=sorted(dest.rglob('*.java')) if dest.exists() else []
   srow['renders'].append({'kind':'jadx','profile':profile,'command':cmd,'sources':[p.relative_to(OUT).as_posix() for p in srcs]})
   for legname in LEGS:
    if len(srcs)!=1:
     manifest['failures'].append(f'{suite}/jadx/{profile}/{legname}: source inventory failure'); continue
    case=OUT/'cases'/suite/legname/f'jadx-{profile}'; case.mkdir(parents=True); sc=case/(cls+'.java'); shutil.copyfile(srcs[0],sc)
    pkg=(h['package_of_text'](sc.read_text(encoding='utf-8')) if 'package_of_text' in h else (re.search(r'(?m)^\s*package\s+([\w.]+)\s*;',sc.read_text(encoding='utf-8')).group(1) if re.search(r'(?m)^\s*package\s+([\w.]+)\s*;',sc.read_text(encoding='utf-8')) else None)); adapt=h.get('adapt_runner',h.get('copy_runner')); rc=adapt(pkg,case/'Runner.java')
    runner_class=(pkg+'.' if pkg else '')+'Runner'; row,rt,outs=h['compile_run'](recorder,f'{suite}-{legname}-jadx-{profile}',[sc,rc],legs[legname],runner_class)
    row.update({'kind':'jadx','jdk_leg':legname,'profile':profile,'generated_source':sc.relative_to(OUT).as_posix(),'runner_adaptation':rc.relative_to(OUT).as_posix(),'runtime_raw_match':bool(rt and oracle.get(legname) and (rt['exit'],rt['stdout'],rt['stderr'])==(oracle[legname]['exit'],oracle[legname]['stdout'],oracle[legname]['stderr']))})
    srow['cases'].append({'kind':'jadx','leg':legname,'profile':profile,'row':row})
    if not row['compile_success'] or not row['runtime_success'] or not row['class_set_exact'] or not row['runtime_raw_match']: manifest['failures'].append(f'{suite}/{legname}/{profile}: JADX complete class failure')
  for legname in LEGS:
   for mode in ('default','all'):
    dest=OUT/'cases'/suite/legname/f'jarde-{mode}'; dest.mkdir(parents=True)
    argv=[cli,'class-source','--input',originals[legname],'--class',cls,'--policy','single-class','--release','8','--format','json']
    if mode=='all':argv.extend(['--evidence','all'])
    code,out,err,cmd=recorder.run(f'{suite}-{legname}-jarde-{mode}',argv,legs[legname]['home']); p=dest/'report.json'; p.write_bytes(out)
    try:doc=json.loads(out)
    except Exception:doc=None; manifest['failures'].append(f'{suite}/{legname}/{mode}: invalid candidate JSON')
    render={'kind':'jarde','leg':legname,'mode':mode,'command':cmd,'document':p.relative_to(OUT).as_posix(),'document_sha256':sha(out),'exit':code,'stderr_sha256':sha(err)}; srow['renders'].append(render)
    if doc is None: continue
    docs[(legname,mode)]=doc
    generated=dest/(cls+'.java'); generated.write_text(doc.get('text',''),encoding='utf-8')
    pkg=(h['package_of_text'](doc.get('text','')) if 'package_of_text' in h else (re.search(r'(?m)^\s*package\s+([\w.]+)\s*;',doc.get('text','')).group(1) if re.search(r'(?m)^\s*package\s+([\w.]+)\s*;',doc.get('text','')) else None)); adapt=h.get('adapt_runner',h.get('copy_runner')); rc=adapt(pkg,dest/'Runner.java')
    runner_class=(pkg+'.' if pkg else '')+'Runner'
    # Every returned class-source is compiled/run before source-map policy can reject it.
    cr,rt,outs=h['compile_run'](recorder,f'{suite}-{legname}-jarde-{mode}',[generated,rc],legs[legname],runner_class)
    same=bool(rt and oracle.get(legname) and (rt['exit'],rt['stdout'],rt['stderr'])==(oracle[legname]['exit'],oracle[legname]['stdout'],oracle[legname]['stderr']))
    render.update({'generated_source':generated.relative_to(OUT).as_posix(),'generated_source_sha256':sha(generated.read_bytes()),'runner_adaptation':rc.relative_to(OUT).as_posix(),'complete_class_compile_run':cr,'runtime_raw_match':same})
    try:
     facts=h['verify_report_map'](doc,h['b3'](originals[legname].read_bytes()),originals[legname].read_bytes(),javaps[legname],True) if suite=='postfix' else h['verify_report_map'](doc,originals[legname].read_bytes(),javaps[legname],mode)
     render['map_facts']=facts
    except Exception as e:
     render['map_error']=f'{type(e).__name__}: {e}'; manifest['failures'].append(f'{suite}/{legname}/{mode}: source map helper rejected: {type(e).__name__}')
    if not cr['compile_success'] or not cr['runtime_success'] or not same: manifest['failures'].append(f'{suite}/{legname}/{mode}: complete class compile/runtime mismatch')
  for legname in LEGS:
   pair=(docs.get((legname,'default')),docs.get((legname,'all')))
   if all(pair):
    ddoc,adoc=pair; text_equal=ddoc.get('text')==adoc.get('text')
    def maps(d):return {h['method_key'](x['item']):x.get('outcome',{}).get('report',{}).get('source_map') for x in d.get('methods',[])}
    map_equal=maps(ddoc)==maps(adoc)
    for mode in ('default','all'):
     row=next(x for x in srow['renders'] if x.get('kind')=='jarde' and x.get('leg')==legname and x.get('mode')==mode)
     row.update({'default_all_text_equal':text_equal,'default_all_source_maps_equal':map_equal})
    if not text_equal or not map_equal:manifest['failures'].append(f'{suite}/{legname}: default/all source or map differs')
 pins_after_replay,_=actual_source_pins()
 manifest['metadata']['source_pins_after_replay']=pins_after_replay
 if pins_after_replay!=pins:manifest['failures'].append('product/test/canonical pins changed during replay')
 manifest['failures'].extend(f'JDK tool check failed: {leg}/{tool}' for leg,l in legs.items() for tool,v in l['checks'].items() if not v['ok'])
 manifest['status']='observations-recorded'
 writej(OUT/'manifest.json',manifest)
 inventory=[]
 for fp in sorted(OUT.rglob('*')):
  if fp.is_file() and fp.name!='file-inventory.json':
   b=fp.read_bytes(); inventory.append({'path':fp.relative_to(OUT).as_posix(),'bytes':len(b),'sha256':sha(b)})
 writej(OUT/'file-inventory.json',inventory)
 print(f'observations recorded at {OUT}; verifier decides acceptance')
if __name__=='__main__':main()
