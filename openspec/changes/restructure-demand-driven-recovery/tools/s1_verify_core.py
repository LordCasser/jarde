#!/usr/bin/env python3
"""Replay frozen reports across evidence selections; run only controlled fixture oracles."""
import gzip,json,sys,os
from pathlib import Path
sys.path.insert(0,str(Path.cwd()/'openspec/changes/restructure-demand-driven-recovery/tools'))
import s0_baseline as b
out=Path(sys.argv[1]).resolve();bins=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=True)
checks=[]
for name in ['root-jarde','java-jarde_java','java-p3_java_recovery','java-p3_patterns','java-class_initializer_candidates','java-anonymous_allocation_candidates','root-lambda_root_member_folding','root-member_class_static_folding','root-generic_outer_member_family','root-inner_class_static_mixed_folding']:
 binary=bins/name
 if not binary.exists():continue
 meta,text=b.run([binary,'--test-threads=1'],out/name,timeout=300)
 checks.append(dict(kind='unit',name=name,binary=b.pin(binary),exit_code=meta['exit_code'],summary=text.splitlines()[-3:]));b.write(out/'checks.json',checks);print(json.dumps(checks[-1]),flush=True)
cli=bins/'cli-jarde-cli';comparisons=[]
def outline(v,path=''):
 result={}
 if isinstance(v,dict):
  if 'evidence' in v and 'source_map' in v and 'text' in v:
   result[path]=dict(method=v.get('method'),outcome=v.get('outcome'),representation=v.get('representation'),quality=v.get('quality'),syntax_status=v.get('syntax_status'),content=v.get('content'),text=v.get('text'))
  for k,x in v.items():
   if k in ['evidence','source_map','usage']:continue
   if k in ['kind','status','state'] and isinstance(x,(str,int,bool)) and any(t in path for t in ['family','projection']):result[path+'/'+k]=x
   result.update(outline(x,path+'/'+k))
 elif isinstance(v,list):
  for i,x in enumerate(v):result.update(outline(x,path+'/'+str(i)))
 return result
def ensure_essential(v):
 if isinstance(v,dict):
  if 'evidence' in v and 'source_map' in v and 'text' in v:
   assert not v['source_map'].get('segments'),str(v['source_map'])[:80]
   assert not v.get('regions') and not v.get('news')
  for x in v.values():ensure_essential(x)
 elif isinstance(v,list):
  for x in v:ensure_essential(x)
for base in [b.CHANGE/'results/s0-baseline-v1',b.CHANGE/'results/s0-resumed-v2/expanded-matrix']:
 for row in json.loads((base/'matrix-results.json').read_text()):
  tag=row['id']+'-'+row['compiler']+'-'+row['debug'];source=base/'matrix'/tag
  old_cmd=json.loads((source/'recover-jarde.meta.json').read_text())['command'];old_cmd[0]=str(cli)
  assert b.pin(Path(row['input']['path']))==row['input']
  baseline=json.loads(gzip.decompress((source/'recover-jarde.stdout.gz').read_bytes()));reports={};runs={}
  for evidence in ['all','essential','source_map']:
   cmd=old_cmd.copy();cmd[cmd.index('--evidence')+1]=evidence
   d=out/'matrix'/tag/evidence;meta,text=b.run(cmd,d/'recover',timeout=120);candidate=json.loads(text)
   reports[evidence]=candidate;runs[evidence]=dict(exit_code=meta['exit_code'],process=meta)
   b.compress(d/'recover.stdout')
   if evidence=='essential':ensure_essential(candidate)
  result=dict(id=tag,input=row['input'],all_text_preserved=baseline.get('text')==reports['all'].get('text'),all_core_preserved=outline(baseline)==outline(reports['all']),evidence_text_equal=len({x.get('text') for x in reports.values()})==1,evidence_core_equal=all(outline(x)==outline(reports['all']) for x in reports.values()),runs=runs)
  comparisons.append(result);b.write(out/'matrix-comparison.json',comparisons)
  if not all(result[k] for k in ['all_text_preserved','all_core_preserved','evidence_text_equal','evidence_core_equal']): print('MISMATCH '+tag+' '+json.dumps({k:result[k] for k in ['all_text_preserved','all_core_preserved','evidence_text_equal','evidence_core_equal']}),flush=True)
