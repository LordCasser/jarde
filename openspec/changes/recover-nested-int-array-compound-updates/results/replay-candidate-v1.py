#!/usr/bin/env python3
"""Fresh four-leg candidate replay against frozen P02 and NestedIntUpdates inputs."""
from __future__ import annotations
import hashlib, json, os, shutil, subprocess, sys
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS=ROOT/'openspec/changes/recover-nested-int-array-compound-updates/results'
OUT=RESULTS/'candidate-v1'
P02_ROOT=ROOT/'openspec/changes/recover-bigdecimal-number-widening/results/nested-array-update-baseline-v1'
P02_MANIFEST=P02_ROOT/'manifest.json'
NESTED_BASELINE=RESULTS/'controls-baseline-v1/manifest.json'
CONTROLS=RESULTS/'controls-v1/manifest.json'
REMOVED=('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH')
PRODUCTS={'crates/jarde-java/src/init.rs','crates/jarde-java/src/report.rs','crates/jarde-java/src/build.rs','src/class_source.rs','Cargo.lock'}
TESTS={'tests/p3_nested_int_array_compound_updates.rs','.github/workflows/ci.yml','tests/recover_lambda_primitive_array_capture.rs','tests/p3_constructed_reference_array_elements.rs','tests/recover_boxed_number_widening.rs'}

def sha(b): return hashlib.sha256(b).hexdigest()
def filehash(p): return sha(Path(p).read_bytes())
def record(p,base=ROOT):
 p=Path(p); return {'path':str(p.relative_to(base)),'bytes':p.stat().st_size,'sha256':filehash(p)}
def capture(path,data):
 path.parent.mkdir(parents=True,exist_ok=True); path.write_bytes(data)
 return {'path':str(path.relative_to(OUT)),'bytes':len(data),'sha256':sha(data)}
def env_for(home=None):
 env={k:v for k,v in os.environ.items() if k not in REMOVED}
 if home: env['JAVA_HOME']=home; env['PATH']=str(Path(home)/'bin')+os.pathsep+env.get('PATH','')
 return env
def run(label,argv,cwd,home,commands):
 p=subprocess.run([str(x) for x in argv],cwd=cwd,env=env_for(home),stdout=subprocess.PIPE,stderr=subprocess.PIPE,check=False)
 row={'label':label,'argv':[str(x) for x in argv],'cwd':str(cwd),'java_home':home,'environment_removed':list(REMOVED),'exit':p.returncode,
      'stdout':capture(OUT/'streams'/f'{label}.stdout',p.stdout),'stderr':capture(OUT/'streams'/f'{label}.stderr',p.stderr)}
 commands.append(row); return p,row
def members(doc):
 out=[]
 for m in doc.get('methods',[]):
  it=m.get('item',{}); oc=m.get('outcome',{}); rp=oc.get('report') if oc.get('kind')=='recovered' else None
  sm=rp.get('source_map') if rp else None; bcis=set()
  for seg in (sm or {}).get('segments',[]):
   org=seg.get('origin',{}); b=org.get('primary',{}).get('bci')
   if isinstance(b,int): bcis.add(b)
   bcis.update(x['bci'] for x in org.get('derived',[]) if isinstance(x.get('bci'),int))
  body=rp.get('text','') if rp else ''
  out.append({'name':it.get('name',{}).get('escaped'),'name_raw':it.get('name',{}).get('raw'),
   'descriptor':it.get('descriptor',{}).get('escaped'),'descriptor_raw':it.get('descriptor',{}).get('raw'),
   'access_flags':it.get('access_flags'),'declaration':m.get('declaration'),'outcome':oc.get('kind'),
   'refusal':oc.get('refusal'),'quality':rp.get('quality') if rp else None,
   'representation':rp.get('representation') if rp else None,'markers':m.get('markers',[]),'body':body,
   'body_sha256':sha(body.encode()) if rp else None,'source_map':sm,'source_map_all_bcis':sorted(bcis)})
 return out
def history_file(path,expected):
 p=Path(path); p=p if p.is_absolute() else ROOT/p; data=p.read_bytes()
 if sha(data)!=expected: raise SystemExit(f'historical input changed: {p}')
 return data
def check_inventory(root,rows):
 for row in rows:
  p=Path(row['path']); p=p if p.is_absolute() else root/p
  data=p.read_bytes()
  if len(data)!=row['bytes'] or sha(data)!=row['sha256']: raise SystemExit(f'historical inventory mismatch: {p}')
