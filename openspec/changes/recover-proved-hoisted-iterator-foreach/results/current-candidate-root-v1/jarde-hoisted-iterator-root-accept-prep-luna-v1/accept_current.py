#!/usr/bin/env python3
"""Independent, read-only acceptor for the fixed TestNestedLoops capture."""
from __future__ import annotations
import hashlib, importlib.util, json, sys
from pathlib import Path
sys.dont_write_bytecode = True
ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
CAP = Path('/private/tmp/jarde-hoisted-iterator-current-candidate-root-v1')
EXEC, CHECK = CAP/'execution.json', CAP/'checker-root-v7.json'
LAUNCH = Path('/private/tmp/jarde-hoisted-iterator-current-candidate-root-v1-launcher-receipt.json')
FOCUS = Path('/private/tmp/jarde-hoisted-iterator-focused-root-v2/execution.json')
BUILD = Path('/private/tmp/jarde-hoisted-iterator-current-candidate-root-v1-build-subreceipt.json')
CLI = Path('/private/tmp/jarde-hoisted-iterator-candidate-cli-root-v2')
INPUTS = Path('/private/tmp/jarde-cf11-testnestedloops-inputs-root-v1')
INPUT_PINS = INPUTS/'frozen-pins.json'
SOURCE_PREP_PINS = Path('/private/tmp/jarde-cf11-testnestedloops-input-luna-v3/pins.json')
SOURCE_CLOSURE = Path('/private/tmp/jarde-constructor-conditional-own-ci-verifier-prep-luna-v2/verify-own-ci.py')
CHECKER = Path('/private/tmp/jarde-for-condition-checker-root-v7/check_controls.py')
BASELINE_CONFIG = Path('/private/tmp/jarde-cf11-testnestedloops2-baseline-root-v1/execution.json')
OUT = Path('/private/tmp/jarde-hoisted-iterator-root-accept-prep-luna-v1/accepted-current.json')
SHA = {
 'execution':'92f634dc3f062c2e0807d6b7cd3cde1c152f3bfe320e85a0f350d7ada5b958a9',
 'checker':'670ca7ce9b68c98a6001b8d8efcebb07ed0bb444326b406535c5788e68c6b2d9',
 'launch':'5e42b3c4636d606d14e190f9d214ff2cb5708b62f775d76b16a1574513bae6be',
 'focus':'69206e83ccb235ce5318b161c8398fc70e3da6ddb40b377217415af0a9e90092',
 'build':'9139b6e3301955fe62c4171063787a5a727f84118e2b816624db2e480a7173e5',
 'cli':'8bdb868ab58e4c9196970438c71de529996c66ba826c84fe216794d328fdfc9a',
 'checker_tool':'a05acce687bf0e22bfab9eb8ae00520d53a006b740732f4b0041182ce72a464a',
 'source_closure':'ac511e1560c5da57fca5fcf4d37e837b726b4e20f2327014b9f29394870f0d1e',
 'baseline_config':'a7791a753850c636aa0c2d5fee5cce5b0a79434ef539eb486f5f4c8499f56951',
 'input_pins':'850f390681d35a98cbaed1fb8e5c714dd5e0e698dd7f18f8976950c9c52a4099',
 'source_prep':'d09a4acb1e4a5a0789e06643028f53b1aa3103a2c5ea15287710e81d65c2273b',
}
def digest(raw: bytes) -> str: return hashlib.sha256(raw).hexdigest()
def pin(path: Path) -> dict:
 raw=path.read_bytes(); return {'path':str(path.resolve()),'bytes':len(raw),'sha256':digest(raw)}
def need(ok, message):
 if not ok: raise ValueError(message)
def load(path): return json.loads(path.read_text(encoding='utf-8'))
def exact_file(path, want, label): need(path.is_file() and digest(path.read_bytes())==want, f'{label} SHA changed')
def import_module(path, name):
 spec=importlib.util.spec_from_file_location(name,path); need(spec and spec.loader,f'cannot import {path}')
 module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module
