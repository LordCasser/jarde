#!/usr/bin/env python3
"""Replay saved CLI8 class-source argv with CLI9 and compare outputs byte-for-byte.
Only nested usage.elapsed_millis is masked in the additional JSON semantic comparison.
"""
import argparse, hashlib, json, os, re, subprocess, sys
from pathlib import Path

HERE=Path(__file__).resolve().parent
CHANGE=Path(__file__).resolve().parents[3]
CAND=CHANGE/'results/candidate'
CLI8=Path('/private/tmp/jarde-generic-calls-candidate-v8-cli')
CLI9=Path('/private/tmp/jarde-generic-calls-candidate-v9-cli')
SOURCES=[
 ('matrix','candidate-v8/gc01-08-140/manifest.json','commands'),
 ('nested','candidate-v8/nested-call-4/manifest.json','commands'),
 ('field','gc09-field-23-v9/adapter-run/run-metadata.json','actual_commands'),
 ('constructor','gc09-constructor-80-v8/run-metadata.json','commands'),
 ('raw','gc09-raw-64-candidate-v8/outer-run-metadata.json','subprocess_records'),
]
def sha(p):
 h=hashlib.sha256()
 with open(p,'rb') as f:
  for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
 return h.hexdigest()
def norm_json_bytes(b):
 try: x=json.loads(b)
 except Exception:return None
 def walk(v,parent=None):
  if isinstance(v,dict):
   return {k:walk(val,k) for k,val in v.items() if not (k=='elapsed_millis' and parent=='usage')}
  if isinstance(v,list): return [walk(z,parent) for z in v]
  return v
 return (json.dumps(walk(x),sort_keys=True,separators=(',',':'),ensure_ascii=False)+'\n').encode()
def get_expected(base,cmd):
 if cmd.get('stdout'): return Path(cmd['stdout'])
 argv=cmd['argv']; cl=argv[argv.index('--class')+1]
 return Path(cmd['cwd'])/'jarde'/f'{cl}.java'
def get_expected_err(cmd):
 if cmd.get('stderr'): return Path(cmd['stderr'])
 argv=cmd['argv']; cl=argv[argv.index('--class')+1]
 return Path(cmd['cwd'])/'jarde'/f'jarde-{cl}.stderr'
