#!/usr/bin/env python3
"""只读核验实际raw、完整方法与物理来源；不运行工具链。"""
import argparse,hashlib,importlib.util,json,re
from pathlib import Path

def load(p,n):
 s=importlib.util.spec_from_file_location(n,p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m

def main():
 ap=argparse.ArgumentParser();ap.add_argument('out',type=Path);ap.add_argument('--mode',choices=['baseline','final'],required=True);ap.add_argument('--baseline',type=Path);a=ap.parse_args();o=a.out.resolve();x=json.loads((o/'execution.json').read_text());assert x['status']=='complete_pending_review' and x['mode']==a.mode
 c=load(Path(__file__).with_name('compare.py'),'cmp');rows=x['commands'];assert [v['index'] for v in rows]==list(range(1,len(rows)+1));raw={}
 for v in rows:
  assert v['guard_stop'] is None and v['peak_target_bytes']<=1024**3 and v['free_bytes_after']>=5*1024**3
  for channel in ('stdout','stderr'):raw[v['index'],channel]=c.command_artifact(v,channel)
 def cmd(n):return rows[n-1]
 assert len(x['original_oracles'])==4
 oracle={}
 for v in x['original_oracles']:
  assert cmd(v['command'])['exit_code']==0;c.valid_nine_stdout(raw[v['command'],'stdout'],'original');oracle[v['input_compiler'],v['runtime']]=raw[v['command'],'stdout'];assert not raw[v['command'],'stderr']
 assert len(set(oracle.values()))==1
 count=0;digests={};alltexts={};targets={};baseline_reports={};fingerprint=None
 if a.mode=='final':
  assert a.baseline is not None,'final acceptance requires the actual pre-change baseline'
  baseline=json.loads((a.baseline/'execution.json').read_text());assert baseline['mode']=='baseline' and baseline['status']=='complete_pending_review'
  baseline_rows={r['index']:r for r in baseline['commands']}
  for cell in baseline['jarde_cells']:
   doc=json.loads(c.command_artifact(baseline_rows[cell['command']],'stdout'))
   baseline_reports[cell['label'],cell['evidence']]={(m['item']['name']['escaped'],m['item']['descriptor']['escaped']):m['outcome']['report'] for m in doc['methods']}
  fingerprint_path=Path(__file__).resolve().parents[4]/'tests/fixtures/corpus-fingerprint.json'
  fingerprint={v['path']:v for v in json.loads(fingerprint_path.read_text())['files']}
 for cell in x['jarde_cells']:
  n=cell['command'];args=cmd(n)['argv'];assert cmd(n)['exit_code']==0 and args[args.index('--input')+1]==cell['input_class']['path']
  p=cell['input_class'];b=Path(p['path']).read_bytes();assert len(b)==p['bytes'] and hashlib.sha256(b).hexdigest()==p['sha256']
  doc=json.loads(raw[n,'stdout']);assert Path(cell['json_artifact']['path']).read_bytes()==raw[n,'stdout'];assert Path(cell['source_artifact']['path']).read_bytes()==doc['text'].encode()
  facts=c.parse_class(Path(p['path']));members=doc['methods'];assert len(members)==len(facts['methods']);cb=doc['class']['class_bytes'];assert cb['length']==len(b);digests[cell['label'],cell['evidence']]=cb
  if fingerprint is not None:
   relative=str(Path(p['path']).relative_to(Path(__file__).resolve().parents[4]));fp=fingerprint[relative];assert fp['bytes']==len(b) and cb['digest']==fp['blake3']
  is_sealed=cell['label'].startswith('sealed');target_found=False
  for physical,m in zip(facts['methods'],members):
   item=m['item'];name=item['name']['escaped'];desc=item['descriptor']['escaped'];key=(name,desc);assert key==(physical['name'],physical['descriptor']) and item['index']==physical['index']
   identity=item['identity'];assert bytes(identity['name']).decode()==name and bytes(identity['descriptor']).decode()==desc and identity['owner']['class_bytes']==cb
   assert m['outcome']['kind']=='recovered';report=m['outcome']['report'];assert report['method']==name+desc
   expected_fallback=(name in ('finallyReturns','finallyThrows') and is_sealed) or (name=='tryThrowsFinallyRuns' and a.mode=='baseline')
   assert (report['quality']=='fallback')==expected_fallback
   text=report['text'].encode();covered=set()
   for seg in report['source_map']['segments']:
    lo,hi=seg['start'],seg['end'];assert 0<=lo<hi<=len(text);text[:lo].decode();text[:hi].decode()
    for origin in [seg['origin']['primary'],*seg['origin']['derived']]:
     if origin.get('bci') is not None:assert origin['method']==identity;covered.add(origin['bci'])
   assert covered==set(physical['code']['bcis']),(cell['label'],name,covered,physical['code']['bcis'])
   if name=='tryThrowsFinallyRuns':
    target_found=True;assert physical['code']['bcis']==[0,2,5,6,9,10,11,13,16,17,18]
    if a.mode=='final':assert report['representation']=='java' and report['fallbacks']==[] and '@bytecode' not in report['text'] and report['text'].count('finally {')==1
   if a.mode=='final' and name!='tryThrowsFinallyRuns':
    prior=baseline_reports[cell['label'],cell['evidence']][key]
    for field in ('text','source_map','quality','representation','content','fallbacks'):assert report[field]==prior[field],(cell['label'],name,field,'non-target changed')
   targets[cell['label'],cell['evidence'],name+desc]=report['text'];count+=1
  assert target_found;alltexts[cell['label'],cell['evidence']]=doc['text']
  for comp in cell['compiles']:
   n=comp['command'];args=cmd(n)['argv'];assert args[-1]==cell['source_artifact']['path'] and comp['exit_code']==cmd(n)['exit_code']
   if is_sealed:
    assert cmd(n)['exit_code']==1 and comp['compiled_class'] is None
    num,diags=c.missing_return_count(raw[n,'stderr'],Path(cell['source_artifact']['path']));expected={'finallyReturns','finallyThrows'}|({'tryThrowsFinallyRuns'} if a.mode=='baseline' else set());assert num==len(expected) and {d['method'] for d in diags}==expected
   elif a.mode=='baseline':
    assert cmd(n)['exit_code']==1 and comp['class'] is None and comp['runs']==[]
    num,diags=c.missing_return_count(raw[n,'stderr'],Path(cell['source_artifact']['path']));assert num==1 and diags[0]['method']=='tryThrowsFinallyRuns'
   else:
    assert cmd(n)['exit_code']==0 and comp['class'];assert len(comp['runs'])==2
    for run in comp['runs']:
     n=run['command'];assert cmd(n)['exit_code']==0;c.valid_nine_stdout(raw[n,'stdout'],'rebuilt');leg=cell['label'].removeprefix('exception-only-');assert raw[n,'stdout']==oracle[leg,run['runtime']] and not raw[n,'stderr']
 for label in ('exception-only-javac8','exception-only-javac23','sealed-javac8','sealed-javac23'):
  assert alltexts[label,'default']==alltexts[label,'all'] and digests[label,'default']==digests[label,'all']
 assert alltexts['exception-only-javac8','default']==alltexts['exception-only-javac23','default']
 jadx_compiles=[];jadx_runs=[]
 for v in x['jadx_runs']:
  for comp in v['compiles']:
   assert cmd(comp['command'])['exit_code']==comp['exit_code'];jadx_compiles.append(comp['exit_code'])
   for run in comp['runs']:jadx_runs.append({'exit_code':cmd(run['command'])['exit_code'],'stderr':raw[run['command'],'stderr'].decode(errors='replace')[:240]})
 assert x['inputs_and_tools_unchanged'] and x['source_pins_unchanged'] and x['repository_target_absent_after']
 print(json.dumps({'status':'root accepted','mode':a.mode,'commands':len(rows),'raw_streams_verified':len(raw),'method_profiles_verified':count,'class_profiles':len(alltexts),'physical_method_maps_BCIs_identity_UTF8':True,'original_runs':4,'original_cases':36,'Jarde_rebuilt_runs':0 if a.mode=='baseline' else 16,'JADX_compile_exits':jadx_compiles,'JADX_runtime_observations':jadx_runs,'class_identity_binding':'independent standard corpus BLAKE3 plus physical input SHA' if fingerprint is not None else 'physical command input SHA and coherent raw report identity; final corpus digest gate separate','non_target_unchanged_from_actual_baseline':a.mode=='final'},ensure_ascii=False,indent=2))
if __name__=='__main__':main()
