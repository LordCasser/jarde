#!/usr/bin/env python3
"""Guarded validation and private CLI freeze for conditional switch fallthrough."""
from __future__ import annotations
import argparse, datetime, hashlib, importlib.util, json, os, re, shutil, stat, subprocess, sys
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
TASK=Path(__file__).resolve().parent
RESULTS=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results'
OUT=RESULTS/'validation-build-root-v2'
CLI_PATH=Path('/private/tmp/jarde-proved-conditional-switch-cli-v1')
METADATA_PATH=RESULTS/'candidate-cli-root-v1.json'
TEMPLATE=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
TEMPLATE_SHA256='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SOURCE_COMMIT_BASE='2d70da515896c25ce022b8c28f4935ff2e105026'
EXPECTED_LIB_COUNT:int|None=None
CI_CLIPPY_ALLOW_LINTS=(
 'too_many_arguments','cloned_ref_to_slice_refs','collapsible_if','type_complexity','len_zero',
 'needless_option_as_deref','needless_borrow','useless_conversion','large_enum_variant','question_mark',
 'comparison_to_empty','op_ref','manual_range_patterns','if_same_then_else','filter_map_bool_then',
 'filter_next','unneeded_struct_pattern','redundant_guards','map_identity','redundant_slicing',
 'unnecessary_get_then_check','unnecessary_unwrap','redundant_locals','replace_box','map_clone',
 'unnecessary_mut_passed','single_element_loop','unnecessary_to_owned','needless_lifetimes')