# Same S0 E2 fixture, source is used verbatim, compilers and execution have no original class fallback.
base=b.CHANGE/'results/s0-baseline-v1/matrix/E2-dev-javac8-none';case=next(x for x in json.loads((b.CHANGE/'fixtures/s0-mechanisms-v1/matrix.json').read_text())['cases'] if x['id']=='E2-dev')
empty=out/'empty';empty.mkdir(exist_ok=True);oracle=[]
for evidence in ['essential','source_map','all']:
 raw=out/'matrix/E2-dev-javac8-none'/evidence/'recover.stdout.gz';report=json.loads(gzip.decompress(raw.read_bytes()))
 for label,jdk in [('javac8',b.J8),('javac23',b.J23)]:
  d=out/'compile-oracle'/(evidence+'-'+label);d.mkdir(parents=True,exist_ok=True);src=d/(case['class_name']+'.java');src.write_text(report['text']);classes=d/'classes';classes.mkdir()
  flags=['-source','8','-target','8'] if label=='javac8' else ['--release','8']
  meta,text=b.run([jdk/'bin/javac','-proc:none','-g:none','-sourcepath',empty,'-classpath',empty,*flags,'-d',classes,src],d/'compile')
  result=dict(evidence=evidence,compiler=label,source=b.pin(src),compile_exit=meta['exit_code'],runtimes={})
  if meta['exit_code']==0:
   for runtime,jre in [('jvm8',b.J8),('jvm23',b.J23)]:
    meta,text=b.run([jre/'bin/java','-Xverify:all','-classpath',str(base/'driver')+os.pathsep+str(classes),case['driver']],d/runtime)
    expected=(base/('original-'+runtime+'.stdout')).read_text();result['runtimes'][runtime]=dict(exit_code=meta['exit_code'],equal_to_original=text==expected)
  oracle.append(result);b.write(out/'compile-oracle.json',oracle);print('oracle '+json.dumps(result),flush=True)
# Recheck the two actually compilable real-package classes; do not execute target packages.
real=[]
for row in json.loads((b.CHANGE/'results/s0-baseline-v1/real-class-results.json').read_text()):
 if row['compilers'] != {'javac8':0,'javac23':0}: continue
 original=Path(row['source']['path']);assert b.pin(original)==row['source'];base_dir=original.parent
 d=out/'real-valid'/row['artifact']/original.stem;d.mkdir(parents=True,exist_ok=True)
 cmd=json.loads((base_dir/'recover.meta.json').read_text())['command'];cmd[0]=str(cli)
 meta,text=b.run(cmd,d/'recover',timeout=120);report=json.loads(text)
 src=d/original.name;src.write_text(report['text']);b.compress(d/'recover.stdout')
 item=dict(artifact=row['artifact'],name=row['name'],physical_definition=row['physical_definition'],text_equal=src.read_bytes()==original.read_bytes(),exit_code=meta['exit_code'],source=b.pin(src),compilers={})
 for compiler in ['javac8','javac23']:
  cmd=json.loads((base_dir/('compile-'+compiler+'.meta.json')).read_text())['command']
  target=d/compiler;target.mkdir();cmd[cmd.index('-d')+1]=str(target);cmd[-1]=str(src)
  meta,text=b.run(cmd,d/('compile-'+compiler),timeout=120);item['compilers'][compiler]=meta['exit_code']
 real.append(item);b.write(out/'real-valid.json',real);print('real '+json.dumps(item),flush=True)
summary=dict(unit_runs=len(checks),unit_failures=sum(x['exit_code']!=0 for x in checks),matrix_cases=len(comparisons),all_text_mismatches=sum(not x['all_text_preserved'] for x in comparisons),all_core_mismatches=sum(not x['all_core_preserved'] for x in comparisons),evidence_text_mismatches=sum(not x['evidence_text_equal'] for x in comparisons),evidence_core_mismatches=sum(not x['evidence_core_equal'] for x in comparisons),compile_cases=len(oracle),compile_failures=sum(x['compile_exit']!=0 for x in oracle),runtime_checks=sum(len(x['runtimes']) for x in oracle),runtime_failures=sum(not(r['exit_code']==0 and r['equal_to_original']) for x in oracle for r in x['runtimes'].values()))
summary.update(real_valid_classes=len(real),real_text_mismatches=sum(not x['text_equal'] for x in real),real_compile_failures=sum(c!=0 for x in real for c in x['compilers'].values()))
b.write(out/'summary.json',summary);print(json.dumps(summary),flush=True)
assert len(comparisons)==136 and all(x['exit_code']==0 for x in checks)
assert not any(summary[x] for x in ['unit_failures','all_text_mismatches','all_core_mismatches','evidence_text_mismatches','evidence_core_mismatches','compile_failures','runtime_failures'])
assert summary['runtime_checks']==12

assert len(real)==2 and all(x['text_equal'] and x['exit_code']==0 and all(c==0 for c in x['compilers'].values()) for x in real)