def input_path(cmd):
 a=cmd['argv']; return Path(a[a.index('--input')+1])
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--output',required=True);ap.add_argument('--skip-unavailable-field',action='store_true');args=ap.parse_args()
 out=Path(args.output).resolve();out.mkdir(parents=True,exist_ok=False)
 records=[]; skips=[]; idx=0
 field=json.load(open(CAND/'gc09-field-23-v9/adapter-run/run-metadata.json'))
 reused={(x['family'],x['leg']):Path(x['jar']) for x in field.get('historical_jar_reuse_events',[])}
 orig={Path(x['temporary_path']):(x['family'],x['leg'],x) for x in field['temporary_original_jars']}
 for group,rel,key in SOURCES:
  data=json.load(open(CAND/rel)); cmds=data[key]
  for c in cmds:
   a=c.get('argv',[])
   if len(a)<2 or a[0]!=str(CLI8) or a[1]!='class-source': continue
   argv=list(a); src_input=input_path(c); input_actual=src_input
   expected_input_sha=None
   if group=='field':
    inf=orig.get(src_input)
    if inf:
     fam,leg,item=inf; expected_input_sha=item['jar_sha256']
     if not input_actual.is_file():
      if item.get('kind')=='historical-jar-byte-copy' and (fam,leg) in reused: input_actual=reused[(fam,leg)]
      else:
       skips.append({'group':group,'label':c.get('label'),'input':str(src_input),'family':fam,'leg':leg,'kind':item.get('kind'),'reason':'frozen temp input absent; separately assigned field18 recovery'})
       continue
    else:
     ev=next((x for x in field.get('historical_input_jars',[]) if x['path']==str(src_input)),None)
     if ev: expected_input_sha=ev['sha256']
   if not input_actual.is_file():
    skips.append({'group':group,'label':c.get('label'),'input':str(src_input),'reason':'input missing'});continue
   observed_input_sha=sha(input_actual)
   if expected_input_sha and observed_input_sha!=expected_input_sha:
    raise SystemExit(f'input SHA mismatch: {input_actual}: {observed_input_sha} != {expected_input_sha}')
   argv[0]=str(CLI9)
   if input_actual!=src_input: argv[argv.index('--input')+1]=str(input_actual)
   p=subprocess.run(argv,cwd=c.get('cwd') or str(CHANGE),stdout=subprocess.PIPE,stderr=subprocess.PIPE)
   idx+=1; stem=f'{idx:04d}-{group}-{c.get("label","class-source")}'.replace('/','_').replace(' ','_')
   op=out/(stem+'.stdout'); ep=out/(stem+'.stderr');op.write_bytes(p.stdout);ep.write_bytes(p.stderr)
   oldout=get_expected(CAND/rel,c);olderr=get_expected_err(c)
   oldoutb=oldout.read_bytes() if oldout.is_file() else None;olderrb=olderr.read_bytes() if olderr.is_file() else None
   rawout=oldoutb==p.stdout; rawerr=olderrb==p.stderr
   jsem=None; old_sem_sha=None; new_sem_sha=None
   if '--format' in argv and argv[argv.index('--format')+1]=='json' and oldoutb is not None:
    ob=norm_json_bytes(oldoutb); nb=norm_json_bytes(p.stdout)
    if ob is not None and nb is not None:
     old_sem_sha=hashlib.sha256(ob).hexdigest();new_sem_sha=hashlib.sha256(nb).hexdigest();jsem=ob==nb
   records.append({'index':idx,'group':group,'label':c.get('label'),'argv':argv,'cwd':c.get('cwd'),'returncode':p.returncode,'input_original':str(src_input),'input_used':str(input_actual),'input_sha256':observed_input_sha,'expected_input_sha256':expected_input_sha,'expected_stdout_path':str(oldout),'expected_stdout_sha256':sha(oldout) if oldout.is_file() else None,'actual_stdout_path':str(op),'actual_stdout_sha256':sha(op),'stdout_byte_equal':rawout,'expected_stderr_path':str(olderr),'expected_stderr_sha256':sha(olderr) if olderr.is_file() else None,'actual_stderr_path':str(ep),'actual_stderr_sha256':sha(ep),'stderr_byte_equal':rawerr,'json_elapsed_masked_equal':jsem,'json_expected_semantic_sha256':old_sem_sha,'json_actual_semantic_sha256':new_sem_sha})
   if p.returncode: print(f'nonzero CLI9 {idx}: {p.returncode}',file=sys.stderr)
 manifest={'cli8':str(CLI8),'cli8_sha256':sha(CLI8),'cli9':str(CLI9),'cli9_sha256':sha(CLI9),'groups':{},'records':records,'skipped':skips,'normalization':'Only for an additional JSON comparison, recursively remove dictionary key elapsed_millis exactly when immediate parent key is usage. Raw stdout/stderr hashes and byte comparisons are always retained.','complete_call_count':len(records),'skipped_call_count':len(skips)}
 for g,_,_ in SOURCES: manifest['groups'][g]={'run':sum(r['group']==g for r in records),'skipped':sum(s['group']==g for s in skips)}
 (out/'manifest.json').write_text(json.dumps(manifest,indent=2,ensure_ascii=False)+'\n')
 print(json.dumps({'output':str(out),'run':len(records),'skipped':len(skips),'groups':manifest['groups'],'stdout_equal':sum(r['stdout_byte_equal'] for r in records),'stderr_equal':sum(r['stderr_byte_equal'] for r in records),'json_semantic_equal':sum(r['json_elapsed_masked_equal'] is True for r in records),'json_semantic_compared':sum(r['json_elapsed_masked_equal'] is not None for r in records)},ensure_ascii=False))
if __name__=='__main__':main()
