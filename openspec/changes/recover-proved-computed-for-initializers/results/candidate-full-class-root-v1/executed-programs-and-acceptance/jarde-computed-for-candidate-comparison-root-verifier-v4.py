#!/usr/bin/env python3
"""Independent read-only verifier for the guarded candidate controls/direct comparison."""
from __future__ import annotations
import argparse, hashlib, json, os, re, stat
from pathlib import Path
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
BASE=Path('/private/tmp/jarde-computed-for-controls-jadx-root-v2/execution.json')
CLI_PATH='/private/tmp/jarde-computed-for-candidate-cli-root-v1'
DIRECT_ROWS=['hit:2:1','bounded-hit:4:1','miss:-1:1','empty-range:-1:1']
JDK_HOME={'javac8':'/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home','javac23':'/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home'}
PKG=re.compile(rb'(?m)^[ \t]*package\s+([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)\s*;[ \t]*$')
INCLUDE=re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')
def product_paths():
 paths={ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'.github/workflows/ci.yml',ROOT/'tests/fixtures/corpus-fingerprint.json'}
 for base in ('src','crates','tests'): paths.update((ROOT/base).rglob('*.rs'))
 paths.update((ROOT/'crates').glob('*/Cargo.toml'))
 for base in (ROOT/'tests/fixtures',ROOT/'openspec/changes/recover-class-scope-constructor-parameters/evidence/frozen'):
  paths.update(p for p in base.rglob('*') if p.is_file())
 for source in list(paths):
  if source.suffix=='.rs':
   for rel in INCLUDE.findall(source.read_text(encoding='utf-8')):
    child=(source.parent/rel).resolve()
    need(child.is_relative_to(ROOT),'literal include escapes workspace: '+str(source))
    if child.is_file(): paths.add(child)
 return {p.relative_to(ROOT).as_posix() for p in paths}
def sha(b): return hashlib.sha256(b).hexdigest()
def need(ok,msg):
 if not ok: raise ValueError(msg)
def pin(path, fact=None, label='file'):
 p=Path(path); need(p.is_file(),f'{label}: missing {p}'); b=p.read_bytes(); x={'path':str(p),'bytes':len(b),'sha256':sha(b)}
 if fact is not None: need(('bytes' not in fact or x['bytes']==fact['bytes']) and x['sha256']==fact.get('sha256'),f'{label}: pin mismatch {p}')
 return x
def load(p): return json.loads(Path(p).read_text())
def streams(exe,cmd):
 for k in ('stdout','stderr'):
  r=cmd['streams'][k]; pin(r['path'],r,f'{cmd["label"]} {k}')
def old_raw(record):
 p=Path(record['path'])
 if not p.is_absolute(): p=BASE.parent/p
 return pin(p,record,'old execution raw')
def inventory(folder):
 base=Path(folder)
 return [{'path':str(p),'relative':p.relative_to(base).as_posix(),'bytes':p.stat().st_size,'sha256':sha(p.read_bytes())} for p in sorted(base.rglob('*.class'))]
def same_inventory(recorded, folder, label):
 actual=inventory(folder)
 need(recorded==actual,label+' generated class inventory differs from disk')
 return actual
def javac_prefix(jdk):
 home=JDK_HOME[jdk]; base=[home+'/bin/javac']
 return base+(['-source','8','-target','8'] if jdk=='javac8' else ['--release','8'])+['-g:none','-proc:none']
def local_classpath(command, expected, label):
 argv=command['argv']; need('-cp' in argv,label+' missing classpath'); cp=argv[argv.index('-cp')+1]
 need(os.pathsep not in cp and Path(cp).resolve()==Path(expected).resolve(),label+' classpath is not its isolated captured directory')
 need(not Path(cp).resolve().is_relative_to(ROOT/'target'),label+' borrowed a repository target class')
 return Path(cp)