def main():
 if len(sys.argv)!=3: raise SystemExit('usage: replay-candidate-v1.py FROZEN_CLI CLI_METADATA_JSON')
 cli=Path(sys.argv[1]).resolve(); meta_path=Path(sys.argv[2]).resolve(); meta_raw=meta_path.read_bytes(); meta=json.loads(meta_raw)
 if not cli.is_file() or filehash(cli)!=meta['cli_sha256'] or Path(meta['cli_path']).resolve()!=cli: raise SystemExit('candidate CLI identity mismatch')
 if set(meta['candidate_sources'])!=PRODUCTS or set(meta.get('test_sources',{}))!=TESTS: raise SystemExit('candidate metadata source set changed')
 for rel,h in {**meta['candidate_sources'],**meta['test_sources']}.items():
  if filehash(ROOT/rel)!=h: raise SystemExit(f'frozen candidate source changed: {rel}')
 if OUT.exists(): raise SystemExit(f'refusing to overwrite {OUT}')
 p02_raw=P02_MANIFEST.read_bytes(); p02=json.loads(p02_raw)
 nested_raw=NESTED_BASELINE.read_bytes(); nested=json.loads(nested_raw)
 control_raw=CONTROLS.read_bytes(); control=json.loads(control_raw)
 if p02.get('schema')!='fresh-nested-array-update-baseline-v1' or nested.get('status')!='complete_baseline_recorded' or control.get('status')!='complete':
  raise SystemExit('frozen historical baseline set is incomplete')
 check_inventory(P02_ROOT,p02['files'])
 check_inventory(ROOT,control['fixture_files'])
 check_inventory(ROOT,nested['file_inventory'])
 if nested['controls_input']['manifest_sha256']!=sha(control_raw): raise SystemExit('nested baseline does not bind the frozen controls manifest')
 OUT.mkdir(); commands=[]; cases=[]; inputs=[]
 control_legs={x['leg']:x for x in control['legs']}; p02_cases={x['leg']:x for x in p02['cases']}
 nested_cases={x['leg']:x for x in nested['cases'] if x['kind']=='jarde'}
 for leg in ('javac8','javac23'):
  pc=p02_cases[leg]; nc=nested_cases[leg]
  p02_class=Path(pc['input']['path']); p02_bytes=history_file(p02_class,pc['input']['sha256'])
  nested_classrow=next(r for r in control_legs[leg]['class_files'] if Path(r['path']).name=='NestedIntUpdates.class')
  nested_class=ROOT/nested_classrow['path']; nested_bytes=history_file(nested_class,nested_classrow['sha256'])
  originals=[]
  p02run=next(c for c in p02['commands'] if c['leg']==leg and c['label']=='original-run' and c['argv'][0]==pc['tools']['java']['path'])
  p02stdout=history_file(p02run['stdout']['path'],p02run['stdout']['sha256']); p02stderr=history_file(p02run['stderr']['path'],p02run['stderr']['sha256'])
  if p02stdout!=b'6\n' or p02run['exit']!=0: raise SystemExit(f'{leg}: P02 historical oracle changed')
  cn=next(c for c in control_legs[leg]['commands'] if c['label']=='run-runner')
  nestedstdout=history_file(CONTROLS.parent/'controls-v1'/cn['stdout']['path'],cn['stdout']['sha256']); nestedstderr=history_file(CONTROLS.parent/'controls-v1'/cn['stderr']['path'],cn['stderr']['sha256'])
  if nestedstdout!=control['expected_runner_stdout']['utf8'].encode() or cn['exit']!=0: raise SystemExit(f'{leg}: Nested historical oracle changed')
  for dataset,source,source_row,orig_stdout,orig_stderr,orig_exit,oracle_row in [
   ('P02',p02_bytes,pc['input'],p02stdout,p02stderr,p02run['exit'],p02run),
   ('NestedIntUpdates',nested_bytes,nested_classrow,nestedstdout,nestedstderr,cn['exit'],cn)]:
   label=f'{dataset}-{leg}'; case=OUT/'cases'/label; case.mkdir(parents=True)
   inp=capture(case/'input.class',source); original={'exit':orig_exit,'stdout':capture(case/'original.stdout',orig_stdout),'stderr':capture(case/'original.stderr',orig_stderr),'fresh_execution':False}
   inputs.append({'case':label,'source_path':source_row.get('path'),'source_sha256':sha(source),'copied_input':inp,'original_raw_fresh':False})
   jdk=(pc['tools'] if dataset=='P02' else control_legs[leg]['jdk_tools'])
   for tool in ('java','javac','javap'):
    entry=jdk[tool]; executable=Path(entry['path']); data=executable.read_bytes()
    if len(data)!=entry['bytes'] or sha(data)!=entry['sha256']: raise SystemExit(f'{leg}: frozen {tool} identity changed')
    other=(control_legs[leg]['jdk_tools'][tool] if dataset=='P02' else pc['tools'][tool])
    if (entry['path'],entry['sha256'])!=(other['path'],other['sha256']): raise SystemExit(f'{leg}: P02/control {tool} identities differ')
   javac,javap,java=(jdk['javac']['path'],jdk['javap']['path'],jdk['java']['path'])
   oldcompile=next(c for c in (p02['commands'] if dataset=='P02' else control_legs[leg]['commands'])
                   if c['label'] in ('candidate-compile','compile') and (dataset!='P02' or c['leg']==leg))
   flags=oldcompile['argv'][1:oldcompile['argv'].index('-classpath')]
   empty=case/'empty-classpath-sourcepath'; empty.mkdir(); classes=case/'candidate-classes'; classes.mkdir()
   render_argv=[str(cli),'class-source','--input',str(case/'input.class'),'--class',
      'P02_multianewarray' if dataset=='P02' else 'NestedIntUpdates','--policy','single-class','--release','8','--format','json','--evidence','all']
   rendered,render_record=run(label+'-render',render_argv,ROOT,None,commands); doc=None; source=None; parse_error=None
   if rendered.returncode==0:
    try:
     doc=json.loads(rendered.stdout); source_text=doc['text']; source=case/('P02_multianewarray.java' if dataset=='P02' else 'NestedIntUpdates.java'); source.write_text(source_text)
    except (json.JSONDecodeError,KeyError,TypeError) as e: parse_error=str(e)
   report={'class':doc.get('class') if doc else None,'json_parsed':doc is not None,'source':record(source,OUT) if source else None,
           'source_text_sha256':sha(doc['text'].encode()) if doc else None,'members':members(doc) if doc else [],'parse_error':parse_error,
           'raw_json_stdout':render_record['stdout']}
   compile_record=runtime_record=None; generated=[]
   compile_sources=[]
   if source:
    compile_sources=[source]
    if dataset=='NestedIntUpdates':
     runner_src=Path(control['fixture_root'])/'Runner.java'; runner_copy=case/'Runner.java'; shutil.copyfile(runner_src,runner_copy)
     compile_sources.append(runner_copy)
    compile_argv=[javac,*flags,'-classpath',str(empty),'-sourcepath',str(empty),'-d',str(classes),*[str(x) for x in compile_sources]]
    _,compile_record=run(label+'-compile',compile_argv,ROOT,str(Path(java).parent.parent),commands)
    if compile_record['exit']==0:
     main='P02_multianewarray' if dataset=='P02' else 'Runner'
     _,runtime_record=run(label+'-run',[java,'-Xverify:all','-cp',str(classes),main],ROOT,str(Path(java).parent.parent),commands)
   if classes.exists(): generated=[record(p,classes) for p in sorted(classes.rglob('*.class'))]
   runtime_match=bool(runtime_record and runtime_record['exit']==orig_exit and
      (OUT/runtime_record['stdout']['path']).read_bytes()==orig_stdout and (OUT/runtime_record['stderr']['path']).read_bytes()==orig_stderr)
   success=bool(report['json_parsed'] and source and compile_record and compile_record['exit']==0 and runtime_record and runtime_match)
   cases.append({'label':label,'dataset':dataset,'leg':leg,'input':inp,'original':original,
      'original_stream_source':{'manifest':str(P02_MANIFEST if dataset=='P02' else CONTROLS),'command_label':oracle_row['label'],
       'stdout_path':oracle_row['stdout']['path'],'stdout_sha256':oracle_row['stdout']['sha256'],
       'stderr_path':oracle_row['stderr']['path'],'stderr_sha256':oracle_row['stderr']['sha256'],'fresh_execution':False},
      'report':report,'compile_sources':[record(x,OUT) for x in compile_sources],'compiler_flags':flags,
      'empty_classpath_sourcepath':str(empty),'candidate_classes':generated,'compile':compile_record,'runtime':runtime_record,
      'runtime_matches_original_exit_stdout_stderr':runtime_match,'candidate_success':success})
   if not success: pass
 baseline_refs=[]
 for path,raw in ((P02_MANIFEST,p02_raw),(NESTED_BASELINE,nested_raw),(CONTROLS,control_raw)):
  baseline_refs.append({'path':str(path),'sha256':sha(raw),'fresh_execution_claim':False})
 for p,h in [(P02_MANIFEST,p02_raw),(NESTED_BASELINE,nested_raw),(CONTROLS,control_raw)]:
  if sha(p.read_bytes())!=sha(h): raise SystemExit('historical baseline changed during replay')
 manifest={'schema':'nested-int-array-compound-candidate-v1','status':'complete_replay_recorded',
  'runner':record(Path(__file__).resolve()),'output_root':str(OUT),'candidate_cli':{'path':str(cli),'sha256':meta['cli_sha256'],
   'metadata_path':str(meta_path),'metadata_sha256':sha(meta_raw),'candidate_sources':meta['candidate_sources'],'test_sources':meta['test_sources']},
  'historical_baselines':baseline_refs,'input_classes':inputs,'commands':commands,'cases':cases,
  'expected_denominator':4,'environment_removed':list(REMOVED),'files':[]}
 manifest['files']=[record(p,OUT) for p in sorted(OUT.rglob('*')) if p.is_file() and p.name!='manifest.json']
 (OUT/'manifest.json').write_text(json.dumps(manifest,indent=2,ensure_ascii=False)+'\n')
 if len(cases)!=4 or not all(c['candidate_success'] for c in cases): raise SystemExit('candidate replay failed; raw evidence retained in manifest')
 print(json.dumps({'status':manifest['status'],'cases':len(cases),'successes':sum(c['candidate_success'] for c in cases),'manifest':str(OUT/'manifest.json')}))

if __name__=='__main__': main()
