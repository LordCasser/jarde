#!/usr/bin/env python3
"""Independent read-only checker for v3 capture; reports JADX outcomes separately."""
import hashlib,json,pathlib,re,stat,sys
E=pathlib.Path(sys.argv[1] if len(sys.argv)>1 else '/private/tmp/jarde-computed-shared-stack-compare-root-v3/execution.json')
OUT=E.parent;ROOT=pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
BUILD=pathlib.Path('/private/tmp/jarde-computed-for-cli-build-root-v1/execution.json')
CLI=pathlib.Path('/private/tmp/jarde-computed-for-candidate-cli-root-v1')
ENV_KEYS=('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH')
sha=lambda b:hashlib.sha256(b).hexdigest()
def require(ok,msg):
 if not ok:raise AssertionError(msg)
def pin(p):
 p=pathlib.Path(p);b=p.read_bytes();return {'path':str(p),'bytes':len(b),'sha256':sha(b)}
def raw(ref):
 p=pathlib.Path(ref['path']);b=p.read_bytes();require(len(b)==ref['bytes'] and sha(b)==ref['sha256'],f'raw stream pin mismatch: {p}');return b
def command_index(d):
 out={}
 for c in d['commands']:
  require(c['index'] not in out,f'duplicate command index {c["index"]}')
  require(c['label'] not in out,f'duplicate command label {c["label"]}')
  require(c['argv'] and c['cwd']==str(ROOT),f'invalid command/cwd {c.get("label")}')
  raw(c['streams']['stdout']);raw(c['streams']['stderr']);out[c['label']]=c
 return out
def inv(root,expected):
 root=pathlib.Path(root);actual=[]
 for p in sorted(root.rglob('*.class')):
  b=p.read_bytes();actual.append({'relative':str(p.relative_to(root)),'bytes':len(b),'sha256':sha(b)})
 require(actual==expected,f'class inventory differs from live directory {root}')
 return actual
def method_id(m):return (bytes(m['item']['identity']['name']),bytes(m['item']['identity']['descriptor']))
def javap_instructions(data):
 lines=data.decode('utf-8').splitlines();start=None
 for i,line in enumerate(lines):
  if re.search(r'\bfind\(int\[\], int, int\);\s*$',line.strip()):start=i;break
 require(start is not None,'javap lacks find(int[], int, int) header')
 result=[]
 for line in lines[start+1:]:
  if re.match(r'^\s*(?:public|protected|private|static|final|synchronized|native|abstract)\b.*\);\s*$',line) and 'Code:' not in line:break
  m=re.match(r'^\s*(\d+):\s+([a-z0-9_]+)(?:\s+(.*))?$',line)
  if m:result.append((int(m.group(1)),m.group(2),m.group(3) or ''))
  if result and line.strip() in ('LineNumberTable:','LocalVariableTable:','StackMapTable:','Exceptions:'):break
 require(result,'javap find instruction list empty');return result
def shared_pattern(ins):
 calls=[i for i,x in enumerate(ins) if 'limit:()I' in x[2]]
 dups=[i for i,x in enumerate(ins) if x[1]=='dup']
 puts=[i for i,x in enumerate(ins) if x[1]=='putstatic' and 'savedField:I' in x[2]]
 stores=[i for i,x in enumerate(ins) if x[1].startswith('istore')]
 require(len(calls)==len(dups)==len(puts)==len(stores)==1,'expected one call, dup, savedField putstatic, and local istore')
 call,dup,put,store=calls[0],dups[0],puts[0],stores[0]
 subtract=next((i for i,x in enumerate(ins) if i>call and x[1]=='isub'),None)
 require(subtract is not None and call<subtract<dup<put<store,'call/isub/dup/putstatic/istore sequence absent')
 return {'call_bci':ins[call][0],'dup_bci':ins[dup][0],'putstatic_bci':ins[put][0],'istore_bci':ins[store][0],'instruction_count':len(ins),'physical_bcis':sorted(b for b,_,_ in ins)}
