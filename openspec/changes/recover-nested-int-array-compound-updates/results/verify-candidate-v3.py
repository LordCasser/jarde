#!/usr/bin/env python3
"""Read-only independent verifier for candidate-v2's four frozen-input legs."""
import hashlib, json, sys
from pathlib import Path

ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS=ROOT/'openspec/changes/recover-nested-int-array-compound-updates/results'
OUT=RESULTS/'candidate-v2'
P02_ROOT=ROOT/'openspec/changes/recover-bigdecimal-number-widening/results/nested-array-update-baseline-v1'
P02_MANIFEST=P02_ROOT/'manifest.json'
CONTROLS=RESULTS/'controls-v1/manifest.json'
NESTED_BASELINE=RESULTS/'controls-baseline-v1/manifest.json'
PRODUCTS={'crates/jarde-java/src/init.rs','crates/jarde-java/src/report.rs','crates/jarde-java/src/build.rs','src/class_source.rs','Cargo.lock'}
TESTS={'tests/p3_nested_int_array_compound_updates.rs','tests/recover_lambda_primitive_array_capture.rs','tests/p3_constructed_reference_array_elements.rs','tests/recover_boxed_number_widening.rs','.github/workflows/ci.yml'}
P02_BCIS={0,1,2,3,4,5,6,7,10,11,12}
NESTED_METHODS={'<init>','plain2','plain3','scalar','traced','row','index','rhs','swap','replaceRow','different'}
COMPOUND={'plain2','plain3','scalar','traced','replaceRow'}

def sha(b): return hashlib.sha256(b).hexdigest()
def load(p): return json.loads(Path(p).read_text())
def filebytes(p):
 p=Path(p); return (p if p.is_absolute() else ROOT/p).read_bytes()
def verify_file_row(base,row):
 p=Path(row['path']); p=p if p.is_absolute() else base/p
 b=p.read_bytes(); return len(b)==row['bytes'] and sha(b)==row['sha256']
def expect(ok,msg,errors):
 if not ok: errors.append(msg)
def member_map(rows): return {r['name']:r for r in rows}
def raw_members(doc):
 out=[]
 for row in doc.get('methods',[]):
  item=row.get('item',{}); outcome=row.get('outcome',{}); recovered=outcome.get('report') if outcome.get('kind')=='recovered' else None
  sm=recovered.get('source_map') if recovered else None; bcis=set()
  for segment in (sm or {}).get('segments',[]):
   origin=segment.get('origin',{}); primary=origin.get('primary',{}); b=primary.get('bci')
   if isinstance(b,int): bcis.add(b)
   bcis.update(x['bci'] for x in origin.get('derived',[]) if isinstance(x.get('bci'),int))
  out.append({'name':item.get('name',{}).get('escaped'),'descriptor':item.get('descriptor',{}).get('escaped'),
   'outcome':outcome.get('kind'),'quality':recovered.get('quality') if recovered else None,
   'representation':recovered.get('representation') if recovered else None,'markers':row.get('markers',[]),
   'body':recovered.get('text','') if recovered else '', 'source_map':sm,'bcis':sorted(bcis)})
 return out
