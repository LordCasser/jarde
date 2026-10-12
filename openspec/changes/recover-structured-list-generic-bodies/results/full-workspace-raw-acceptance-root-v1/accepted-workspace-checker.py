#!/usr/bin/env python3
"""Read-only acceptance of the guarded CF11 full-workspace receipt."""
from __future__ import annotations
import argparse, collections, hashlib, importlib.util, json, re, sys
from pathlib import Path
sys.dont_write_bytecode = True
ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
RUNNER = Path('/private/tmp/jarde-cf11-full-workspace-prep-luna-v3/workspace_runner.py')
EXPECTED_BOOKKEEPING_ERROR = "RuntimeError: required workspace tests did not pass by exact name: ['diagnose_late_analysis_stop_phase_without_assuming_raw_map_is_empty']"
EXPECTED_EXECUTION_SHA256 = 'a230c1a4de86c3d92cc55799bd36be586eacbcffae81f1b5ee8b3df6e01d52c8'
EXPECTED_EXECUTION_PATH = Path('/private/tmp/jarde-cf11-structured-list-full-workspace-root-v2/execution.json')
SEEDS = ('5350648285461741569', '5350648285461741570')
BASE_ENV = {'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always','CARGO_PROFILE_TEST_STRIP':'symbols'}
FREE_LIMIT, TARGET_LIMIT = 5 * 1024**3, 1024**3
SUMMARY = re.compile(r'^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;', re.M)
OUTCOME = re.compile(r'^test (.+) \.\.\. (ok|ignored|FAILED)(?:, .*)?$', re.M)
ANSI = re.compile(r'\x1b\[[0-9;]*m')
RUNNING_TARGET = re.compile(r'^\s*Running\b.*\((target/debug/(?:deps|examples)/[^)\s]+)\)\s*$', re.M)
RUNNING_TESTS = re.compile(r'^running (\d+) tests?$', re.M)
FOCUSED_OK = {
 'nearby_list_controls_refuse_only_unproved_generic_projections',
 'inner_list_option_actual_uses_the_receiver_element_type',
 'fixed_nested_string_body_projects_source_and_preserves_physical_origins',
 'differently_named_string_and_integer_classes_follow_their_list_element_type',
 'exact_list_null_add_removes_only_the_synthetic_object_wrapper',
 'pre_cancel_does_not_install_a_partial_generic_declaration_or_method_projection',
 'budget_stop_during_list_proof_keeps_raw_body_and_installs_no_projection',
}
FOCUSED_EXPECTED = {
 'nearby_list_controls_refuse_only_unproved_generic_projections':'ok',
 'inner_list_option_actual_uses_the_receiver_element_type':'ok',
 'fixed_nested_string_body_projects_source_and_preserves_physical_origins':'ok',
 'differently_named_string_and_integer_classes_follow_their_list_element_type':'ok',
 'exact_list_null_add_removes_only_the_synthetic_object_wrapper':'ok',
 'pre_cancel_does_not_install_a_partial_generic_declaration_or_method_projection':'ok',
 'budget_stop_during_list_proof_keeps_raw_body_and_installs_no_projection':'ok',
 'diagnose_late_analysis_stop_phase_without_assuming_raw_map_is_empty':'ignored',
}
def sha(b): return hashlib.sha256(b).hexdigest()
def need(ok, msg):
 if not ok: raise ValueError(msg)
def load_module(path):
 spec=importlib.util.spec_from_file_location('cf11_workspace_runner',path); need(spec and spec.loader,'cannot load pinned workspace runner')
 mod=importlib.util.module_from_spec(spec); spec.loader.exec_module(mod); return mod