def methods_text_maps(doc):
 out=[]
 for m in doc['methods']:
  report=m.get('outcome',{}).get('report') or {}
  out.append({'identity':m['item']['identity'],'method_text':m.get('text'),'body_text':report.get('text'),'source_map':report.get('source_map')})
 return out
def check_exact_find_map(doc,physical):
 digest=doc.get('class',{}).get('class_bytes',{}).get('digest');require(isinstance(digest,str),'class byte identity missing')
 hits=[m for m in doc.get('methods',[]) if method_id(m)==(b'find',b'([III)I')]
 require(len(hits)==1,'find([III)I method is not unique');m=hits[0];r=m.get('outcome',{}).get('report');require(isinstance(r,dict),'find has no recovered report')
 text=r.get('text');require(isinstance(text,str),'find source text missing');utf=text.encode('utf-8')
 segs=r.get('source_map',{}).get('segments');require(isinstance(segs,list),'find source-map segments missing')
 seen=set();rows=0
 for seg in segs:
  start,end=seg.get('start'),seg.get('end')
  require(isinstance(start,int) and isinstance(end,int) and 0<=start<=end<=len(utf),'source-map byte span out of bounds')
  # Decode both sides as well as the selected slice: all span endpoints are UTF-8 boundaries.
  utf[:start].decode('utf-8');utf[start:end].decode('utf-8');utf[end:].decode('utf-8')
  origin=seg.get('origin');require(isinstance(origin,dict),'map segment has no origin')
  origins=[origin.get('primary'),*origin.get('derived',[])]
  for item in origins:
   require(isinstance(item,dict),'origin row malformed')
   bci=item.get('bci');ident=item.get('method')
   require(isinstance(bci,int) and isinstance(ident,dict),'physical origin missing BCI or method identity')
   require(bytes(ident.get('name',[]))==b'find' and bytes(ident.get('descriptor',[]))==b'([III)I','source-map origin has a foreign method identity')
   owner=ident.get('owner',{}).get('class_bytes',{}).get('digest')
   require(owner==digest,'source-map origin has a foreign owner class')
   seen.add(bci);rows+=1
 require(seen==set(physical),f'origin BCI set differs: missing={sorted(set(physical)-seen)} extra={sorted(seen-set(physical))}')
 require(text.count('limit()')==1,'find text must contain exactly one limit() invocation')
 require('savedField' in text,'find text omits field assignment')
 return {'method_text_sha256':sha(utf),'physical_bcis':sorted(physical),'origin_bcis':sorted(seen),'origin_rows':rows,'all_origins_exact':True,'utf8_spans_valid':True,'limit_mentions':text.count('limit()'),'savedField_mentions':text.count('savedField')}