def verify():
 errors=[]; checks=0
 mpath=OUT/'manifest.json'
 if not mpath.is_file(): raise SystemExit(f'missing {mpath}')
 m=load(mpath); p02=load(P02_MANIFEST); controls=load(CONTROLS); nested=load(NESTED_BASELINE)
 expect(m.get('schema')=='nested-int-array-compound-candidate-v2','candidate manifest schema',errors); checks+=1
 expect(m.get('expected_denominator')==4 and len(m.get('cases',[]))==4,'four-case denominator',errors); checks+=1
 # Historical references are byte-bound and explicitly reused rather than fresh runs.
 wantrefs={str(P02_MANIFEST):sha(filebytes(P02_MANIFEST)),str(CONTROLS):sha(filebytes(CONTROLS)),str(NESTED_BASELINE):sha(filebytes(NESTED_BASELINE))}
 refs={r['path']:r for r in m.get('historical_baselines',[])}
 for p,h in wantrefs.items():
  expect(p in refs and refs[p]['sha256']==h and refs[p]['fresh_execution_claim'] is False,f'historical baseline binding {p}',errors); checks+=1
 expect(nested['controls_input']['manifest_sha256']==wantrefs[str(CONTROLS)],'nested baseline binds controls input',errors); checks+=1
 for title,base,rows in [('P02 frozen files',P02_ROOT,p02.get('files',[])),('controls fixture',ROOT,controls.get('fixture_files',[])),('controls baseline',ROOT,nested.get('file_inventory',[]))]:
  for row in rows: expect(verify_file_row(base,row),f'{title}: {row["path"]}',errors); checks+=1
 for leg in ('javac8','javac23'):
  pc=next(x for x in p02['cases'] if x['leg']==leg); cl=next(x for x in controls['legs'] if x['leg']==leg)
  for tool in ('java','javac','javap'):
   for source in (pc['tools'][tool],cl['jdk_tools'][tool]):
    b=filebytes(source['path']); expect(len(b)==source['bytes'] and sha(b)==source['sha256'],f'{leg} frozen {tool} binary',errors); checks+=1
   expect((pc['tools'][tool]['path'],pc['tools'][tool]['sha256'])==(cl['jdk_tools'][tool]['path'],cl['jdk_tools'][tool]['sha256']),f'{leg} matching frozen {tool} identity',errors); checks+=1
 expect(m.get('runner',{}).get('sha256')==sha(filebytes(RESULTS/'replay-candidate-v2.py')),'runner source identity',errors); checks+=1
 # The recorded closed output inventory covers every produced file and raw stream.
 actual={str(p.relative_to(OUT)) for p in OUT.rglob('*') if p.is_file() and p.name!='manifest.json'}
 rows={r['path']:r for r in m.get('files',[])}
 expect(actual==set(rows),'candidate file inventory is closed',errors); checks+=1
 for name,row in rows.items(): expect(verify_file_row(OUT,row),f'file hash {name}',errors); checks+=1
 commands=m.get('commands',[]); bylabel={c['label']:c for c in commands}
 expect(len(commands)==12,'three actual subprocesses per case',errors); checks+=1
 for c in commands:
  for stream in ('stdout','stderr'):
   r=c[stream]; b=filebytes(OUT/r['path']); expect(len(b)==r['bytes'] and sha(b)==r['sha256'],f"command stream {c['label']} {stream}",errors); checks+=1
 p02cases={c['leg']:c for c in p02['cases']}; controllegs={x['leg']:x for x in controls['legs']}
 nestedjarde={x['leg']:x for x in nested['cases'] if x['kind']=='jarde'}
 for case in m['cases']:
  label=case['label']; ds=case['dataset']; leg=case['leg']; prefix=label
  expect(case['candidate_success'] is True and case['runtime_matches_original_exit_stdout_stderr'] is True,f'{label} success/raw equality',errors); checks+=1
  expect(case['original']['fresh_execution'] is False and case['original_stream_source']['fresh_execution'] is False,f'{label} historical oracle not fresh',errors); checks+=1
  expected_manifest=str(P02_MANIFEST if ds=='P02' else CONTROLS)
  expect(case['original_stream_source']['manifest']==expected_manifest and case['original_stream_source']['command_label']==('original-run' if ds=='P02' else 'run-runner'),f'{label} oracle provenance label',errors); checks+=1
  original=case['original'];
  for s in ('stdout','stderr'):
   r=original[s]; b=filebytes(OUT/r['path']); expect(len(b)==r['bytes'] and sha(b)==r['sha256'],f'{label} copied original {s}',errors); checks+=1
  expect(original['exit']==case['runtime']['exit'],f'{label} original and candidate exits',errors); checks+=1
  expect(case['compile']['exit']==0 and case['runtime']['exit']==0,f'{label} compile/runtime exit zero',errors); checks+=1
  expect(case['input']['sha256']==case['input']['copied_input']['sha256'],f'{label} input copy identity',errors); checks+=1
  source=case['report']['source']; expect(source is not None and verify_file_row(OUT,source),f'{label} emitted source hash',errors); checks+=1
  source_text=filebytes(OUT/source['path']).decode(); expect(sha(source_text.encode())==case['report']['source_text_sha256'],f'{label} saved source hash',errors); checks+=1
  expect(case['report']['json_parsed'] and case['report']['parse_error'] is None,f'{label} JSON parsed',errors); checks+=1
  # Independently bind the renderer input and exact command shape.
  render=bylabel[prefix+'-render']; inp=OUT/case['input']['copied_input']['path']
  expect(render['argv'][0]==m['candidate_cli']['path'] and render['argv'][render['argv'].index('--input')+1]==str(inp),f'{label} actual render argv',errors); checks+=1
  expect(sha(filebytes(inp))==case['input']['sha256'],f'{label} copied class readable/hash',errors); checks+=1
  rawdoc=json.loads(filebytes(OUT/render['stdout']['path']))
  expect(rawdoc.get('text')==source_text,f'{label} rendered raw JSON text equals saved source',errors); checks+=1
  actual_methods=raw_members(rawdoc); actual=member_map(actual_methods); summary=member_map(case['report']['members'])
  expect(set(actual)==set(summary),f'{label} raw-vs-summary member closure',errors); checks+=1
  for name,item in actual.items():
   recorded=summary.get(name,{})
   expect(item['descriptor']==recorded.get('descriptor') and item['outcome']==recorded.get('outcome') and item['quality']==recorded.get('quality') and item['representation']==recorded.get('representation') and item['body']==recorded.get('body') and item['bcis']==recorded.get('source_map_all_bcis'),f'{label} raw-vs-summary facts {name}',errors); checks+=1
  comp=case['compile']; argv=comp['argv']; cp=Path(argv[argv.index('-classpath')+1]); sp=Path(argv[argv.index('-sourcepath')+1]); dest=Path(argv[argv.index('-d')+1])
  frozen_case=p02cases[leg] if ds=='P02' else None
  expected_tools=frozen_case['tools'] if frozen_case else controllegs[leg]['jdk_tools']
  expect(argv[0]==expected_tools['javac']['path'],f'{label} uses frozen javac path',errors); checks+=1
  expect(comp['java_home']==str(Path(expected_tools['java']['path']).parent.parent),f'{label} uses frozen Java home',errors); checks+=1
  expect(argv[1:argv.index('-classpath')]==case['compiler_flags'],f'{label} compiler flags preserved',errors); checks+=1
  expect(cp.is_dir() and not any(cp.iterdir()) and sp.is_dir() and not any(sp.iterdir()),f'{label} explicit empty classpath/sourcepath',errors); checks+=1
  expect(str(dest)==str(OUT/'cases'/label/'candidate-classes'),f'{label} private classes destination',errors); checks+=1
  sources=[OUT/x['path'] for x in case['compile_sources']]
  actual_source_args=[Path(x) for x in argv[argv.index('-d')+2:]]
  expect(actual_source_args==sources and all(x.is_file() for x in sources),f'{label} all exact sources compiled',errors); checks+=1
  run=case['runtime']; ra=run['argv'];
  expect(ra[0]==expected_tools['java']['path'],f'{label} uses frozen java path',errors); checks+=1
  expect(run['java_home']==str(Path(expected_tools['java']['path']).parent.parent),f'{label} runtime Java home',errors); checks+=1
  expect('-Xverify:all' in ra and ra[ra.index('-cp')+1]==str(dest),f'{label} runtime uses only fresh verified classes',errors); checks+=1
  classfiles=sorted(p.relative_to(dest).as_posix() for p in dest.rglob('*.class'))
  expect(classfiles==sorted(Path(r['path']).name for r in case['candidate_classes']),f'{label} fresh class closure',errors); checks+=1
  class_rows=case['candidate_classes']
  expect(all((dest/r['path']).is_file() and verify_file_row(dest,r) for r in class_rows),f'{label} generated class hashes',errors); checks+=1
  # Re-open immutable oracle bytes from their original manifest, not just the copied records.
  if ds=='P02':
   bc=p02cases[leg]; src=bc['input'];
   expect(filebytes(src['path'])==filebytes(inp) and sha(filebytes(inp))==src['sha256'],f'{label} P02 original class identity',errors); checks+=1
   oracle=next(x for x in p02['commands'] if x['leg']==leg and x['label']=='original-run')
   for s in ('stdout','stderr'): expect(filebytes(oracle[s]['path'])==filebytes(OUT/original[s]['path']),f'{label} P02 historical {s} bytes',errors); checks+=1
   methods=actual; expect(set(methods)=={'<init>','sum','main','lambda$sum$0'},f'{label} complete P02 members',errors); checks+=1
   helper=methods.get('lambda$sum$0',{}); expect(helper.get('descriptor')=='([[ILjava/lang/Integer;)V' and helper.get('outcome')=='recovered' and helper.get('quality')=='structured' and helper.get('representation')=='java',f'{label} P02 helper structured',errors); checks+=1
   expect(set(helper.get('bcis',[]))==P02_BCIS,f'{label} P02 physical 11-BCI map',errors); checks+=1
   historical={x['name']:x for x in bc['members']}
   for n in ('<init>','sum','main'): expect(methods[n]['body']==historical[n]['body'],f'{label} preserved historical {n} body',errors); checks+=1
  else:
   legrow=controllegs[leg]; cls=next(x for x in legrow['class_files'] if Path(x['path']).name=='NestedIntUpdates.class')
   expect(filebytes(cls['path'])==filebytes(inp) and sha(filebytes(inp))==cls['sha256'],f'{label} Nested original class identity',errors); checks+=1
   oracle_cmd=next(x for x in legrow['commands'] if x['label']=='run-runner')
   expect(oracle_cmd['exit']==0 and controls['expected_runner_stdout']['utf8'].encode()==filebytes(CONTROLS.parent/oracle_cmd['stdout']['path']),f'{label} frozen Nested expected behavior',errors); checks+=1
   for s in ('stdout','stderr'):
    oracle_path=CONTROLS.parent/oracle_cmd[s]['path']; expect(filebytes(oracle_path)==filebytes(OUT/original[s]['path']),f'{label} Nested historical {s} bytes',errors); checks+=1
   expect(nestedjarde[leg]['original_runtime']['stdout']['sha256']==oracle_cmd['stdout']['sha256'],f'{label} Nested baseline runtime binding',errors); checks+=1
   methods=actual; expect(set(methods)==NESTED_METHODS,f'{label} all 11 Nested methods',errors); checks+=1
   for n,x in methods.items():
    expect(x.get('outcome')=='recovered' and x.get('quality')=='structured' and x.get('representation')=='java' and not x.get('markers') and 'jarde_refused_body' not in x.get('body',''),f'{label} method recovered {n}',errors); checks+=1
    expect(bool(x.get('source_map')) and bool(x.get('bcis')),f'{label} source map {n}',errors); checks+=1
   for n in COMPOUND: expect('+=' in methods[n]['body'],f'{label} compound body {n}',errors); checks+=1
   expect('=' in methods['different']['body'] and '+=' not in methods['different']['body'],f'{label} different remains ordinary assignment',errors); checks+=1
   runner_rows=[x for x in case['compile_sources'] if Path(x['path']).name=='Runner.java']
   expect(len(runner_rows)==1 and runner_rows[0]['sha256']==controls['source_files'][1]['sha256'],f'{label} frozen Runner source',errors); checks+=1
 # Candidate CLI + metadata paths and five production/five test source identities.
 cli=m['candidate_cli']; meta=load(cli['metadata_path']);
 expect(filebytes(cli['path']) and sha(filebytes(cli['path']))==cli['sha256']==meta['cli_sha256'], 'candidate CLI bytes and metadata identity',errors); checks+=1
 expect(sha(filebytes(cli['metadata_path']))==cli['metadata_sha256'],'candidate metadata hash',errors); checks+=1
 expect(set(meta['candidate_sources'])==PRODUCTS and set(meta['test_sources'])==TESTS,'metadata source closure',errors); checks+=1
 for rel,h in {**meta['candidate_sources'],**meta['test_sources']}.items(): expect(sha(filebytes(ROOT/rel))==h,f'current metadata source {rel}',errors); checks+=1
 report={'schema':'nested-int-array-compound-candidate-verification-v3','status':'passed' if not errors else 'failed','checks':checks,'errors':errors,
  'candidate_manifest_sha256':sha(filebytes(mpath)),'candidate_cli_sha256':cli['sha256'],'case_count':len(m['cases']),'successful_case_count':sum(c.get('candidate_success') is True for c in m['cases'])}
 target=RESULTS/'candidate-v2-verification-v3.json'
 if target.exists(): raise SystemExit(f'refusing to overwrite {target}')
 target.write_text(json.dumps(report,indent=2)+'\n')
 print(json.dumps(report,indent=2)); return 0 if not errors else 1

if __name__=='__main__': raise SystemExit(verify())