def compiler_test_artifacts(raw):
 text=raw.decode('utf-8','replace'); found=[]
 for line in text.splitlines():
  if not line.startswith('{'): continue
  obj=json.loads(line)
  if obj.get('reason')!='compiler-artifact' or obj.get('profile',{}).get('test') is not True or not obj.get('executable'):
   continue
  target=obj.get('target',{}); kinds=target.get('kind',[])
  need(isinstance(obj.get('package_id'),str) and isinstance(target.get('name'),str) and kinds,
       'compiler-artifact test target is incomplete')
  path=str(obj['executable']).replace('\\','/')
  marker='/target/debug/'
  need(marker in path,'compiler-artifact executable is outside target/debug')
  suffix=path.split(marker,1)[1]
  need(suffix.startswith(('deps/','examples/')),'compiler-artifact executable is outside test output roots')
  found.append({'target':(obj['package_id'],target['name'],kinds[0]),'executable_suffix':suffix})
 need(len(found)==len({item['target'] for item in found}),'duplicate compiler-artifact test target')
 need(len(found)==len({item['executable_suffix'] for item in found}),'duplicate compiler-artifact executable')
 return found

def test_suites_from_raw(stdout, stderr, artifacts):
 out=ANSI.sub('',stdout.decode('utf-8','replace'))
 err=ANSI.sub('',stderr.decode('utf-8','replace'))
 summaries=list(SUMMARY.finditer(out)); outcomes=list(OUTCOME.finditer(out))
 running=list(RUNNING_TARGET.finditer(err)); need(len(running)==len(artifacts),'stderr Running target count differs from compiler-artifact JSON')
 need(len(summaries)==len(artifacts),'raw test summary count differs from compiler-artifact JSON')
 by_suffix={item['executable_suffix']:item['target'] for item in artifacts}
 suites=[]; cursor=0; outcome_cursor=0; seen_targets=[]
 for summary in summaries:
  block=[item for item in outcomes if cursor<=item.start()<summary.start()]
  starts=list(RUNNING_TESTS.finditer(out,cursor,summary.start()))
  need(len(starts)==1,'raw test suite is missing one unambiguous running-count line')
  need(int(starts[0].group(1))==len(block),'raw named test count differs from its running-count line')
  values=[{'name':item.group(1),'status':item.group(2)} for item in block]
  ps,fail,ign=map(int,summary.groups()[1:])
  counts=collections.Counter(item['status'] for item in values)
  need((counts['ok'],counts['FAILED'],counts['ignored'])==(ps,fail,ign),
       'raw named outcomes differ from their target summary totals')
  need(len({item['name'] for item in values})==len(values),'duplicate named outcome within one target suite')
  need(outcome_cursor+len(block)<=len(outcomes) and outcomes[outcome_cursor:outcome_cursor+len(block)]==block,
       'raw named outcome order is inconsistent')
  outcome_cursor+=len(block)
  suite_index=len(suites); running_path=running[suite_index].group(1).replace('\\','/')
  need(running_path.startswith('target/debug/'),'stderr target path is not under target/debug')
  suffix=running_path.split('target/debug/',1)[1]
  target=by_suffix.get(suffix)
  need(target is not None,'stderr Running executable does not match a compiler-artifact JSON executable')
  seen_targets.append(target)
  suites.append({'target':target,'summary':{'status':summary.group(1),'passed':ps,'failed':fail,'ignored':ign},
                 'named_outcomes':values})
  cursor=summary.end()
 need(outcome_cursor==len(outcomes),'raw named outcomes remain outside all target summaries')
 need(len(seen_targets)==len(set(seen_targets)),'one compiler target produced multiple target suites in a batch')
 return suites

def expected_batch_targets(argv, metadata):
 need(len(argv)>=5 and argv[:2]==['cargo','test'] and argv[2]=='-p','batch argv is not a Cargo test command')
 package_name=argv[3]
 packages=[p for p in metadata['packages'] if p['name']==package_name and p['id'] in set(metadata['workspace_members'])]
 need(len(packages)==1,'batch package name does not resolve uniquely in Cargo metadata')
 package=packages[0]; selected=set()
 flags={'--lib':'lib','--bins':'bin','--examples':'example','--benches':'bench'}
 for flag,kind in flags.items():
  if flag in argv: selected.update((package['id'],t['name'],kind) for t in package['targets'] if t['kind'][0]==kind)
 tests=[argv[i+1] for i,item in enumerate(argv[:-1]) if item=='--test']
 need(len(tests)==len(set(tests)),'batch repeats a --test target')
 for name in tests:
  matches=[t for t in package['targets'] if t['kind'][0]=='test' and t['name']==name]
  need(len(matches)==1,'selected integration-test target does not resolve uniquely in metadata')
  selected.add((package['id'],name,'test'))
 need(selected,'batch has no metadata-selected test targets')
 return selected