def main():
 need(not OUT.exists(),'refuse to overwrite acceptance manifest')
 for role,path in [('execution',EXEC),('checker',CHECK),('launch',LAUNCH),('focus',FOCUS),('build',BUILD),('cli',CLI),('checker_tool',CHECKER),('source_closure',SOURCE_CLOSURE),('baseline_config',BASELINE_CONFIG),('input_pins',INPUT_PINS),('source_prep',SOURCE_PREP_PINS)]: exact_file(path,SHA[role],role)
 need(CLI.stat().st_size==61314560 and (CLI.stat().st_mode&0o777)==0o555,'candidate CLI bytes/mode changed')
 e,c,l,f,b=map(load,(EXEC,CHECK,LAUNCH,FOCUS,BUILD))
 need(f['schema']=='hoisted-iterator-focused-root-v2' and f['status']=='passed_focused_candidate_only','focused parent schema/status changed')
 need(f['target_absent'] is True and f['runner_sha256_before']==f['runner_sha256_after'],'focused build cleanup/source binding failed')
 before,after=f['source_pins_before'],f['source_pins_after']; need(isinstance(before,dict) and len(before)==4729 and before==after,'focused source closure not frozen')
 cli_obj={'path':str(CLI),'bytes':61314560,'sha256':SHA['cli'],'mode':'0o555'}; need(f['candidate_cli']==cli_obj and b['candidate_cli']==cli_obj,'focused/subreceipt CLI binding differs')
 build_rows=[r for r in f['commands'] if r.get('argv')==['cargo','build','-p','jarde-cli','--bin','jarde-cli','--locked']]
 need(len(build_rows)==1 and build_rows[0]['index']==6 and build_rows[0]['exit_code']==0 and build_rows[0]['guard_stop'] is None,'focused receipt command index 6 is not successful build')
 need(b['schema']=='hoisted-iterator-candidate-cli-build-subreceipt-v1' and b['status']=='passed_candidate_build_only','derived subreceipt schema/status differs')
 need(b['parent_focused_receipt']=={'path':str(FOCUS),'bytes':FOCUS.stat().st_size,'sha256':SHA['focus'],'schema':f['schema'],'status':f['status']},'derived receipt does not preserve focused parent identity/status')
 need(b['build_command']['index']==6 and b['build_command']['argv']==build_rows[0]['argv'] and b['build_command']['exit_code']==0 and b['build_command']['guard_stop'] is None,'derived command differs from focused command')
 need(b['source_pins_before']==before and b['source_pins_after']==after and b['target_absent'] is True,'derived source/cleanup evidence differs')
 for stream,row in build_rows[0]['streams'].items():
  raw=Path(row['path']).read_bytes(); need(len(raw)==row['bytes'] and digest(raw)==row['sha256'],'focused cargo build raw stream differs')
  need(b['build_command']['streams'][stream]==row,'derived cargo stream pin differs')
 need(e['schema']=='cf11-target-full-source-capture-v2' and e['status']=='observed' and e['source_tool_inputs_unchanged'] is True,'capture schema/status/source pins differ')
 need(c['schema']=='cf11-target-full-source-observation-v2' and c['status']=='verified_observation_capture' and c['execution']==pin(EXEC),'checker capture status/execution binding differs')
 need(l['status']=='complete' and l['collector_exit_code']==l['checker_exit_code']==0 and l['target_absent_after'] is True,'launcher did not finish cleanly')
 need(l['focused_receipt']=={'path':str(FOCUS),'sha256':SHA['focus']} and l['derived_build_subreceipt']=={'path':str(BUILD),'sha256':SHA['build']},'launcher receipt parent/subreceipt binding differs')
 need(l['source_pins_before']==l['source_pins_after']==before and l['source_pin_count_before']==l['source_pin_count_after']==4729,'launcher source closure differs')
 need(not (ROOT/'target').exists(),'workspace target exists at acceptance time')
 # Rebuild the current source closure and check each frozen path against its live bytes.
 closure=import_module(SOURCE_CLOSURE,'fixed_baseline_source_closure')
 current=closure.pin_sources(closure.source_paths())
 need(current==before and len(current)==4729,'live source closure differs from focused build pins')
 for row in e['source_tool_pins_before']:
  need(pin(Path(row['path']))==row,'live source/tool input changed: '+row['path'])
 need(e['resource_guard']=={'minimum_free_bytes':5*1024**3,'maximum_target_bytes':1024**3,'poll_seconds':1,'peak_target_sampled':True},'capture guard configuration differs')
 need(e['actual_command_count']==len(e['commands'])==l['actual_command_count']==29 and c['actual_command_count']==29,'actual command count differs from 29')
 need(c['raw_stream_count']==l['raw_stream_count']==58 and l['raw_stream_error'] is None and l['command_count_ok'],'raw stream/launcher count checks failed')
 need([x['index'] for x in e['commands']]==list(range(1,30)),'command indices are not contiguous')
 need(sorted(p.name for p in (CAP/'raw').iterdir())==sorted(f'{i}.{s}.raw' for i in range(1,30) for s in ('stdout','stderr')),'raw file inventory differs')
 by={}
 for row in e['commands']:
  need(row['state']=='observed' and row['guard_stop'] is None and row['exit_code']==0,'command incomplete/nonzero: '+row['label'])
  need(row['peak_target_bytes']<=1024**3 and row['free_bytes_after']>=5*1024**3,'command guard evidence differs: '+row['label'])
  by[row['label']]=row
  for stream,ref in row['streams'].items():
   raw=Path(ref['path']).read_bytes(); need(Path(ref['path'])==CAP/'raw'/f"{row['index']}.{stream}.raw",'raw path differs')
   need(len(raw)==ref['bytes'] and digest(raw)==ref['sha256'],'raw stream hash differs')
 # Exact original class set is target plus Runner; no anonymous/generated classes.
 need(e['classes']==['cf11.UpstreamNestedLoops'] and e['runner_class']=='cf11.Runner','target/runner inventory differs')
 need(e['input_entries']['package']=='cf11' and e['input_entries']['source_level']=='Java 8' and len(e['input_entries']['target_classes'])==1,'input manifest shape differs')
 need(e['input_entries']['target_classes'][0]['class']=='cf11.UpstreamNestedLoops' and e['input_entries']['runner']['class']=='cf11.Runner','input class names differ')
 frozen=load(INPUT_PINS); need(digest(INPUT_PINS.read_bytes())==SHA['input_pins'],'input pin file differs')
 manifest=load(INPUTS/'manifest.json'); need(e['inputs_manifest']==pin(INPUTS/'manifest.json') and e['input_entries']==manifest,'capture manifest content/pin differs')
 for row in frozen['input_files']+frozen['provenance_files']+[frozen['manifest_file']]:
  p=Path(row['path']); need(p.is_file() and p.stat().st_size==row['bytes'] and digest(p.read_bytes())==row['sha256'] and p.stat().st_mode&0o777==0o444,'frozen input file differs: '+str(p))
 need(e['cli']==pin(CLI) and e['cli_mode']=='0o555' and e['build_receipt']==pin(BUILD),'execution CLI/build receipt pin differs')
 need(e['build_receipt_cli_binding']==cli_obj,'execution build receipt CLI binding differs')
 need(e['resource_guard']==c.get('resource_guard',e['resource_guard']),'checker guard mismatch')
 oracle8=Path(by['javac8-original-run']['streams']['stdout']['path']).read_bytes(); oracle23=Path(by['javac23-original-run']['streams']['stdout']['path']).read_bytes()
 need(oracle8 and oracle8==oracle23 and Path(by['javac8-original-run']['streams']['stderr']['path']).read_bytes()==b'' and Path(by['javac23-original-run']['streams']['stderr']['path']).read_bytes()==b'','original 8-case runtime oracle differs')
 lines=oracle8.decode('utf-8').splitlines(); need(len(lines)==8,'original Runner oracle is not 8 lines')
 need(c['original_oracle']['exit_code']==0 and c['original_oracle']['stdout_nonempty'] and c['original_oracle']['stderr_empty'] and c['original_oracle']['jdk8_jdk23_equal'],'checker original oracle summary differs')
 gen=c['generated_compile_observations']; need(len(gen)==8 and all(x['compile_exit']==0 and x['runtime_exit']==0 and x['class_set_exact'] and x['same_jdk_original'] for x in gen),'one of 8 generated runtime legs did not match its oracle')
 runtime_rows=[by[x['run_command']] for x in e['original_compile_rows']+e['jadx_compile_rows']+e['jarde_compile_rows'] if x.get('run_command')]
 need(len(runtime_rows)==10,'expected 2 original plus 8 generated runtime commands')
 for row in runtime_rows:
  out=Path(row['streams']['stdout']['path']).read_bytes(); err=Path(row['streams']['stderr']['path']).read_bytes()
  if row['label'].endswith('original-run') or row['label'].endswith('-run'):
   need(row['exit_code']==0 and out==oracle8 and err==b'','runtime bytes differ from original oracle: '+row['label'])
 need(c['behavior_mismatches']==[] and c['behavioral_acceptance']=='not_claimed','behavior mismatch or acceptance claim differs')
 need(c['source_map_gaps']==[] and c['all_method_physical_coverage_complete'] is True and c['jarde_method_maps_checked']==8,'physical BCI map completeness differs')
 # Verify each Jarde profile map against its original class bytes, javap, and physical instruction boundaries.
 checker=import_module(CHECKER,'fixed_baseline_checker_helpers'); parser=checker.import_parser(e); physical=[]
 jdk_manifest=load(Path(e['collector_config']['jdk_manifest'])); legs={}
 for entry in jdk_manifest['legs']:
  if entry['leg'] in ('javac8','javac23'):
   tools={k:Path(v['path']).resolve() for k,v in entry['jdk_tools'].items()}
   legs[entry['leg']]={**tools,'home':tools['java'].parent.parent}
 need(set(legs)=={'javac8','javac23'},'JDK toolchain legs differ')
 for row in e['commands']:
  need(Path(row['cwd']).resolve()==ROOT,'command working directory differs: '+row['label'])
  need(row['argv']==checker.expected_argv(CAP,e,row,legs),'actual argv differs: '+row['label'])
  jdk='javac23' if row['label'].startswith('jadx-') else row['label'].split('-')[0]
  need(row['java_home']==str(legs[jdk]['home']),'JAVA_HOME differs: '+row['label'])
 for profile in e['jarde_profiles']:
  need(pin(Path(profile['source_pin']['path']))==profile['source_pin'] and pin(Path(profile['report_pin']['path']))==profile['report_pin'],'Jarde source/report artifact pin differs')
  text=Path(profile['source_pin']['path']).read_text(encoding='utf-8')
  need(text.count('for (')==1 and text.count('while (')==1 and text.count('ordinary_generic_source_unproved')==1,'Jarde foreach/while/generic refusal profile shape differs')
  jdk=profile['jdk']; orig=next(x for x in e['original_compile_rows'] if x['label']==jdk+'-original')
  classrow=next(x for x in orig['classes'] if x['path'].endswith('/cf11/UpstreamNestedLoops.class'))
  javap=next(x for x in e['javap'] if x['jdk']==jdk)
  doc=load(Path(profile['report_pin']['path']))
  rows=checker.verify_class(parser,doc,Path(classrow['path']),Path(javap['stdout_pin']['path']).read_bytes(),'cf11/UpstreamNestedLoops')
  need(len(rows)==2 and rows[0]['name']=='<init>' and rows[1]['name']=='test','physical map method inventory differs')
  need(len(rows[0]['instruction_bcis'])==3 and len(rows[1]['instruction_bcis'])==45 and all(r['physical_coverage_complete'] for r in rows),'45 test BCI map or full physical coverage differs')
  physical.append({'label':profile['label'],'methods':[{'name':r['name'],'descriptor':r['descriptor'],'instruction_bci_count':len(r['instruction_bcis']),'physical_coverage_complete':r['physical_coverage_complete']} for r in rows]})
 need(len(physical)==4,'Jarde map profiles not 4')
 need(len(c['default_all'])==2 and all(x['both_rendered'] and x['source_and_maps_equal'] for x in c['default_all']),'default/all maps parity differs')
 summary={'schema':'hoisted-iterator-fixed-current-independent-acceptance-v1','status':'accepted_fixed_capture_only','execution_sha256':SHA['execution'],'checker_sha256':SHA['checker'],'launcher_receipt_sha256':SHA['launch'],'focused_parent_sha256':SHA['focus'],'derived_build_subreceipt_sha256':SHA['build'],'focused_parent_status':f['status'],'commands':29,'raw_streams':58,'source_closure_files':4729,'target_absent':True,'original_runtime_legs':2,'generated_runtime_legs':8,'original_runner_cases':lines,'jarde_profiles':physical,'jarde_method_maps_checked':8,'test_method_physical_bcis_per_profile':45,'source_map_gaps':[],'behavior_mismatches':[],'behavioral_acceptance':'not_claimed','scope':'Fixed TestNestedLoops capture only; this does not establish whole-workspace or general foreach acceptance.'}
 OUT.write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
 print(json.dumps(summary,ensure_ascii=False,indent=2))
if __name__=='__main__':
 try: main()
 except Exception as exc:
  print(f'fixed capture acceptance failed: {exc}',file=sys.stderr); raise