PRODUCT_PATHS={
 'Cargo.toml','Cargo.lock','crates/jarde-jvm/Cargo.toml','crates/jarde-reader/Cargo.toml',
 'crates/jarde-query/Cargo.toml','crates/jarde-java/Cargo.toml','crates/jarde-cli/Cargo.toml',
 'crates/jarde-java/src/region.rs','crates/jarde-java/src/build.rs','crates/jarde-java/src/emit.rs',
 'crates/jarde-java/src/report.rs','crates/jarde-java/src/lib.rs','src/class_source.rs','src/facade.rs',
 'src/lib.rs','crates/jarde-cli/src/main.rs','crates/jarde-cli/src/task.rs',
}
TEST_PATHS={
 '.github/workflows/ci.yml',
 'crates/jarde-java/tests/cf12_proved_local_source_types.rs',
 'crates/jarde-java/tests/p3_conditional_switch_fallthrough.rs',
 'crates/jarde-java/tests/p3_conditional_switch_boundary_rejection.rs',
 'crates/jarde-java/tests/p3_conditional_switch_scope_boundaries.rs',
 'crates/jarde-java/tests/p3_proved_boolean_conditional_returns.rs',
 'crates/jarde-java/tests/p3_patterns.rs',
 'tests/p3_declarations.rs','tests/p3_parameter_slots.rs','tests/p3_guard.rs',
 'tests/p3_popped_static_qualifier.rs','tests/p3_invocation_arguments.rs',
 'tests/p3_array_slot_retype_locals.rs',
 'tests/p3_switch_fallthrough.rs','tests/p3_string_switch_projection.rs',
 'tests/p3_switch_loop_exits.rs','tests/p3_switch_arm_loop_exits.rs',
 'tests/p3_switch_forward_join.rs','tests/p3_switch_value.rs','tests/p3_char_switch.rs',
}
INCLUDE_RE=re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')
TEST_RE=re.compile(r'(?ms)^\s*#\[test\]\s*((?:#\[[^\]]+\]\s*)*)fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(')
SUMMARY_RE=re.compile(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored')
REQUIRED_TESTS={
 'crates/jarde-java/src/region.rs':('cf12_switch_certificate_accepts_real_paths_and_rejects_edge_row_variants',),
 'crates/jarde-java/src/build.rs':('cf12_switch_break_builder_requires_the_active_nearest_switch',),
 'crates/jarde-java/tests/cf12_proved_local_source_types.rs':(
  'cf12_charat_stored_value_drives_char_source_type_and_full_origins',
  'cf12_null_first_direct_string_writes_drive_string_source_type_and_full_origins',
  'cf12_recovery_budget_stop_and_cancellation_publish_no_partial_source',
  'typed_boundary_four_char_seeds_have_specific_types_and_full_origins_in_both_debug_profiles',
  'typed_boundary_range_arithmetic_and_input_copy_remain_int_in_both_debug_profiles',
  'typed_boundary_null_first_exact_string_stays_string_in_both_debug_profiles',
  'typed_boundary_mixed_all_null_and_unknown_reference_stay_object_in_both_debug_profiles',
  'typed_boundary_real_slot_conflict_is_preserved_as_the_exact_fallback_in_default_and_all',
  'typed_boundary_proof_budget_stops_before_publication_at_observed_sites',),
 'crates/jarde-java/tests/p3_conditional_switch_fallthrough.rs':(
  'cf12_conditional_switch_keeps_exact_case_and_exit_sources',
  'cf12_recovery_with_pre_cancelled_public_budget_publishes_no_partial_artifact',
  'cf12_switch_proof_stops_at_observed_analysis_step_boundaries_without_an_artifact',),
 'crates/jarde-java/tests/p3_conditional_switch_boundary_rejection.rs':(
  'nonadjacent_conditional_switch_is_refused_by_the_public_caller',
  'pre_cancelled_public_recovery_publishes_no_partial_artifact',),
 'crates/jarde-java/tests/p3_conditional_switch_scope_boundaries.rs':(
  'loop_nested_switch_and_caught_edges_remain_conservative',
  'terminal_return_and_throw_keep_exact_text_and_physical_origins',),
}
COMMANDS=[]
TEST_COMMANDS=set()
EXPECTED_SUMMARIES={}
EXPECTED_TEST_NAMES={}

def sha_file(p:Path)->str:
 h=hashlib.sha256()
 with p.open('rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
 return h.hexdigest()

def load_guard():
 if sha_file(TEMPLATE)!=TEMPLATE_SHA256:raise RuntimeError('pinned v9 command guard SHA mismatch')
 spec=importlib.util.spec_from_file_location('conditional_validation_guard_v9',TEMPLATE)
 if spec is None or spec.loader is None:raise RuntimeError('cannot load v9 command guard')
 module=importlib.util.module_from_spec(spec);sys.modules[spec.name]=module;spec.loader.exec_module(module)
 return module

def test_functions(path:Path)->list[tuple[str,bool]]:
 return [(name,'ignore' in attrs) for attrs,name in TEST_RE.findall(path.read_text(encoding='utf-8'))]

def discovered_literal_paths()->set[str]:
 found=set()
 for rel in sorted(PRODUCT_PATHS|TEST_PATHS):
  if rel.endswith('.rs'):
   src=(ROOT/rel).read_text(encoding='utf-8')
   for value in INCLUDE_RE.findall(src):
    p=(ROOT/rel).parent.joinpath(value).resolve()
    if not p.is_file() or not p.is_relative_to(ROOT.resolve()):
     raise RuntimeError('literal Rust include is missing or outside repository: '+rel+' -> '+value)
    found.add(p.relative_to(ROOT).as_posix())
 return found

def selected_regression_groups():
 return [
  ('switch-string-loop', 'jarde', ('p3_switch_fallthrough','p3_string_switch_projection','p3_switch_loop_exits','p3_switch_arm_loop_exits')),
  ('switch-join-value-selector', 'jarde', ('p3_switch_forward_join','p3_switch_value','p3_char_switch')),
 ]

def test_argv(package:str,target:str,filter_name:str|None=None):
 argv=['cargo','test','-p',package,'--locked']
 if target=='--lib': argv.append('--lib')
 else: argv += ['--test',target]
 if filter_name is not None: argv.append(filter_name)
 return argv+['--','--nocapture']

def expected_for_target(pkg:str,target:str):
 source=ROOT/(('crates/jarde-java/tests/' if pkg=='jarde-java' else 'tests/')+target+'.rs')
 found=test_functions(source)
 if not found:raise RuntimeError('selected test target has no discoverable tests: '+str(source))
 return (sum(not ignored for _,ignored in found),0,sum(ignored for _,ignored in found))

def summary_check(index:int,stdout:bytes)->dict:
 text=stdout.decode('utf-8',errors='replace')
 actual=[tuple(map(int,m)) for m in SUMMARY_RE.findall(text)]
 expected=EXPECTED_SUMMARIES.get(index)
 names=EXPECTED_TEST_NAMES.get(index,())
 names_ok=all(re.search(r'(?m)^test [^\n]*'+re.escape(name)+r' \.\.\. ok$',text) for name in names)
 nonempty=bool(actual) and all(failed==0 for _,failed,_ in actual) and any(p+i+f>0 for p,f,i in actual)
 ok=(expected is None or actual==expected) and (index not in TEST_COMMANDS or nonempty) and names_ok
 reason=None
 if expected is not None and actual!=expected:reason=f'expected summaries {expected}, got {actual}'
 elif index in TEST_COMMANDS and not nonempty:reason='required test summary absent, empty, or failed'
 elif not names_ok:reason='caller-selected exact test name absent or not passed'
 return {'expected':expected,'actual':actual,'required':index in TEST_COMMANDS,
         'required_test_names':list(names),'required_test_names_present':names_ok,'ok':ok,'failure_reason':reason}

def write_execution(guard,rows,preflight,status,freeze=None):
 guard.write_json(OUT/'execution.json',{
  'schema':'recover-proved-conditional-switch-fallthrough-validation-build-root-v2',
  'validation_runner':{'path':str(Path(__file__).resolve()),'sha256':sha_file(Path(__file__).resolve())},
  'guarded_runner_template':{'path':str(TEMPLATE),'sha256':TEMPLATE_SHA256},
  'status':status,'source_commit_base_expected':SOURCE_COMMIT_BASE,
  'uncommitted_local_conditional_switch_product':True,
  'environment_overrides':guard.ENV_VALUES,
  'required_tests':{path:list(names) for path,names in REQUIRED_TESTS.items()},
  'expected_library_test_count':EXPECTED_LIB_COUNT,
  'guards':{'minimum_free_bytes':guard.FREE_LIMIT,'maximum_target_bytes':guard.TARGET_LIMIT},
  'preflight':preflight,'commands':rows,'freeze':freeze})

def main()->int:
 global EXPECTED_LIB_COUNT,COMMANDS,TEST_COMMANDS,EXPECTED_SUMMARIES,EXPECTED_TEST_NAMES
 ap=argparse.ArgumentParser(description=__doc__)
 ap.add_argument('--source-base',required=True)
 ap.add_argument('--expected-lib-count',required=True,type=int,
                 help='exact library test count observed by root for this candidate; not mirrored in this runner')
 a=ap.parse_args()
 if a.source_base!=SOURCE_COMMIT_BASE:raise SystemExit('--source-base must be 2d70da515896c25ce022b8c28f4935ff2e105026')
 if a.expected_lib_count<=0:raise SystemExit('--expected-lib-count must be positive')
 EXPECTED_LIB_COUNT=a.expected_lib_count
 guard=load_guard();guard.ROOT=ROOT;guard.HERE=TASK;guard.RESULTS=RESULTS;guard.OUT=OUT;guard.CLI_PATH=CLI_PATH;guard.METADATA_PATH=METADATA_PATH;guard.SOURCE_COMMIT_BASE=SOURCE_COMMIT_BASE
 guard.PRODUCT_PATHS=set(PRODUCT_PATHS);guard.TEST_PATHS=set(TEST_PATHS)
 for rel,names in REQUIRED_TESTS.items():
  found={name for name,_ in test_functions(ROOT/rel)}
  exact=not rel.endswith(('/src/region.rs','/src/build.rs'))
  drift=(found!=set(names)) if exact else (not set(names)<=found)
  if drift:
   raise RuntimeError('selected test inventory drifted: '+rel+' expected '+repr(sorted(names))+' got '+repr(sorted(found)))
 literal_paths=discovered_literal_paths();guard.CANONICAL_PATHS=literal_paths
 if len(PRODUCT_PATHS)!=17 or len(TEST_PATHS)!=20:raise RuntimeError('reviewed product/test source path count changed')
 COMMANDS=[['cargo','fmt','--all','--','--check'],
  ['cargo','clippy','--workspace','--all-targets','--all-features','--locked','--',*sum((['-A',f'clippy::{x}'] for x in CI_CLIPPY_ALLOW_LINTS),[]),'-D','warnings'],
  ['cargo','test','-p','jarde-java','--lib','--locked']]
 COMMANDS.append(test_argv('jarde-java','--lib','cf12_switch_certificate_accepts_real_paths_and_rejects_edge_row_variants'))
 COMMANDS.append(test_argv('jarde-java','--lib','cf12_switch_break_builder_requires_the_active_nearest_switch'))
 suites=[('jarde-java','cf12_proved_local_source_types'),
         ('jarde-java','p3_conditional_switch_fallthrough'),
         ('jarde-java','p3_conditional_switch_boundary_rejection'),
         ('jarde-java','p3_conditional_switch_scope_boundaries')]
 for package,target in suites: COMMANDS.append(test_argv(package,target))
 for _,package,targets in selected_regression_groups():
  argv=['cargo','test','-p',package]
  for target in targets:argv+=['--test',target]
  COMMANDS.append(argv+['--locked','--','--nocapture'])
 COMMANDS += [['cargo','test','-p','jarde-reader','--lib','--locked'],['cargo','build','-p','jarde-cli','--locked']]
 TEST_COMMANDS=set(range(2,12))
 EXPECTED_SUMMARIES[2]=[(EXPECTED_LIB_COUNT,0,0)]
 EXPECTED_SUMMARIES[3]=[(1,0,0)];EXPECTED_TEST_NAMES[3]=REQUIRED_TESTS['crates/jarde-java/src/region.rs']
 EXPECTED_SUMMARIES[4]=[(1,0,0)];EXPECTED_TEST_NAMES[4]=REQUIRED_TESTS['crates/jarde-java/src/build.rs']
 for index,(package,target) in enumerate(suites,start=5):
  EXPECTED_SUMMARIES[index]=[expected_for_target(package,target)]
  EXPECTED_TEST_NAMES[index]=REQUIRED_TESTS['crates/jarde-java/tests/'+target+'.rs']
 for index,(_,package,targets) in enumerate(selected_regression_groups(),start=9):
  EXPECTED_SUMMARIES[index]=[expected_for_target(package,target) for target in targets]
 EXPECTED_SUMMARIES[11]=[(178,0,0)]
 guard.COMMANDS=COMMANDS;guard.TEST_COMMANDS=TEST_COMMANDS;guard.EXPECTED_SUMMARIES=EXPECTED_SUMMARIES;guard.expected_test_summaries=summary_check
 guard.command_stream=lambda p:{'path':str(p.resolve()),'bytes':p.stat().st_size,'sha256':sha_file(p)}
 if OUT.exists() or CLI_PATH.exists() or METADATA_PATH.exists():raise FileExistsError('refusing to overwrite conditional CLI/output/metadata')
 if shutil.disk_usage(ROOT).free<guard.FREE_LIMIT or guard.target_bytes()>guard.TARGET_LIMIT:raise RuntimeError('initial v9 disk guard failed')
 preflight={'source_pins_before':None,'source_pins_after':None,'source_commit_base_expected':SOURCE_COMMIT_BASE,
  'uncommitted_local_conditional_switch_product':True,'expected_library_test_count':EXPECTED_LIB_COUNT,'java_environment':preflight['java_environment'],
  'selected_test_suites':[{'package':p,'target':t,'test_count':len(test_functions(ROOT/(('crates/jarde-java/tests/' if p=='jarde-java' else 'tests/')+t+'.rs')))} for p,t in suites],
  'selected_unit_tests':{path:list(names) for path,names in REQUIRED_TESTS.items() if path.endswith('/src/region.rs') or path.endswith('/src/build.rs')},
  'regression_groups':[{'label':label,'package':package,'targets':list(targets)} for label,package,targets in selected_regression_groups()],
  'canonical_closure':{'product_source_paths':len(PRODUCT_PATHS),'test_source_paths':len(TEST_PATHS),
                       'rust_source_literal_includes':len(literal_paths),'unique_canonical_files':len(literal_paths)},'jdk23':None}
 OUT.mkdir(parents=True);rows=[]
 try:
  before={'candidate_sources':guard.source_pins(PRODUCT_PATHS),'test_sources':guard.source_pins(TEST_PATHS),'canonical_files':guard.source_pins(literal_paths)}
  preflight['source_pins_before']=before
  env=os.environ.copy();stripped=[]
  for k in guard.STRIPPED_ENV_KEYS:
   if k in env:stripped.append(k);env.pop(k)
  env.update(guard.ENV_VALUES);preflight['stripped_environment_keys']=stripped
  preflight['jdk23']=guard.configure_jdk23(env)
  jdk23_home=Path(preflight['jdk23']['java_home']).resolve()
  env['JAVA_HOME']=str(jdk23_home)
  env['PATH']=str(jdk23_home/'bin')+os.pathsep+env.get('PATH','')
  env['LC_ALL']='C';env['TZ']='UTC'
  preflight['java_environment']={'JAVA_HOME':env['JAVA_HOME'],'PATH':env['PATH'],'LC_ALL':env['LC_ALL'],'TZ':env['TZ']}
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
   'metadata_path':str(METADATA_PATH),'source_commit_base':SOURCE_COMMIT_BASE,'uncommitted_local_conditional_switch_product':True,
   'validation_runner':{'path':str(Path(__file__).resolve()),'sha256':sha_file(Path(__file__).resolve())},
   'guarded_runner_template':{'path':str(TEMPLATE),'sha256':TEMPLATE_SHA256},
   'product_paths':sorted(PRODUCT_PATHS),'test_paths':sorted(TEST_PATHS),'canonical_paths':sorted(literal_paths),
   'required_tests':{path:list(names) for path,names in REQUIRED_TESTS.items()},'expected_library_test_count':EXPECTED_LIB_COUNT}
  write_execution(guard,rows,preflight,'validation-passed-cli-frozen',freeze)
  metadata={'schema':'recover-proved-conditional-switch-fallthrough-candidate-cli-root-v1',**freeze,
   'candidate_sources':after['candidate_sources'],'test_sources':after['test_sources'],'canonical_files':after['canonical_files'],
   'build_result_sha256':sha_file(OUT/'execution.json')}
  with METADATA_PATH.open('x',encoding='utf-8') as f:f.write(json.dumps(metadata,ensure_ascii=False,indent=2)+'\n')
  return 0
 except Exception as e:
  write_execution(guard,rows,{**preflight,'failure':f'{type(e).__name__}: {e}'},'failed')
  print(f'validation stopped: {type(e).__name__}: {e}',file=sys.stderr);return 1

if __name__=='__main__':raise SystemExit(main())
