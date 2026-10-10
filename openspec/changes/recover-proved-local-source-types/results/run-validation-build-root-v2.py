#!/usr/bin/env python3
"""Guarded local-source-types validation and private CLI freeze draft.

Root-reviewed adapter. Execution evidence is written separately.
"""
from __future__ import annotations
import argparse, datetime, hashlib, importlib.util, json, os, re, shutil, stat, subprocess, sys
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
TASK=Path(__file__).resolve().parent
RESULTS=ROOT/'openspec/changes/recover-proved-local-source-types/results'
OUT=RESULTS/'validation-build-root-v2'
CLI_PATH=Path('/private/tmp/jarde-proved-local-source-types-cli-v1')
METADATA_PATH=RESULTS/'candidate-cli-typed-root-v1.json'
TEMPLATE=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
TEMPLATE_SHA256='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SOURCE_COMMIT_BASE: str|None=None
EXPECTED_LIB_COUNT=337
EXPECTED_ORIGIN_COUNT=0
REQUIRED_ORIGIN_TESTS: tuple[str,...]=()

PRODUCT_PATHS={
 'Cargo.toml','Cargo.lock','crates/jarde-jvm/Cargo.toml','crates/jarde-reader/Cargo.toml',
 'crates/jarde-query/Cargo.toml','crates/jarde-java/Cargo.toml','crates/jarde-cli/Cargo.toml',
 'crates/jarde-java/src/region.rs','crates/jarde-java/src/build.rs','crates/jarde-java/src/emit.rs',
 'crates/jarde-java/src/report.rs','crates/jarde-java/src/lib.rs','src/class_source.rs','src/facade.rs',
 'src/lib.rs','crates/jarde-cli/src/main.rs','crates/jarde-cli/src/task.rs',
}
# These targets cover the changed declaration proof and its nearest consumers: descriptor-based
# parameters, boolean/guard precedence, call overload selection, qualifier semantics, and slot reuse.
TEST_PATHS={
 '.github/workflows/ci.yml',
 'crates/jarde-java/tests/cf12_proved_local_source_types.rs',
 'crates/jarde-java/tests/p3_proved_boolean_conditional_returns.rs',
 'crates/jarde-java/tests/p3_patterns.rs',
 'tests/p3_declarations.rs','tests/p3_parameter_slots.rs','tests/p3_guard.rs',
 'tests/p3_popped_static_qualifier.rs','tests/p3_invocation_arguments.rs',
 'tests/p3_array_slot_retype_locals.rs',
}
INCLUDE_RE=re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')
TEST_RE=re.compile(r'(?ms)^\s*#\[test\]\s*((?:#\[[^\]]+\]\s*)*)fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(')
SUMMARY_RE=re.compile(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored')
CI_CLIPPY_ALLOW_LINTS=(
 'too_many_arguments','cloned_ref_to_slice_refs','collapsible_if','type_complexity','len_zero',
 'needless_option_as_deref','needless_borrow','useless_conversion','large_enum_variant','question_mark',
 'comparison_to_empty','op_ref','manual_range_patterns','if_same_then_else','filter_map_bool_then',
 'filter_next','unneeded_struct_pattern','redundant_guards','map_identity','redundant_slicing',
 'unnecessary_get_then_check','unnecessary_unwrap','redundant_locals','replace_box','map_clone',
 'unnecessary_mut_passed','single_element_loop','unnecessary_to_owned','needless_lifetimes')

# Source inclusion closure is recomputed after the test list is checked. These are the staged targets.
COMMANDS=[]
TEST_COMMANDS={2,3,4,5,6,7,8,9,10,11,12}
EXPECTED_SUMMARIES={}


def sha_file(p:Path)->str:
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
 return h.hexdigest()

def load_guard():
 if sha_file(TEMPLATE)!=TEMPLATE_SHA256:raise RuntimeError('pinned v9 command guard SHA mismatch')
 spec=importlib.util.spec_from_file_location('typed_validation_guard_v9',TEMPLATE)
 if spec is None or spec.loader is None:raise RuntimeError('cannot load v9 command guard')
 module=importlib.util.module_from_spec(spec);sys.modules[spec.name]=module;spec.loader.exec_module(module)
 return module

def test_functions(path:Path)->list[tuple[str,bool]]:
 return [(name,'ignore' in attrs) for attrs,name in TEST_RE.findall(path.read_text(encoding='utf-8'))]

def discovered_literal_paths()->set[str]:
 found=set()
 for rel in sorted(TEST_PATHS):
  if rel.endswith('.rs'):
   src=(ROOT/rel).read_text(encoding='utf-8')
   for value in INCLUDE_RE.findall(src):
    p=(ROOT/rel).parent.joinpath(value).resolve()
    found.add(p.relative_to(ROOT).as_posix())
 return found

def summary_check(index:int,stdout:bytes)->dict:
 text=stdout.decode('utf-8',errors='replace')
 actual=[tuple(map(int,m)) for m in SUMMARY_RE.findall(text)]
 expected=EXPECTED_SUMMARIES.get(index)
 nonempty=bool(actual) and all(failed==0 for _,failed,_ in actual) and any(p+i+f>0 for p,f,i in actual)
 names=[];names_ok=True
 if index==3:
  names=list(REQUIRED_ORIGIN_TESTS)
  names_ok=all(re.search(r'(?m)^test '+re.escape(n)+r' \.\.\. ok$',text) for n in names)
 ok=(expected is None or actual==expected) and (index not in TEST_COMMANDS or nonempty) and names_ok
 reason=None
 if expected is not None and actual!=expected:reason=f'expected summaries {expected}, got {actual}'
 elif index in TEST_COMMANDS and not nonempty:reason='required test summary absent, empty, or failed'
 elif not names_ok:reason='caller-supplied exact typed-test name absent or not passed'
 return {'expected':expected,'actual':actual,'required':index in TEST_COMMANDS,
         'required_test_names':names,'required_test_names_present':names_ok,'ok':ok,'failure_reason':reason}

def write_execution(guard,rows,preflight,status,freeze=None):
 guard.write_json(OUT/'execution.json',{
  'schema':'recover-proved-local-source-types-validation-build-root-v1',
  'validation_runner':{'path':str(Path(__file__).resolve()),'sha256':sha_file(Path(__file__).resolve())},
  'guarded_runner_template':{'path':str(TEMPLATE),'sha256':TEMPLATE_SHA256},
  'status':status,'source_commit_base_expected':SOURCE_COMMIT_BASE,
  'uncommitted_local_source_types_product':True,
  'environment_overrides':guard.ENV_VALUES,
  'required_origin_tests':list(REQUIRED_ORIGIN_TESTS),
  'expected_origin_test_count':EXPECTED_ORIGIN_COUNT,'expected_library_test_count':EXPECTED_LIB_COUNT,
  'guards':{'minimum_free_bytes':guard.FREE_LIMIT,'maximum_target_bytes':guard.TARGET_LIMIT},
  'preflight':preflight,'commands':rows,'freeze':freeze})

def main()->int:
 global SOURCE_COMMIT_BASE,EXPECTED_LIB_COUNT,EXPECTED_ORIGIN_COUNT,REQUIRED_ORIGIN_TESTS,COMMANDS,EXPECTED_SUMMARIES
 ap=argparse.ArgumentParser(description=__doc__)
 ap.add_argument('--source-base',required=True,help='expected clean base; source-types product remains an uncommitted local change')
 ap.add_argument('--expected-lib-count',required=True,type=int)
 ap.add_argument('--expected-origin-test-count',required=True,type=int)
 ap.add_argument('--new-test-name',action='append',required=True,help='exact names from cf12_proved_local_source_types.rs')
 a=ap.parse_args()
 if not re.fullmatch(r'[0-9a-f]{40}',a.source_base):raise SystemExit('--source-base must be lowercase 40-hex')
 if a.expected_lib_count!=337:raise SystemExit('expected library count must match currently accepted 337 baseline; update only with reviewed evidence')
 if a.expected_origin_test_count<=0 or len(a.new_test_name)!=a.expected_origin_test_count or len(set(a.new_test_name))!=len(a.new_test_name):raise SystemExit('new origin names must be distinct and match explicit positive count')
 if any(not re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*',x) for x in a.new_test_name):raise SystemExit('invalid Rust test identifier')
 SOURCE_COMMIT_BASE=a.source_base;EXPECTED_LIB_COUNT=a.expected_lib_count;EXPECTED_ORIGIN_COUNT=a.expected_origin_test_count;REQUIRED_ORIGIN_TESTS=tuple(sorted(a.new_test_name))
 guard=load_guard();guard.ROOT=ROOT;guard.HERE=TASK;guard.RESULTS=RESULTS;guard.OUT=OUT;guard.CLI_PATH=CLI_PATH;guard.METADATA_PATH=METADATA_PATH;guard.SOURCE_COMMIT_BASE=SOURCE_COMMIT_BASE
 guard.PRODUCT_PATHS=set(PRODUCT_PATHS);guard.TEST_PATHS=set(TEST_PATHS)
 typed_path=ROOT/'crates/jarde-java/tests/cf12_proved_local_source_types.rs'
 if {n for n,_ in test_functions(typed_path)}!=set(REQUIRED_ORIGIN_TESTS):raise RuntimeError('explicit typed test names do not exactly match typed integration source')
 literal_paths=discovered_literal_paths();guard.CANONICAL_PATHS=literal_paths
 targets=[
  ('typed-origin','jarde-java','cf12_proved_local_source_types'),
  ('declaration','jarde','p3_declarations'),
  ('parameter-slots','jarde','p3_parameter_slots'),
  ('boolean-proof','jarde-java','p3_proved_boolean_conditional_returns'),
  ('guard-precedence','jarde','p3_guard'),
  ('qualifier','jarde','p3_popped_static_qualifier'),
  ('invocation-overload','jarde','p3_invocation_arguments'),
  ('array-slot-reuse','jarde','p3_array_slot_retype_locals'),
  ('pattern-literals','jarde-java','p3_patterns'),
 ]
 COMMANDS=[['cargo','fmt','--all','--','--check'],
  ['cargo','clippy','--workspace','--all-targets','--all-features','--locked','--',*sum((['-A',f'clippy::{x}'] for x in CI_CLIPPY_ALLOW_LINTS),[]),'-D','warnings'],
  ['cargo','test','-p','jarde-java','--lib','--locked']]
 for label,package,target in targets:
  COMMANDS.append(['cargo','test','-p',package,'--test',target,'--locked','--','--nocapture'])
 COMMANDS += [['cargo','test','-p','jarde-reader','--lib','--locked'],['cargo','build','-p','jarde-cli','--locked']]
 # Parse source annotations to establish each pre-existing binary's exact expected totals; do not
 # guess future typed test counts. The typed count/name list stays an explicit caller input.
 for i,(_,pkg,target) in enumerate(targets,start=3):
  src=ROOT/(('crates/jarde-java/tests/' if pkg=='jarde-java' else 'tests/')+target+'.rs')
  found=test_functions(src)
  if not found:raise RuntimeError('selected regression target has no statically discoverable tests: '+str(src))
  EXPECTED_SUMMARIES[i]=[(sum(not ignored for _,ignored in found),0,sum(ignored for _,ignored in found))]
 EXPECTED_SUMMARIES[2]=[(EXPECTED_LIB_COUNT,0,0)]
 EXPECTED_SUMMARIES[12]=[(178,0,0)]
 guard.COMMANDS=COMMANDS;guard.TEST_COMMANDS=TEST_COMMANDS;guard.EXPECTED_SUMMARIES=EXPECTED_SUMMARIES;guard.expected_test_summaries=summary_check
 guard.command_stream=lambda p:{'path':str(p.resolve()),'bytes':p.stat().st_size,'sha256':sha_file(p)}
 if len(PRODUCT_PATHS)!=17 or len(TEST_PATHS)!=10:raise RuntimeError('reviewed source pin path counts changed')
 if OUT.exists() or CLI_PATH.exists() or METADATA_PATH.exists():raise FileExistsError('refusing to overwrite typed CLI/output/metadata')
 if shutil.disk_usage(ROOT).free<guard.FREE_LIMIT or guard.target_bytes()>guard.TARGET_LIMIT:raise RuntimeError('initial v9 disk guard failed')
 preflight={'source_pins_before':None,'source_pins_after':None,'source_commit_base_expected':SOURCE_COMMIT_BASE,
  'uncommitted_local_source_types_product':True,'typed_test_count':len(REQUIRED_ORIGIN_TESTS),
  'selected_regression_targets':[{'label':x,'package':p,'target':t,'source_test_count':len(test_functions(ROOT/(('crates/jarde-java/tests/' if p=='jarde-java' else 'tests/')+t+'.rs')))} for x,p,t in targets],
  'canonical_closure':{'test_source_paths':len(TEST_PATHS),'literal_includes':len(literal_paths),'unique_canonical_files':len(literal_paths)},'jdk23':None}
 OUT.mkdir(parents=True);rows=[]
 try:
  before={'candidate_sources':guard.source_pins(PRODUCT_PATHS),'test_sources':guard.source_pins(TEST_PATHS),'canonical_files':guard.source_pins(literal_paths)}
  preflight['source_pins_before']=before
  env=os.environ.copy();stripped=[]
  for k in guard.STRIPPED_ENV_KEYS:
   if k in env:stripped.append(k);env.pop(k)
  env.update(guard.ENV_VALUES);preflight['stripped_environment_keys']=stripped
  preflight['jdk23']=guard.configure_jdk23(env)
  head=subprocess.run(['git','rev-parse','HEAD'],cwd=ROOT,capture_output=True,check=False)
  (OUT/'git-head.stdout.raw').write_bytes(head.stdout);(OUT/'git-head.stderr.raw').write_bytes(head.stderr)
  actual=head.stdout.decode('ascii',errors='replace').strip()
  preflight['git_head']={'argv':['git','rev-parse','HEAD'],'exit_code':head.returncode,'value':actual,'matches_expected':head.returncode==0 and actual==SOURCE_COMMIT_BASE,
   'stdout':guard.command_stream(OUT/'git-head.stdout.raw'),'stderr':guard.command_stream(OUT/'git-head.stderr.raw')}
  if not preflight['git_head']['matches_expected']:raise RuntimeError('HEAD differs from explicit --source-base')
  cleanup=[];guard.clean_check_only_metadata(cleanup);preflight['check_only_metadata_cleanup']=cleanup
  write_execution(guard,rows,preflight,'running')
  for i,argv in enumerate(COMMANDS):
   row=guard.run_command(i,[str(x) for x in argv],env);rows.append(row);write_execution(guard,rows,preflight,'running');print(json.dumps(row),flush=True)
   is_test=argv[:2]==['cargo','test']
   if row['exit_code']!=0 or row['guard_stop'] is not None or (is_test and (not row['test_summary_check'] or not row['test_summary_check']['ok'])):raise RuntimeError(f'validation command {i} failed or did not meet exact test expectations')
   row['cleanup']={'test_binaries':[],'check_only_metadata':[]}
   if i==1:guard.clean_check_only_metadata(row['cleanup']['check_only_metadata'])
   if is_test:guard.clean_successful_test_binary(i,row['cleanup']['test_binaries'])
   write_execution(guard,rows,preflight,'running')
  after={'candidate_sources':guard.source_pins(PRODUCT_PATHS),'test_sources':guard.source_pins(TEST_PATHS),'canonical_files':guard.source_pins(literal_paths)}
  preflight['source_pins_after']=after
  if before!=after:raise RuntimeError('pinned source inputs changed during validation')
  binary=ROOT/'target/debug/jarde-cli'
  if not binary.is_file():raise FileNotFoundError('successful CLI build produced no jarde-cli binary')
  CLI_PATH.parent.mkdir(parents=True,exist_ok=True);tmp=CLI_PATH.with_name('.'+CLI_PATH.name+f'.tmp-{os.getpid()}')
  shutil.copyfile(binary,tmp);os.chmod(tmp,0o555);os.replace(tmp,CLI_PATH)
  freeze={'cli_path':str(CLI_PATH),'cli_sha256':sha_file(CLI_PATH),'cli_mode':oct(stat.S_IMODE(CLI_PATH.stat().st_mode)),
   'metadata_path':str(METADATA_PATH),'source_commit_base':SOURCE_COMMIT_BASE,'uncommitted_local_source_types_product':True,
   'validation_runner':{'path':str(Path(__file__).resolve()),'sha256':sha_file(Path(__file__).resolve())},
   'guarded_runner_template':{'path':str(TEMPLATE),'sha256':TEMPLATE_SHA256},
   'product_paths':sorted(PRODUCT_PATHS),'test_paths':sorted(TEST_PATHS),'canonical_paths':sorted(literal_paths),
   'required_origin_tests':list(REQUIRED_ORIGIN_TESTS),'expected_origin_test_count':EXPECTED_ORIGIN_COUNT,'expected_library_test_count':EXPECTED_LIB_COUNT}
  write_execution(guard,rows,preflight,'validation-passed-cli-frozen',freeze)
  metadata={'schema':'recover-proved-local-source-types-candidate-cli-root-v1',**freeze,
   'candidate_sources':after['candidate_sources'],'test_sources':after['test_sources'],'canonical_files':after['canonical_files'],
   'build_result_sha256':sha_file(OUT/'execution.json')}
  with METADATA_PATH.open('x',encoding='utf-8') as f:f.write(json.dumps(metadata,ensure_ascii=False,indent=2)+'\n')
  return 0
 except Exception as e:
  write_execution(guard,rows,{**preflight,'failure':f'{type(e).__name__}: {e}'},'failed')
  print(f'validation stopped: {type(e).__name__}: {e}',file=sys.stderr);return 1

if __name__=='__main__':raise SystemExit(main())