def main():
 ap=argparse.ArgumentParser(); ap.add_argument('--execution',required=True,type=Path); ap.add_argument('--build',required=True,type=Path); ap.add_argument('--out',required=True,type=Path); a=ap.parse_args()
 d=load(a.execution); b=load(a.build); old=load(BASE); need(d['status']=='completed_capture','candidate capture incomplete')
 need(b['schema']=='jarde-final-cli-build-clean-root-v1' and b['status']=='validation-passed-cli-frozen-target-cleaned','build record not frozen and clean')
 need(b['guard_limits']=={'free_bytes':5*1024**3,'target_bytes':1024**3},'build guard is not the required 5 GiB/1 GiB guard')
 need(d['guard']==b['guard_template'] and d['scanner']==b['target_size_scan_adapter'],'replay did not reuse the exact frozen build guard/scanner')
 need(b.get('pre_build_target_absent') is True and b.get('target_absent_after_clean') is True,'build record lacks target-absent-before/after proof')
 need(b.get('source_paths_before')==b.get('source_paths_after'),'build product paths changed around freeze')
 f=b['freeze']['frozen_cli']; cli=Path(f['path']); need(str(cli)==CLI_PATH and pin(cli)['sha256']==f['sha256'] and cli.stat().st_size==f['size_bytes'],'candidate CLI differs from build freeze')
 need(oct(stat.S_IMODE(cli.stat().st_mode))==f['mode']=='0o555','candidate CLI mode is not 0555')
 # Bind the exact source closure captured by the build, including pre/post equality and live files.
 source_map=b['freeze']['product_source_pins']; before=b['source_pins_before']['full_product_source_and_include_closure']; after=b['source_pins_after']['full_product_source_and_include_closure']
 need(source_map==before==after,'build source closure changed around build')
 need(set(source_map)==set(b['source_paths_before'])==set(b['source_paths_after']),'build paths differ from frozen source closure')
 need(set(source_map)==product_paths(),'build pin set is not the complete current Cargo/CI/corpus/include closure')
 for rel,digest in source_map.items(): need(pin(ROOT/rel)['sha256']==digest,'build source pin differs live: '+rel)
 need(d['build_record']['sha256']==sha(a.build.read_bytes()) and d['freeze']==f,'candidate capture not bound to supplied build record')
 for bc in b['commands']:
  need(bc['exit_code']==0 and bc['guard_stop'] is None,'CLI build/clean command failed or hit guard')
  for sr in bc['streams'].values(): pin(sr['path'],sr,'build command raw stream')
 need(d['base_jadx_execution']['sha256']==sha(BASE.read_bytes()),'fixed JADX baseline execution changed')
 for p,fact in old['inputs'].items(): pin(p,fact,'fixed baseline input')
 for p,fact in old['inputs'].items(): need(d['inputs'].get(p)==pin(p,fact,'candidate replay input'),'replay did not bind baseline input '+p)
 need(pin(d['runner']['path'],d['runner'],'candidate runner'),'runner pin invalid')
 for ref in (d['guard'],d['scanner']): pin(ref['path'],{'sha256':ref['sha256']},'guard/scanner')
 need(d['guard_env_overrides']==b['environment_overrides'],'guard environment overrides differ from build runner')
 commands=d['commands']; by={c['label']:c for c in commands}; need(len(by)==len(commands),'duplicate command labels')
 need(d['command_count']==len(commands),'command count metadata mismatch')
 for c in commands:
  need(c['cwd']==str(ROOT),'command cwd changed'); need(c['guard_stop'] is None,'command hit resource guard'); streams(d,c)
  need(c.get('jdk') in JDK_HOME,'command is not tied to a pinned JDK')
  need(c['env_overrides']==b['environment_overrides'],'guard environment overrides mismatch')
  need(c['removed_env_keys']==d['removed_java_env_keys'],'removed Java environment metadata mismatch')
 need(set(d['jdk_pins'])=={f'{j}.{t}' for j in JDK_HOME for t in ('java','javac')},'JDK pin set incomplete')
 for j,home in JDK_HOME.items():
  for t in ('java','javac'): pin(d['jdk_pins'][j+'.'+t]['path'],d['jdk_pins'][j+'.'+t],j+' '+t)
 cases={c['id']:c for c in old['cases']}; need(set(cases)=={c['id'] for c in d['cases']},'case matrix differs from frozen six inputs')
 for key,c in cases.items():
  for field in ('class_input','source_input','runner_source'):
   pin(c[field]['path'],c[field],key+' '+field)
  src=c['jadx_generated_sources'][0]; pin(src['path'],src,key+' complete JADX source')
 labels=set()
 for family in ('controls','direct-return'):
  for j in JDK_HOME: labels|={f'oracle-compile-{family}-{j}',f'oracle-runtime-{family}-{j}'}
 for key,c in cases.items():
  labels.add('raw-runtime-'+key)
  for j in JDK_HOME: labels|={f'jadx-compile-{key}-{j}',f'jadx-runtime-{key}-{j}'}
  for e in ('default','all'):
   labels|={f'jarde-render-{key}-{e}',f'jarde-compile-{key}-{e}'}
   if c['fixture']=='direct-return': labels.add(f'jarde-runtime-{key}-{e}')
 need(set(by)==labels,f'command matrix mismatch missing={sorted(labels-set(by))}, extra={sorted(set(by)-labels)}')
 def oracle(fam,j):
  c=next(c for c in d['oracle_results'] if c['family']==fam and c['jdk']==j)
  need(c['compile_exit']==0 and c['runtime_exit']==0,'fresh original oracle did not compile/run')
  return pin(c['stdout']['path'],c['stdout'],fam+' '+j+' fresh oracle')
 for family in ('controls','direct-return'):
  raw=[oracle(family,j).get('sha256') for j in JDK_HOME]; need(len(set(raw))==1,f'{family} fresh original outputs differ across JDKs')
  first=Path(oracle(family,'javac8')['path']).read_bytes()
  if family=='controls':
   need(first.count(b'\n')==9 and first.endswith(b'\n'),'fresh controls oracle is not nine complete rows')
   for fact in old['oracle_crosschecks']['controls_existing_runtime_raws']:
    need(Path(pin(fact['path'],fact,'historical controls oracle')['path']).read_bytes()==first,'fresh controls oracle differs from historical original/pending runtime')
  else:
   need(first.decode('utf-8').splitlines()==DIRECT_ROWS and first.endswith(b'\n'),'direct-return oracle is not the exact four-row output')
   for fact in old['oracle_crosschecks']['direct_return_existing_runtime_raws']:
    need(Path(pin(fact['path'],fact,'historical direct oracle')['path']).read_bytes()==first,'fresh direct oracle differs from historical JDK output')
 for r in d['oracle_results']:
  fam,j=r['family'],r['jdk']; c=next(x for x in old['cases'] if x['fixture']==fam); folder=Path(r['classes_dir'])
  inv=same_inventory(r['generated_classes'],folder,'oracle '+fam+'/'+j)
  names={x['relative'] for x in inv}; need(names=={c['class_name']+'.class',Path(c['runner_source_path']).stem+'.class'},'oracle compile did not emit exactly class and Runner')
  cc=by[f'oracle-compile-{fam}-{j}']; rr=by[f'oracle-runtime-{fam}-{j}']; local_classpath(rr,folder,'oracle runtime')
  need(rr['argv']==[JDK_HOME[j]+'/bin/java','-Xverify:all','-cp',str(folder),c['main_class']],'oracle runtime argv mismatch')
  need(cc['argv'][-2:]==[c['source_input_path'],c['runner_source_path']],'oracle compile does not use fixed source and Runner')
  need(cc['argv']==javac_prefix(j)+['-d',str(folder),c['source_input_path'],c['runner_source_path']],'oracle compile argv/output binding mismatch')
  need(rr['argv'][-1]==c['main_class'] and '-Xverify:all' in rr['argv'],'oracle runtime argv mismatch')
 # All raw class runs use exactly the fresh source oracle and the true JVM verifier.
 for r in d['raw_results']:
  c=cases[r['case']]; cmd=by['raw-runtime-'+r['case']]; need(cmd['exit_code']==0 and '-Xverify:all' in cmd['argv'],'raw runtime failed verification')
  expected=Path(oracle(c['fixture'],c['source_jdk'])['path']).read_bytes(); actual=Path(cmd['streams']['stdout']['path']).read_bytes()
  folder=Path(r['classes_dir']); inv=same_inventory(r['generated_classes'],folder,'raw '+r['case'])
  names={x['relative'] for x in inv}; need(names=={c['class_name']+'.class',Path(c['runner_source_path']).stem+'.class'},'raw classpath has unaccounted/missing classes')
  need(pin(folder/(c['class_name']+'.class'),c['class_input'],'raw fixture class') is not None,'raw fixture input mismatch')
  oracle_row=next(x for x in d['oracle_results'] if x['family']==c['fixture'] and x['jdk']==c['source_jdk'])
  need((folder/Path(c['runner_source_path']).name.replace('.java','.class')).read_bytes()==(Path(oracle_row['classes_dir'])/Path(c['runner_source_path']).name.replace('.java','.class')).read_bytes(),'raw runner is not the matching freshly compiled Runner')
  local_classpath(cmd,folder,'raw runtime'); need(cmd['argv']==[JDK_HOME[c['source_jdk']]+'/bin/java','-Xverify:all','-cp',str(folder),c['main_class']],'raw runtime argv mismatch')
  need(actual==expected and r['matches_fresh_oracle'],'raw class differs from fresh original source oracle')
 # Fixed complete JADX source replay: bytes are the old frozen source; only Runner receives package prefix.
 old_jadx={(x['case'],x['jdk']):x for x in old['jadx_results']}; new_jadx={(x['case'],x['jdk']):x for x in d['results'] if x['kind']=='jadx'}
 need(len(new_jadx)==12,'complete JADX compile/runtime replay matrix incomplete')
 for key,c in cases.items():
  src=pin(c['jadx_source_path'],c['jadx_generated_sources'][0],key+' full JADX source'); need(PKG.findall(Path(src['path']).read_bytes())==[b'defpackage'],'JADX package changed')
  for j in JDK_HOME:
   r=new_jadx[(key,j)]; oldr=old_jadx[(key,j)]; adapter=r['adapter']; before=Path(c['runner_source_path']).read_bytes(); after=Path(adapter['runner_after']['path']).read_bytes(); prefix=b'package defpackage;\n'
   need(adapter['runner_before']['sha256']==sha(before) and after==prefix+before and adapter['runner_body_equal'],'JADX Runner adaptation changed body')
   need(adapter['source_before']['bytes']==adapter['source_after']['bytes']==src['bytes'] and adapter['source_before']['sha256']==adapter['source_after']['sha256']==src['sha256'],'complete JADX source changed during staging')
   cc=by[f'jadx-compile-{key}-{j}']; rr=by[f'jadx-runtime-{key}-{j}']
   need(cc['exit_code']==oldr['compile_exit'] and rr['exit_code']==oldr.get('runtime_exit'),'JADX compile/runtime outcome differs frozen replay')
   jdir=Path(r['classes']); inv=same_inventory(r['generated_classes'],jdir,'JADX '+key+'/'+j)
   need({x['relative'] for x in inv}=={'defpackage/'+c['class_name']+'.class','defpackage/'+c['main_class']+'.class'},'JADX full compile class inventory mismatch')
   ccargv=cc['argv']; need(ccargv==javac_prefix(j)+['-d',str(jdir),adapter['source_after']['path'],adapter['runner_after']['path']],'JADX compile argv/sources are not the unchanged full source plus adapted Runner')
   local_classpath(rr,jdir,'JADX runtime')
   need(rr['argv']==[JDK_HOME[j]+'/bin/java','-Xverify:all','-cp',str(jdir),'defpackage.'+c['main_class']],'JADX runtime argv mismatch')
   # Match exact old and newly observed full output, including original controls' known mismatches.
   oldcmd=next(x for x in old['commands'] if x['label']==f'jadx-runtime-{key}-{j}')
   newbytes=Path(rr['streams']['stdout']['path']).read_bytes(); oldbytes=Path(old_raw(oldcmd['stdout'])['path']).read_bytes()
   need(newbytes==oldbytes,'fixed JADX full-class runtime output changed from old observed source')
   need(r['runtime_matches_fresh_oracle']==(newbytes==Path(oracle(c['fixture'],j)['path']).read_bytes()),'JADX oracle comparison metadata mismatch')
 # Candidate Jarde: control refusal/source exactly preserved; direct full class compiles/runs and uses a for header.
 oldjr={(x['case'],x['evidence']):x for x in old['jarde_results']}; newjr={(x['case'],x['evidence']):x for x in d['results'] if x['kind']=='jarde'}
 need(len(newjr)==12,'candidate Jarde default/all matrix incomplete')
 for key,c in cases.items():
  for ev in ('default','all'):
   r=newjr[(key,ev)]; oldr=oldjr[(key,ev)]; need(r['render_exit']==0,'candidate render failed')
   render=by[f'jarde-render-{key}-{ev}']; want=[str(cli),'class-source','--input',c['class_input']['path'],'--class',c['class_name'],'--policy','single-class','--release','8','--format','json']
   if ev=='all': want+=['--evidence','all']
   need(render['argv']==want and render['exit_code']==0,'candidate render argv/exit mismatch')
   src=pin(r['source']['path'],r['source'],key+' candidate source'); doc=r['report']; need(Path(src['path']).read_bytes()==doc['text'].encode(),'candidate source differs JSON report')
   cli_doc=json.loads(Path(render['streams']['stdout']['path']).read_bytes())
   need(doc==cli_doc,'recorded full Jarde report differs from captured CLI JSON stdout')
   if c['fixture']=='controls':
    need(doc['text']==Path(oldr['source']['path']).read_text(),'candidate control text differs prior frozen output')
    prior=oldr['quality_summary']['methods']
    def current_method_facts(rows):
     out=[]
     for m in rows:
      rep=m.get('outcome',{}).get('report',{})
      name=bytes(m['item']['identity']['name']).decode('utf-8')
      out.append({'name':name,'quality':rep.get('quality'),'content':rep.get('content'),'source_map':rep.get('source_map')})
     return out
    def prior_method_facts(rows):
     out=[]
     for m in rows:
      rep=m.get('outcome',{}).get('report',{})
      out.append({'name':m['name'],'quality':rep.get('quality'),'content':rep.get('content'),'source_map':rep.get('source_map')})
     return out
    need(current_method_facts(doc.get('methods',[]))==prior_method_facts(prior),'control refusal/materialization quality, content, or source maps changed')
   cc=by[f'jarde-compile-{key}-{ev}']; need(cc['exit_code']==r['compile_exit'],'Jarde compile metadata mismatch')
   outdir=Path(cc['argv'][cc['argv'].index('-d')+1]); inv=same_inventory(r['generated_classes'],outdir,'Jarde '+key+'/'+ev)
   need(cc['argv']==javac_prefix(c['source_jdk'])+['-d',str(outdir),r['source']['path'],str(Path(r['source']['path']).parent/Path(c['runner_source_path']).name)],'Jarde compile argv/sources do not bind captured full source and fixed Runner')
   need(not any(Path(x['path']).resolve().is_relative_to(ROOT/'target') for x in inv),'Jarde classes unexpectedly came from target')
   if c['fixture']=='controls':
    need(cc['exit_code']==oldr['compile_exit']==1 and 'jarde-runtime-'+key+'-'+ev not in by,'control full-source refusal changed or was run as runtime')
    need(inv==[],'failed control compile emitted class files')
    need(r.get('runtime_exit') is None and r.get('compile_only') is True,'control compile failure mislabeled as runtime')
   else:
    need(cc['exit_code']==0,'direct-return whole source failed compilation')
    need({x['relative'] for x in inv}>={'ComputedForDirectReturn.class','ComputedForDirectReturnRunner.class'},'direct-return compile lacks its class or Runner')
    run=by[f'jarde-runtime-{key}-{ev}']; need(run['exit_code']==0 and '-Xverify:all' in run['argv'],'direct-return full-source runtime failed')
    local_classpath(run,outdir,'direct-return Jarde runtime'); need(run['argv']==[JDK_HOME[c['source_jdk']]+'/bin/java','-Xverify:all','-cp',str(outdir),c['main_class']],'direct-return Jarde runtime argv mismatch')
    expected=Path(oracle('direct-return',c['source_jdk'])['path']).read_bytes()
    direct_output=Path(run['streams']['stdout']['path']).read_bytes()
    need(direct_output.decode('utf-8').splitlines()==DIRECT_ROWS and direct_output.endswith(b'\n'),'direct-return runtime is not the exact four-row output')
    need(direct_output==expected and r.get('runtime_matches_fresh_oracle') is True,'direct-return output differs fresh original oracle')
    find=next(m for m in doc['methods'] if bytes(m['item']['identity']['name']).decode()=='find')['outcome']['report']
    txt=find['text']; headers=list(re.finditer(r'\bfor\s*\(',txt)); need(len(headers)==1 and not re.search(r'\bwhile\s*\(',txt),'direct find is not a single for-loop')
    init=re.search(r'for\s*\(\s*(?:int\s+)?([A-Za-z_$][\w$]*)\s*=\s*limit\(\)\s*-\s*1\s*;',txt)
    need(init is not None and txt.count('limit()')==1,'for initializer does not materialize limit() exactly once')
    end=txt.find(';',init.end()); need(end>=0,'for condition separator absent')
    need(re.fullmatch(r'\s*'+re.escape(init.group(1))+r'\s*>=\s*arg2\s*',txt[init.end():end]) is not None,'for condition does not retain lower bound')
   # Default/all full-class text and method source maps remain equal.
  da=newjr[(key,'default')]['report']; al=newjr[(key,'all')]['report']; need(da['text']==al['text'],'default/all full source differs '+key)
  need(len(da['methods'])==len(al['methods']),'default/all method counts differ '+key)
  for dm,am in zip(da['methods'],al['methods']):
   need(dm['text']==am['text'],'default/all method source differs '+key)
   dr=dm.get('outcome',{}).get('report'); ar=am.get('outcome',{}).get('report')
   if dr is not None or ar is not None: need(dr is not None and ar is not None and dr.get('source_map')==ar.get('source_map'),'default/all source maps differ '+key)
 for path,fact in d['inputs'].items(): pin(path,fact,'final replay input recheck')
 pin(cli,{'bytes':f['size_bytes'],'sha256':f['sha256']},'final frozen CLI recheck')
 need(oct(stat.S_IMODE(cli.stat().st_mode))=='0o555','final frozen CLI mode changed')
 summary={'status':'accepted_candidate_replay','scope':'candidate runtime/source comparison only; not CI acceptance','commands':len(commands),'fresh_oracle_families':2,'raw_class_runs':len(d['raw_results']),'fixed_jadx_source_replays':len(new_jadx),'candidate_jarde_renders':len(newjr),'controls_compile_refusals_preserved':8,'direct_full_class_verified_runs':4,'source_closure_count':len(source_map)}
 a.out.parent.mkdir(parents=True,exist_ok=True); a.out.write_text(json.dumps(summary,indent=2,sort_keys=True)+'\n'); print(json.dumps(summary,indent=2))
if __name__=='__main__':
 ap=argparse.ArgumentParser(); ap.add_argument('--execution',required=True,type=Path); ap.add_argument('--build',required=True,type=Path); ap.add_argument('--out',required=True,type=Path); a=ap.parse_args();
 try: main()
 except Exception as e:
  obj={'status':'failed','error':type(e).__name__+': '+str(e)}; a.out.parent.mkdir(parents=True,exist_ok=True); a.out.write_text(json.dumps(obj,indent=2)+'\n'); print(obj['error'],file=__import__('sys').stderr); raise