def main():
 ap=argparse.ArgumentParser(description=__doc__)
 ap.add_argument('--execution',type=Path,required=True); ap.add_argument('--execution-sha256',required=True)
 ap.add_argument('--focused-receipt',type=Path,required=True); ap.add_argument('--focused-receipt-sha256',required=True)
 ap.add_argument('--metadata',type=Path,required=True); ap.add_argument('--out',type=Path,required=True)
 a=ap.parse_args(); ep=a.execution.resolve(strict=True); fp=a.focused_receipt.resolve(strict=True); mp=a.metadata.resolve(strict=True); op=a.out.resolve()
 need(not op.exists(),'refusing to overwrite acceptance output')
 eraw=ep.read_bytes(); fraw=fp.read_bytes(); mraw=mp.read_bytes()
 need(ep==EXPECTED_EXECUTION_PATH and a.execution_sha256==EXPECTED_EXECUTION_SHA256 and sha(eraw)==EXPECTED_EXECUTION_SHA256 and
      sha(fraw)==a.focused_receipt_sha256,'execution evidence is not the pinned immutable same-run receipt or focused hash differs')
 e=json.loads(eraw); f=json.loads(fraw); metadata=json.loads(mraw)
 need(e.get('schema')=='cf11-structured-list-full-workspace-root-v2' and e.get('status')=='failed' and e.get('error')==EXPECTED_BOOKKEEPING_ERROR and
      e.get('workspace_totals') is None and e.get('workspace_summary_count') is None and
      e.get('expected_command_count_including_clean')==195 and e.get('observed_command_count_including_clean')==195 and
      e.get('expected_target_count')==364 and e.get('expected_batch_count')==96 and
      e.get('source_closure_count_at_start')==4822,'workspace receipt is not the one exact late required-name bookkeeping failure')
 need(f.get('schema')=='cf11-structured-list-focused-run-v4' and f.get('status')=='passed_initial_product_validation','focused receipt not successful')
 need(f.get('expected_head')=='e9e3a0ec27e5bf7698bdf4817ff08d38bcb63374' and f.get('actual_head')==f.get('expected_head') and f.get('head_matches_expected') is True,'focused receipt source head differs')
 need(e.get('focused_input_receipt')=={'path':str(fp),'sha256':a.focused_receipt_sha256,'schema':f['schema'],'status':f['status']},'focused receipt binding differs')
 cli=f.get('cli_freeze',{}); need(e.get('candidate_cli')=={k:cli.get(k) for k in ('path','bytes','sha256','mode')},'workspace candidate CLI differs from focused freeze')
 need(f.get('source_manifest_before')==f.get('source_manifest_after'),'focused source closure changed')
 runner=load_module(RUNNER); live=runner.pin_sources(runner.source_paths())
 need(live==e.get('source_pins_before')==e.get('source_pins_after'),'live full source closure differs from workspace receipt')
 focused_pins={x.get('path'):x.get('sha256') for x in f['source_manifest_before']}
 need(focused_pins==live and len(focused_pins)==len(f['source_manifest_before']),'focused manifest is not the same exact source closure')
 need(e.get('source_closure_count_at_start')==len(live),'source count differs from dynamic closure')
 need(e.get('guard_limits')=={'minimum_free_bytes':FREE_LIMIT,'maximum_target_bytes':TARGET_LIMIT,'poll_interval_seconds':1},'guard limits differ')
 need(e.get('rust_min_stack') is None,'default Rust stack was overridden')
 need(e.get('seeds')==list(SEEDS),'the two CI Proptest seeds differ')
 need(not (ROOT/'target').exists(),'workspace target remains after cleanup')
 cleanup=e.get('cleanup',{}); need(cleanup.get('status')=='passed' and cleanup.get('target_absent_after') is True and cleanup.get('pins_unchanged') is True,'finally-clean postcondition failed')
 need(cleanup.get('command',{}).get('argv')==['cargo','clean'] and cleanup.get('command',{}).get('cleanup_only',{}).get('exact_argv')==['cargo','clean'],'cleanup exception is not exact cargo clean')
 inventory=[tuple(x) for x in e.get('target_inventory',[])]
 members=set(metadata.get('workspace_members',[]))
 planned=[(p['name'],t['name'],t['kind'][0]) for p in metadata['packages'] if p['id'] in members for t in p['targets']]
 id_to_name={p['id']:p['name'] for p in metadata['packages'] if p['id'] in members}
 need(len(inventory)==len(set(inventory)) and set(inventory)==set(planned),'recorded target inventory differs from raw Cargo metadata')
 need(e.get('metadata',{}).get('target_count')==len(planned),'metadata target count differs')
 batches=e.get('expected_batch_count'); need(type(batches) is int and batches>0,'invalid dynamic batch count')
 members=set(metadata['workspace_members']); planned_batches=[]
 for package in metadata['packages']:
  if package['id'] not in members: continue
  base=['cargo','test','-p',package['name'],'--all-features','--locked','--message-format','json']
  units=[target for target in package['targets'] if target['kind'][0] in ('lib','bin','example','bench')]
  if units:
   flags=[]
   for kind,flag in (('lib','--lib'),('bin','--bins'),('example','--examples'),('bench','--benches')):
    if any(target['kind'][0]==kind for target in units): flags.append(flag)
   planned_batches.append(base+flags)
  tests=[target for target in package['targets'] if target['kind'][0]=='test']
  for offset in range(0,len(tests),4):
   planned_batches.append(base+sum((['--test',target['name']] for target in tests[offset:offset+4]),[]))
 need(len(planned_batches)==batches,'dynamic Cargo target batch plan differs from receipt')
 commands=e.get('commands'); want_count=2+2*batches+1
 need(want_count==195 and len(inventory)==364 and batches==96,'pinned same-run target/batch plan differs from expected 364/96/195 receipt')
 need(isinstance(commands,list) and len(commands)==want_count==e.get('expected_command_count_including_clean')==e.get('observed_command_count_including_clean'),'command count differs from dynamic two-seed plan')
 need(e.get('runner',{}).get('sha256_before')==e.get('runner',{}).get('sha256_after')==sha(RUNNER.read_bytes()),'workspace runner changed')
 need(commands[0].get('argv')==['cargo','fmt','--all','--','--check'] and commands[0].get('exit_code')==0,'fmt check failed')
 need(commands[1].get('argv')==['cargo','metadata','--no-deps','--format-version','1','--locked'] and commands[1].get('exit_code')==0,'metadata command failed')
 md_raw=Path(mp).read_bytes(); md_stream=commands[1]['streams']['stdout']; md_path=Path(md_stream['path'])
 need(not md_path.is_symlink() and md_path.read_bytes()==md_raw and sha(md_raw)==md_stream['sha256'],'metadata file differs from raw stdout')
 seen=[]; raw_paths=set(); named_by_seed={s:[] for s in SEEDS}; observed_targets={s:[] for s in SEEDS}
 suites_by_seed={s:[] for s in SEEDS}; target_suites_seen={s:set() for s in SEEDS}
 totals=[0,0,0]; summary_count=0; outcome_count=0
 for index,row in enumerate(commands[:-1]):
  need(row.get('index')==index and row.get('exit_code')==0 and row.get('guard_stop') is None,'command failed or indices have a gap')
  expected_env={**BASE_ENV,'PROPTEST_RNG_SEED':row.get('proptest_seed',SEEDS[0])}
  need(row.get('env_overrides')==expected_env,'workspace command environment differs')
  need(row.get('cleanup_only') is None and row.get('peak_target_bytes',TARGET_LIMIT+1)<=TARGET_LIMIT and row.get('free_bytes_after',0)>=FREE_LIMIT,'guarded workspace command exceeded resource limits')
  if index>=2:
   seed=row.get('proptest_seed'); need(seed in SEEDS,'test batch lacks approved seed')
   seed_index=SEEDS.index(seed); batch_index=index-2-seed_index*batches
   need(0<=batch_index<batches and row.get('argv')==planned_batches[batch_index],'workspace batch argv/order differs from metadata plan')
   need(row.get('argv',[])[:2]==['cargo','test'],'non-test command in workspace batch sequence')
   need(row.get('env_overrides',{}).get('PROPTEST_RNG_SEED')==seed,'batch environment seed mismatch')
   need(row.get('peak_target_bytes',TARGET_LIMIT+1)<=TARGET_LIMIT and row.get('free_bytes_after',0)>=FREE_LIMIT and row.get('cleanup_only') is None,'test batch exceeded guarded resource limits')
   outref=row['streams']['stdout']; raw_path=Path(outref['path']); raw=raw_path.read_bytes()
   need(sha(raw)==outref['sha256'] and len(raw)==outref['bytes'],'test stdout hash mismatch')
   errref=row['streams']['stderr']; err_path=Path(errref['path']); err=err_path.read_bytes()
   need(sha(err)==errref['sha256'] and len(err)==errref['bytes'],'test stderr hash mismatch')
   artifacts=compiler_test_artifacts(raw)
   artifact_targets=[item['target'] for item in artifacts]
   need(set(artifact_targets)==expected_batch_targets(row['argv'],metadata),
        'raw compiler-artifact JSON targets differ from this metadata-planned batch')
   stored_targets=[tuple(item) for item in row.get('executed_targets',[])]
   need(stored_targets==artifact_targets,'runner target summary differs from independently parsed compiler-artifact JSON')
   target_suites=test_suites_from_raw(raw,err,artifacts)
   need(all(suite['summary']['status']=='ok' and suite['summary']['failed']==0 for suite in target_suites),
        'one raw target suite failed or reported a failed summary')
   need(all(item['status'] in ('ok','ignored') for suite in target_suites for item in suite['named_outcomes']),
        'one raw named test outcome failed')
   raw_outcomes=[(item['name'],item['status']) for suite in target_suites for item in suite['named_outcomes']]
   named=[(x.get('name'),x.get('status')) for x in row.get('named_test_outcomes',[])]
   need(raw_outcomes==named and row.get('named_test_outcome_count')==len(raw_outcomes),
        'named test outcomes differ from raw target-associated stdout')
   named_by_seed[seed].extend(raw_outcomes)
   for suite in target_suites:
    package_id,name,kind=suite['target']; package_name=id_to_name.get(package_id)
    need(package_name is not None,'compiler-artifact package id is absent from workspace metadata')
    suite_key=(package_name,name,kind)
    need(suite_key not in target_suites_seen[seed],'one target produced more than one test suite in this seed')
    target_suites_seen[seed].add(suite_key)
    observed_targets[seed].append(suite_key)
    suite['target_name_tuple']=suite_key
    suites_by_seed[seed].append(suite)
   raw_summaries=[(suite['summary']['status'],suite['summary']['passed'],suite['summary']['failed'],suite['summary']['ignored']) for suite in target_suites]
   stored=[(x.get('status'),x.get('passed'),x.get('failed'),x.get('ignored')) for x in row.get('observed_summaries',[])]
   need(raw_summaries==stored and len(raw_summaries)==len(artifacts),'summary rows differ from raw stdout or target artifacts')
   for _,passed,failed,ignored in raw_summaries: totals[0]+=passed;totals[1]+=failed;totals[2]+=ignored
   summary_count+=len(raw_summaries);outcome_count+=len(raw_outcomes)
  for name,ref in row.get('streams',{}).items():
   path=Path(ref['path']); need(not path.is_symlink() and path.is_file(),'raw stream missing or symlink')
   raw=path.read_bytes(); need(len(raw)==ref['bytes'] and sha(raw)==ref['sha256'],'raw stream pin mismatch')
   need(path.resolve().is_relative_to(ep.parent),'raw stream escaped execution directory')
   raw_paths.add(path.resolve())
 for seed in SEEDS:
  outcomes=named_by_seed[seed]
  focus_outcomes=[(name,state) for name,state in outcomes if name in FOCUSED_EXPECTED]
  need(len(focus_outcomes)==len(FOCUSED_EXPECTED) and dict(focus_outcomes)==FOCUSED_EXPECTED,'focused named outcomes differ for '+seed)
  need(set(name for name,state in outcomes if state=='ok') >= FOCUSED_OK,'focused passing names absent for '+seed)
  need(len(observed_targets[seed])==len(set(observed_targets[seed])) and set(observed_targets[seed])==set(tuple(x) for x in e['target_inventory']),'targets not run exactly once for '+seed)
  suites=suites_by_seed[seed]
  def selected(package,name,kind):
   return [suite for suite in suites if suite['target_name_tuple']==(package,name,kind)]
  focused_suites=selected('jarde','structured_list_generic_bodies','test')
  need(len(focused_suites)==1,'focused CF11 integration target did not run exactly once for '+seed)
  focused_names=focused_suites[0]['named_outcomes']
  focused_actual={item['name']:item['status'] for item in focused_names}
  need(len(focused_names)==8 and len(focused_actual)==8 and focused_actual==FOCUSED_EXPECTED,
       'focused CF11 target outcomes are not the exact seven-pass/one-ignored set for '+seed)
  java_suites=selected('jarde-java','jarde_java','lib')
  need(len(java_suites)==1 and java_suites[0]['summary']=={'status':'ok','passed':375,'failed':0,'ignored':0}
       and len(java_suites[0]['named_outcomes'])==375,'jarde-java lib is not the exact 375-test passing suite for '+seed)
  java_tails=('complete_fixed_void_census_keeps_phi_and_every_formal_read',
              'fixed_census_refuses_physical_inventory_and_operation_disagreements',
              'fixed_read_is_not_unread_when_the_invoke_target_is_unknown',
              'real_parameter_reassignment_and_handler_facts_refuse_the_list_census')
  for tail in java_tails:
   rows=[item for item in java_suites[0]['named_outcomes'] if item['name'].rsplit('::',1)[-1]==tail]
   need(len(rows)==1 and rows[0]['status']=='ok','new jarde-java census outcome is not matched exactly once: '+tail)
  reader_suites=selected('jarde-reader','jarde_reader','lib')
  need(len(reader_suites)==1,'jarde-reader lib target did not run exactly once for '+seed)
  reader_rows=[item for item in reader_suites[0]['named_outcomes'] if item['name'].rsplit('::',1)[-1]=='repository_class_fixtures_validate_without_false_target_rejections']
  need(len(reader_rows)==1 and reader_rows[0]['status']=='ok','reader census test is not matched exactly once for '+seed)
  p5_suites=selected('jarde','p5_corpus_fingerprint','test')
  need(len(p5_suites)==1,'P5 corpus fingerprint target did not run exactly once for '+seed)
  p5_expected={'corpus_files_match_the_recorded_fingerprint':'ok',
               'the_manifest_matches_its_rendered_classification':'ok',
               'every_dimension_is_carried_by_existing_corpus':'ok',
               'every_acceptance_row_is_indexed_against_existing_corpus':'ok',
               'the_indexed_rows_are_the_rows_the_acceptance_table_names':'ok',
               'regenerate_corpus_fingerprint':'ignored'}
  p5_names=p5_suites[0]['named_outcomes']
  p5_actual={item['name'].rsplit('::',1)[-1]:item['status'] for item in p5_names}
  need(len(p5_names)==6 and len(p5_actual)==6 and p5_actual==p5_expected,
       'P5 fingerprint target outcomes are not the exact five-pass/one-ignored set for '+seed)
 clean=commands[-1]; need(clean.get('env_overrides')=={**BASE_ENV,'PROPTEST_RNG_SEED':SEEDS[-1]},'cleanup environment differs')
 need(clean.get('index')==want_count-1 and clean.get('argv')==['cargo','clean'] and clean.get('exit_code')==0 and clean.get('guard_stop') is None and
      clean.get('cleanup_only',{}).get('exact_argv')==['cargo','clean'] and clean.get('cleanup_only',{}).get('space_precheck_bypassed') is True and
      clean.get('cleanup_only',{}).get('space_monitor_abort_bypassed') is True and e.get('cleanup',{}).get('status')=='passed' and
      e.get('cleanup',{}).get('command')==clean,'cleanup-only command or cleanup markers differ')
 for ref in clean.get('streams',{}).values():
  path=Path(ref['path']); need(not path.is_symlink() and path.is_file() and path.resolve().is_relative_to(ep.parent),'cleanup raw stream missing or escaped')
  raw=path.read_bytes(); need(len(raw)==ref['bytes'] and sha(raw)==ref['sha256'],'cleanup raw stream pin mismatch'); raw_paths.add(path.resolve())
 need(len(raw_paths)==want_count*2,'raw stream inventory differs')
 actual_raws={p.resolve() for p in ep.parent.glob('*.raw') if p.is_file()}
 need(actual_raws==raw_paths,'execution directory contains missing or unreferenced raw streams')
 need(e.get('workspace_totals') is None and e.get('workspace_summary_count') is None,
      'bookkeeping-failed runner unexpectedly wrote aggregate totals; use the unmodified raw evidence only')
 need(totals[1]==0 and all(totals[0]>0 for _ in SEEDS),'raw target tests include failures or no passing tests')
 for seed in SEEDS:
  diagnostic=[(name,state) for name,state in named_by_seed[seed]
              if name.rsplit('::',1)[-1]=='diagnose_late_analysis_stop_phase_without_assuming_raw_map_is_empty']
  need(len(diagnostic)==1 and diagnostic[0][1]=='ignored',
       'the sole runner bookkeeping diagnostic is not exactly one ignored raw outcome for '+seed)
 result={'schema':'cf11-structured-list-full-workspace-acceptance-v3','status':'accepted_local_workspace_only','not_ci_or_product_release_acceptance':True,
  'execution_sha256':a.execution_sha256,'execution_runner_status':'failed','execution_runner_error':e['error'],
  'runner_bookkeeping':{'status':'failed_only_at_required_name_summary','exact_error':EXPECTED_BOOKKEEPING_ERROR,
    'sole_misclassified_name':'diagnose_late_analysis_stop_phase_without_assuming_raw_map_is_empty',
    'correct_outcome':'ignored'},
  'raw_execution_result':'all_same_run_targets_passed','execution_aggregate_totals_were_unwritten':True,
  'focused_receipt_sha256':a.focused_receipt_sha256,'source_pin_count':len(live),'target_count':len(planned),'batch_count_per_seed':batches,'seeds':list(SEEDS),
  'target_associated_suite_count_per_seed':{seed:len(suites_by_seed[seed]) for seed in SEEDS},
  'matched_focused_gates_per_seed':{seed:{'structured_list_generic_bodies':'7 passed / 1 ignored','jarde_java_lib':'375 passed','reader_census':'passed once','p5_corpus_fingerprint':'5 passed / 1 ignored'} for seed in SEEDS},
  'test_totals':{'passed':totals[0],'failed':totals[1],'ignored':totals[2],'target_summary_count':summary_count,'named_test_outcome_count':outcome_count},
  'raw_stream_count':len(raw_paths),'target_absent':True}
 op.parent.mkdir(parents=True,exist_ok=True); op.write_text(json.dumps(result,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
 print(json.dumps(result,indent=2,ensure_ascii=False)); return 0
if __name__=='__main__':
 try: raise SystemExit(main())
 except Exception as exc: print(f'acceptance failed: {type(exc).__name__}: {exc}',file=sys.stderr); raise SystemExit(1)