def status_issue(items,msg):items.append(msg)
def main():
 d=json.loads(E.read_text(encoding='utf-8'));require(d.get('status')=='observed_complete_pending_independent_checker','capture did not finish')
 # Verify source/build/tool inputs are bound before and after the run.
 before=d['source_pins'];after=d['pins_after'];require(before==after['source_pins'],'source pins differ before/after')
 for p in before:require(pin(p['path'])==p,'source input live pin mismatch')
 for key in ('runner','guard','scanner','build_receipt','cli','jadx_manifest','jadx_plan','jadx_launcher'):
  require(d[key]==after[key],f'{key} pin differs before/after');require(pin(d[key]['path'])==d[key],f'{key} live pin differs')
 receipt=json.loads(BUILD.read_text(encoding='utf-8'));require(pin(BUILD)==d['build_receipt'],'CLI build receipt changed')
 require(receipt['status']=='validation-passed-cli-frozen-target-cleaned','build receipt status mismatch')
 require(receipt['guard_limits']=={'free_bytes':5*1024**3,'target_bytes':1024**3},'build guard limits differ')
 require(receipt['guard_template']=={'path':d['guard']['path'],'sha256':d['guard']['sha256']},'guard template is not build-bound')
 require(receipt['target_size_scan_adapter']=={'path':d['scanner']['path'],'sha256':d['scanner']['sha256']},'scanner is not build-bound')
 require(receipt['target_absent_after_clean'] and not (ROOT/'target').exists(),'target not absent after build cleanup')
 require(receipt['source_paths_before']==receipt['source_paths_after'],'build source path set changed')
 require(receipt['source_pins_before']==receipt['source_pins_after'],'build source pins changed')
 closure=receipt['freeze']['product_source_pins'];require(closure==receipt['source_pins_before']['full_product_source_and_include_closure'],'build closure differs')
 require(len(closure)==4698 and set(closure)==set(receipt['source_paths_before']),'build closure path count/set mismatch')
 require(d['build_closure']['source_path_count']==len(receipt['source_paths_before']),'recorded build source count mismatch')
 require(d['build_closure']['source_paths_sha256']==sha(json.dumps(receipt['source_paths_before'],separators=(',',':')).encode()),'build source path sequence pin mismatch')
 require(d['build_closure']['source_pins_sha256']==sha(json.dumps(receipt['source_pins_before'],sort_keys=True,separators=(',',':')).encode()),'build closure before pin mismatch')
 require(d['pins_after']['product_source_pins']==closure,'live after pins not equal complete build closure')
 for rel,digest in closure.items():require(sha((ROOT/rel).read_bytes())==digest,f'live closure file changed: {rel}')
 frozen=receipt['freeze']['frozen_cli'];require(frozen==d['build_closure']['frozen_cli'],'frozen CLI receipt not bound')
 built=receipt['freeze']['built_binary'];require(built['size_bytes']==frozen['size_bytes'] and built['sha256']==frozen['sha256'],'built binary and frozen CLI bytes differ')
 require(not pathlib.Path(built['path']).exists(),'cleaned build binary still exists in target')
 cli=pin(CLI);require(cli==d['cli'] and cli['path']==frozen['path'] and cli['bytes']==frozen['size_bytes'] and cli['sha256']==frozen['sha256'],'frozen CLI live bytes/path mismatch')
 require(frozen['mode']=='0o555' and stat.S_IMODE(CLI.stat().st_mode)==0o555,'frozen CLI mode mismatch')
 require(d['build_closure']['limits']==receipt['guard_limits'],'build limits not recorded')
 # Recorded guard and scanner must be the existing validated pair; every command must pass its guard.
 for c in receipt['commands']:
  require(c.get('guard_stop') is None and c.get('exit_code')==0,'build command failed or guard stopped')
  require(c.get('peak_target_bytes',0)<=1024**3 and c.get('free_bytes_after',0)>=5*1024**3,'build command violated target/free guard')
 for c in d['commands']:
  require(c.get('guard_stop') is None,'existing run guard stopped a command')
  require(c.get('peak_target_bytes',0)<=1024**3 and c.get('free_bytes_after',0)>=5*1024**3,'recorded command violates target/free guard')
  require(set(ENV_KEYS).issubset(set(c.get('env_overrides',{}))|set(c.get('removed_env_keys',[]))),f'env sanitization missing for {c.get("label")}')
  require(c['streams']['stdout']['bytes']>=0 and c['streams']['stderr']['bytes']>=0,'stream size missing')
 # Verify frozen JADX/JDK manifests, all 63 jars, and both JDK tool maps.
 manifest=json.loads(pathlib.Path(d['jadx_manifest']['path']).read_text(encoding='utf-8'))
 require(manifest['jadx']['expected_version']=='1.5.6' and d['jadx_launcher']['path']==manifest['jadx']['resolved_launcher'] and d['jadx_launcher']['sha256']==manifest['jadx']['sha256'],'JADX launcher/version manifest mismatch')
 require(pathlib.Path(manifest['jadx']['launcher']).resolve()==pathlib.Path(manifest['jadx']['resolved_launcher']),'JADX launcher symlink resolution mismatch')
 plan=json.loads(pathlib.Path(d['jadx_plan']['path']).read_text(encoding='utf-8'));expected_deps=[]
 for group in ('test_sdk_jars','jadx_product_jars'):
  expected_deps.extend({'path':r['path'],'bytes':r['bytes'],'sha256':r['sha256']} for r in plan['shared_pins'][group])
 require(d['dependency_jars']==expected_deps,'63 dependency pin set/order differs from frozen plan')
 require(d['jdk_tools_expected']=={leg:{tool:ref['sha256'] for tool,ref in manifest['jdk_legs'][leg]['tools'].items()} for leg in ('javac8','javac23')},'expected JDK tool SHAs differ from manifest')
 require(len(d['dependency_jars'])==63,'expected 63 jars')
 require(d['dependency_jars']==after['dependency_jars'],'jar pins differ before/after')
 for fact in d['dependency_jars']:require(pin(fact['path'])==fact,'dependency jar live pin mismatch')
 require(len(d['legs'])==2,'expected two JDK legs')
 for leg in d['legs']:
  name=leg['jdk'];expected=d['jdk_tools_expected'][name];final=after['jdk_tools'][name]
  require(set(leg['tool_pins'])==set(expected)==set(final),'JDK tool set differs')
  for tool,expectsha in expected.items():
   fact=leg['tool_pins'][tool];require(fact['sha256']==expectsha,f'{name}/{tool} sha differs from manifest')
   require(final[tool]==fact and pin(fact['path'])==fact,f'{name}/{tool} live/final pin differs')
 # All commands bind their executable to the pinned tool or CLI/launcher, and raw streams are checked.
 cmds=command_index(d);indexes=sorted(c['index'] for c in d['commands']);require(indexes==list(range(1,len(indexes)+1)),'command indexes are not contiguous')
 require(len(cmds)==len(d['commands']),'command label/index mapping is incomplete')
 runner_path=pathlib.Path(d['runner']['path']);require(pin(runner_path)==d['runner'],'runner self pin mismatch')
 jmanifest=json.loads(pathlib.Path(d['jadx_manifest']['path']).read_text());launcher=jmanifest['jadx']['resolved_launcher']
 for label,c in cmds.items():
  argv=c['argv'];leg=next((x for x in d['legs'] if x['jdk'] in label),None)
  if label in ('jadx-version','jadx-decompile'):require(pathlib.Path(argv[0])==pathlib.Path(launcher),'JADX argv not manifest launcher')
  if label=='jadx-decompile':
   require(d['jadx'].get('jar') is not None and argv[-1]==d['jadx']['jar']['path'] and argv[argv.index('-d')+1]==str(OUT/'jadx'),'JADX decompile input/output paths are not bound')
   require(argv[1:6]==['--no-res','--config','none','--threads-count','1'],'JADX decompile mode differs from frozen request')
  elif label.startswith('jarde-') and label.endswith('-render'):
   require(pathlib.Path(argv[0])==CLI,'Jarde CLI argv not frozen CLI')
   legrec=next(x for x in d['legs'] if x['jdk'] in label);require(str(legrec['original_class']['path']) in argv,'Jarde render input class not bound')
  elif '-javap' in label:require(leg is not None and pathlib.Path(argv[0])==pathlib.Path(leg['tool_pins']['javap']['path']),'javap argv/tool mismatch')
  elif label.endswith('-compile'):require(leg is not None and pathlib.Path(argv[0])==pathlib.Path(leg['tool_pins']['javac']['path']),'javac argv/tool mismatch')
  elif 'runtime' in label:require(leg is not None and pathlib.Path(argv[0])==pathlib.Path(leg['tool_pins']['java']['path']),'java runtime argv/tool mismatch')
 # Validate isolated class inventories and compile/runtime command path bindings.
 for leg in d['legs']:
  if leg['original_class']:
   cls=pathlib.Path(leg['original_class']['path']);require(pin(cls)==leg['original_class'],'original class pin changed')
   classdir=pathlib.Path(leg['original_class']['path']).parent
   require(inv(classdir,leg['original_inventory'])==leg['original_inventory'],'original class inventory changed')
   ri=leg.get('original_runtime_inventory');require(isinstance(ri,dict) and ri.get('before')==leg['original_inventory'] and ri.get('after')==leg['original_inventory'] and ri.get('unchanged') is True,'original runtime class inventory changed across execution')
   require({'SharedStoreInput.class','SharedStoreInputRunner.class'}<={x['relative'] for x in ri['before']},'original runtime target/runner classes were not present before execution')
   c=cmds[next(c['label'] for c in d['commands'] if c['index']==leg['original_compile_command'])]
   require(c['exit_code']==0 and str(classdir) in c['argv'],'original compile command lacks successful isolated classes output')
   require(c['argv'][c['argv'].index('-classpath')+1]==str(OUT/'empty-classpath') and c['argv'][c['argv'].index('-sourcepath')+1]==str(OUT/'empty-classpath'),'original compile did not use explicit empty class/source paths')
   require(str(OUT/leg['jdk']/'original'/'classes') in c['argv'],'original compile output path mismatch')
   require(str(pathlib.Path(d['source_pins'][0]['path'])) in c['argv'] and str(pathlib.Path(d['source_pins'][1]['path'])) in c['argv'],'original compile command lacks pinned source and Runner')
   if leg['original_runtime_command'] is not None:
    rc=next(c for c in d['commands'] if c['index']==leg['original_runtime_command'])
    require(rc['exit_code']==0 and '-Xverify:all' in rc['argv'] and rc['argv'][rc['argv'].index('-cp')+1]==str(classdir) and rc['argv'][-1]=='SharedStoreInputRunner','original runtime not bound to verified original classes')
   if leg['physical_javap_command'] is not None:
    jp=next(c for c in d['commands'] if c['index']==leg['physical_javap_command'])
    require(jp['exit_code']==0 and jp['argv'][jp['argv'].index('-classpath')+1]==str(classdir) and jp['argv'][-2:]==['-v','SharedStoreInput'],'javap is not bound to original compiled class')
 # The fresh JADX input is exactly the original javac8 class as a one-entry jar.
 if d['jadx'].get('jar'):
  import zipfile
  jarref=d['jadx']['jar'];jp=pathlib.Path(jarref['path']);require(pin(jp)['sha256']==jarref['sha256'],'JADX jar pin changed')
  with zipfile.ZipFile(jp) as z:
   require(z.namelist()==['SharedStoreInput.class'],'JADX jar is not a one-class jar')
   javac8=next(x for x in d['legs'] if x['jdk']=='javac8')
   require(z.read('SharedStoreInput.class')==pathlib.Path(javac8['original_class']['path']).read_bytes(),'JADX jar class differs from original javac8 bytes')
 # Original behavior is the independent oracle and must match the exact source-derived four rows.
 expected=b'hit:return=2,calls=1,saved=2\nmiss:return=-1,calls=1,saved=2\nbelow-lower:return=-1,calls=1,saved=2\nthrow:throw=limit,calls=1,saved=-99\n'
 original_issues=[];original_outputs={}
 for leg in d['legs']:
  if leg['original_runtime_command'] is None:status_issue(original_issues,leg['jdk']+' original runtime unavailable');continue
  c=next(c for c in d['commands'] if c['index']==leg['original_runtime_command']);out=raw(c['streams']['stdout']);err=raw(c['streams']['stderr'])
  if c['exit_code']!=0 or err or out!=expected:status_issue(original_issues,leg['jdk']+' original runtime differs from source oracle')
  summary=leg.get('original_runtime_result',{})
  if summary.get('exit_code')!=c['exit_code'] or summary.get('stdout_sha256')!=sha(out) or summary.get('stderr_sha256')!=sha(err):status_issue(original_issues,leg['jdk']+' original runtime summary differs from raw streams')
  original_outputs[leg['jdk']]=out
 original_equal=len(original_outputs)==2 and original_outputs['javac8']==original_outputs['javac23']
 if not original_equal:status_issue(original_issues,'original JDK runtime outputs differ')
 # Confirm the expected dup/field/local shared physical producer on both compilers.
 shared_facts=[];originals_ok=not original_issues
 for leg in d['legs']:
  if leg['physical_javap_command'] is None:originals_ok=False;continue
  c=next(c for c in d['commands'] if c['index']==leg['physical_javap_command'])
  if c['exit_code']!=0:status_issue(original_issues,leg['jdk']+' javap failed');originals_ok=False;continue
  try:
   ins=javap_instructions(raw(c['streams']['stdout']));shared_facts.append({'jdk':leg['jdk'],**shared_pattern(ins)})
  except AssertionError as e:status_issue(original_issues,leg['jdk']+' bytecode: '+str(e));originals_ok=False
 # Jarde report/source/runtime evidence is mandatory independently of any JADX result.
 jarde_issues=[];jarde_summaries=[];docs={}
 for leg in d['legs']:
  if not leg['original_class']:status_issue(jarde_issues,leg['jdk']+' original class absent');continue
  if {x['profile'] for x in leg['renders']}!={'default','all'}:status_issue(jarde_issues,leg['jdk']+' does not have both evidence profiles')
  for row in leg['renders']:
   mode=row['profile'];key=(leg['jdk'],mode)
   if row['render_exit']!=0:status_issue(jarde_issues,f'{key} CLI render exit {row["render_exit"]}');continue
   rc=cmds.get(f'jarde-{leg["jdk"]}-{mode}-render')
   if rc is None:status_issue(jarde_issues,f'{key} render command missing');continue
   input_ix=rc['argv'].index('--input')+1
   if pathlib.Path(rc['argv'][input_ix])!=pathlib.Path(leg['original_class']['path']) or row.get('class_input')!=leg['original_class']:status_issue(jarde_issues,f'{key} CLI input is not pinned original class')
   args=rc['argv'];required_args=['--class','SharedStoreInput','--policy','single-class','--release','8','--format','json']
   evidence_ok=(args[-2:]==['--evidence','all']) if mode=='all' else ('--evidence' not in args)
   if any(x not in args for x in required_args) or not evidence_ok:status_issue(jarde_issues,f'{key} CLI request flags differ')
   if rc['exit_code']!=row['render_exit']:status_issue(jarde_issues,f'{key} render result differs from command')
   try:doc=json.loads(raw(rc['streams']['stdout']).decode('utf-8'))
   except Exception as e:status_issue(jarde_issues,f'{key} JSON invalid: {e}');continue
   docs[key]=doc
   if row.get('report_sha256')!=sha(raw(rc['streams']['stdout'])):status_issue(jarde_issues,f'{key} report raw pin differs')
   if doc.get('text') is None or not row.get('source') or pathlib.Path(row['source']['path']).read_bytes()!=doc['text'].encode('utf-8') or pin(row['source']['path'])!=row['source']:
    status_issue(jarde_issues,f'{key} complete JSON source mismatch');continue
   if row.get('text_equals_json_utf8') is not True:status_issue(jarde_issues,f'{key} source save flag false')
   classdir=pathlib.Path(row['source']['path']).parent/'classes'
   inv(classdir,row['class_inventory'])
   if row.get('compile_exit')!=0:status_issue(jarde_issues,f'{key} generated class compile failed');continue
   compilecmd=next(c for c in d['commands'] if c['index']==row['compile_command'])
   if compilecmd['exit_code']!=row['compile_exit'] or compilecmd['exit_code']!=0 or str(row['source']['path']) not in compilecmd['argv'] or str(pathlib.Path(d['source_pins'][1]['path'])) not in compilecmd['argv'] or str(classdir) not in compilecmd['argv'] or compilecmd['argv'][compilecmd['argv'].index('-classpath')+1]!=str(OUT/'empty-classpath') or compilecmd['argv'][compilecmd['argv'].index('-sourcepath')+1]!=str(OUT/'empty-classpath'):
    status_issue(jarde_issues,f'{key} compile is not bound to rendered source/Runner/output dir');continue
   runtimeidx=row.get('runtime_command')
   if runtimeidx is None:status_issue(jarde_issues,f'{key} has no runtime');continue
   rt=next(c for c in d['commands'] if c['index']==runtimeidx);out=raw(rt['streams']['stdout']);err=raw(rt['streams']['stderr'])
   if '-cp' not in rt['argv'] or rt['argv'][rt['argv'].index('-cp')+1]!=str(classdir) or rt['argv'][-1]!='SharedStoreInputRunner' or '-Xverify:all' not in rt['argv']:status_issue(jarde_issues,f'{key} runtime command not bound to isolated verified classes')
   ri=row.get('runtime_inventory')
   if not isinstance(ri,dict) or ri.get('before')!=row['class_inventory'] or ri.get('after')!=row['class_inventory'] or ri.get('unchanged') is not True:status_issue(jarde_issues,f'{key} runtime class inventory changed')
   elif not {'SharedStoreInput.class','SharedStoreInputRunner.class'}<={x['relative'] for x in ri['before']}:status_issue(jarde_issues,f'{key} runtime target/runner classes absent before execution')
   expected_original=original_outputs.get(leg['jdk'])
   if rt['exit_code']!=row.get('runtime_exit') or rt['exit_code']!=0 or err or out!=expected_original:status_issue(jarde_issues,f'{key} runtime differs from original')
   if row.get('runtime_stdout_sha256')!=sha(out) or row.get('runtime_stderr_sha256')!=sha(err):status_issue(jarde_issues,f'{key} runtime summary differs from raw streams')
   if not row.get('runtime_matches_original'):status_issue(jarde_issues,f'{key} runtime result flag false')
   jarde_summaries.append({'jdk':leg['jdk'],'profile':mode,'report_sha256':row['report_sha256'],'source':row['source'],'class_inventory':row['class_inventory'],'runtime_equal':rt['exit_code']==0 and not err and out==expected_original})
   if leg['physical_javap_command'] is not None:
    javaprow=next(c for c in d['commands'] if c['index']==leg['physical_javap_command'])
    if javaprow['exit_code']==0:
     try:
      ins=javap_instructions(raw(javaprow['streams']['stdout']));maps=check_exact_find_map(doc,{b for b,_,_ in ins})
     except AssertionError as e:status_issue(jarde_issues,f'{key} source map: {e}');continue
     summary=jarde_summaries[-1];summary['source_map']=maps
  # Both profiles use the same full class source and same per-method text/source maps.
  if (leg['jdk'],'default') in docs and (leg['jdk'],'all') in docs:
   a,b=docs[(leg['jdk'],'default')],docs[(leg['jdk'],'all')]
   if a['text']!=b['text'] or methods_text_maps(a)!=methods_text_maps(b):status_issue(jarde_issues,leg['jdk']+' default/all complete text or method maps differ')
 # JADX has its own reported status; differences do not mask or skip the Jarde checks above.
 jadx_issues=[];jadx_results=d['jadx'].get('results',[])
 vc=next((c for c in d['commands'] if c['index']==d['jadx'].get('version_command')),None)
 if vc is None or vc['exit_code']!=0 or raw(vc['streams']['stdout']).strip()!=b'1.5.6':status_issue(jadx_issues,'JADX version probe failed')
 dc=next((c for c in d['commands'] if c['index']==d['jadx'].get('decompile_command')),None)
 if dc is None or dc['exit_code']!=0:status_issue(jadx_issues,'JADX decompile failed')
 adapter=d['jadx'].get('adapter')
 if d['jadx'].get('source') is None:status_issue(jadx_issues,'JADX source discovery unavailable')
 if adapter:
  if adapter['original_runner']!=d['source_pins'][1] or pin(adapter['adapted_runner']['path'])!=adapter['adapted_runner']:status_issue(jadx_issues,'JADX Runner source pins are not bound')
  op=pathlib.Path(adapter['original_runner']['path']).read_bytes();ap=pathlib.Path(adapter['adapted_runner']['path']).read_bytes();prefix=adapter['prefix'].encode()
  if ap!=prefix+op or not adapter['body_exact']:status_issue(jadx_issues,'JADX Runner adapter changed bytes outside exact prefix')
  src=pathlib.Path(d['jadx']['source']['path']).read_bytes()
  pkg=re.findall(rb'(?m)^[ \t]*package[ \t]+([A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*)[ \t]*;[ \t]*$',src)
  expected_prefix=(b'package '+pkg[0]+b';\n') if pkg else b''
  if len(pkg)>1 or expected_prefix!=prefix:status_issue(jadx_issues,'JADX Runner prefix is not the exact source package declaration')
  if pin(d['jadx']['source']['path'])!=d['jadx']['source'] or not adapter['source_exact_copy']:status_issue(jadx_issues,'JADX source pin/copy mismatch')
 for jr in jadx_results:
  classdir=pathlib.Path(jr['classes_dir'])
  cc=next(c for c in d['commands'] if c['index']==jr['compile_command'])
  if cc['exit_code']!=jr['compile_exit'] or str(jr['source']['path']) not in cc['argv'] or str(jr['runner_source']['path']) not in cc['argv'] or str(classdir) not in cc['argv'] or cc['argv'][cc['argv'].index('-classpath')+1]!=str(OUT/'empty-classpath') or cc['argv'][cc['argv'].index('-sourcepath')+1]!=str(OUT/'empty-classpath'):status_issue(jadx_issues,jr['jdk']+' JADX compile source/Runner/output binding mismatch')
  inv(classdir,jr['class_inventory'])
  if jr['compile_exit']!=0:status_issue(jadx_issues,jr['jdk']+' JADX source compile failed');continue
  if pin(jr['source']['path'])!=jr['source']:status_issue(jadx_issues,jr['jdk']+' JADX source input changed')
  if jr.get('runtime_command') is None:status_issue(jadx_issues,jr['jdk']+' JADX runtime missing');continue
  ri=jr.get('runtime_inventory')
  if not isinstance(ri,dict) or ri.get('before')!=jr['class_inventory'] or ri.get('after')!=jr['class_inventory'] or ri.get('unchanged') is not True:status_issue(jadx_issues,jr['jdk']+' JADX runtime class inventory changed')
  elif not {pathlib.PurePosixPath((d['jadx']['adapter']['package']+'.').replace('.', '/')+'SharedStoreInput.class').as_posix(),pathlib.PurePosixPath((d['jadx']['adapter']['package']+'.').replace('.', '/')+'SharedStoreInputRunner.class').as_posix()}<={x['relative'] for x in ri['before']}:status_issue(jadx_issues,jr['jdk']+' JADX runtime target/Runner classes absent before execution')
  rcmd=next(c for c in d['commands'] if c['index']==jr['runtime_command']);got=raw(rcmd['streams']['stdout']);err=raw(rcmd['streams']['stderr'])
  expected_main=(d['jadx']['adapter']['package']+'.SharedStoreInputRunner') if d['jadx'].get('adapter',{}).get('package') else 'SharedStoreInputRunner'
  if jr.get('runtime_exit')!=rcmd['exit_code'] or '-cp' not in rcmd['argv'] or rcmd['argv'][rcmd['argv'].index('-cp')+1]!=str(classdir) or rcmd['argv'][-1]!=expected_main or '-Xverify:all' not in rcmd['argv'] or rcmd['exit_code']!=0 or err or got!=original_outputs.get('javac8') or jr.get('runtime_stdout_sha256')!=sha(got) or jr.get('runtime_stderr_sha256')!=sha(err) or not jr.get('runtime_matches_original'):status_issue(jadx_issues,jr['jdk']+' JADX runtime differs from original oracle')
 jadx_status='matched' if not jadx_issues else 'observed_difference_or_failure'
 jarde_status='matched' if not jarde_issues and len(jarde_summaries)==4 else 'failed_or_incomplete'
 original_status='matched' if not original_issues and originals_ok else 'failed_or_incomplete'
 result={'status':'checked','execution':str(E),'command_count':len(d['commands']),'original':{'status':original_status,'issues':original_issues,'runtime_sha256':{k:sha(v) for k,v in original_outputs.items()},'shared_bytecode':shared_facts},'jarde':{'status':jarde_status,'issues':jarde_issues,'renders':jarde_summaries},'jadx':{'status':jadx_status,'issues':jadx_issues,'results':jadx_results},'limits_and_pins':'verified'}
 (OUT/'verification.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8');print(json.dumps(result,indent=2))
if __name__=='__main__':main()
