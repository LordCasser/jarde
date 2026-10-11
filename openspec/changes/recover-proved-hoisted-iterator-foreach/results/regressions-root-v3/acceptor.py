import json,hashlib,re,shutil,sys
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde');P=Path('/private/tmp/jarde-hoisted-iterator-regressions-root-v3');I=Path('/private/tmp/jarde-cf11-invocations-root-v1/hoisted-iterator-regressions-root-v3');H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def streams(rows):
 count=0
 for row in rows:
  assert set(row['streams'])=={'stdout','stderr'}
  for pin in row['streams'].values():
   f=Path(pin['path']);assert f.is_file() and not f.is_symlink() and f.stat().st_size==pin['bytes'] and H(f)==pin['sha256'];count+=1
 return count
assert H(P/'execution.json')=='5a5ca9c44f10fa4f1f057b16e2efd3e40687e266eb1b8fee8f762167931a108f';d=json.loads((P/'execution.json').read_bytes());i=json.loads((I/'execution.json').read_bytes());assert d['schema']=='hoisted-iterator-regressions-root-v3' and d['status']=='passed_regressions_candidate_only' and i['exit_code']==0 and streams([i])==2
assert len(d['commands'])==8 and streams(d['commands'])==16 and d['rust_min_stack'] is None and d['runner_sha256_before']==d['runner_sha256_after']==H('/private/tmp/jarde-hoisted-iterator-regressions-root-v3.py')
counts={1:(1,0,0),2:(365,0,0),3:(2,0,0),4:(1,0,0),5:(5,0,1)};named=[]
for n,c in enumerate(d['commands']):
 assert c['index']==n and c['cwd']==str(R) and c['exit_code']==0 and c['guard_stop'] is None and c['peak_target_bytes']<1024**3 and c['free_bytes_after']>=5*1024**3
 if n in counts:
  raw=Path(c['streams']['stdout']['path']).read_text();headers=list(re.finditer(r'^test (.+?) \.\.\. ',raw,re.M));outcomes=[]
  for j,header in enumerate(headers):
   end=headers[j+1].start() if j+1<len(headers) else len(raw);body=raw[header.end():end].split('\ntest result:',1)[0].rstrip();terminal=re.search(r'(?<!\S)(ok|FAILED|ignored)(?:, [^\n]*)?\s*$',body);assert terminal,(n,header.group(1),body[-200:]);outcomes.append((header.group(1),terminal.group(1)))
  assert len(outcomes)==sum(counts[n]) and len({a for a,b in outcomes})==len(outcomes);assert tuple(sum(status==want for a,status in outcomes) for want in ['ok','FAILED','ignored'])==counts[n]
  assert c['observed_test_summary']==dict(zip(['passed','failed','ignored'],counts[n]));named.append(dict(index=n,argv=c['argv'],outcomes=outcomes))
assert d['commands'][0]['argv']==['cargo','fmt','--all'] and d['commands'][-1]['argv']==['cargo','clean'];assert d['source_pins_before']==d['source_pins_after'] and len(d['source_pins_after'])==4732
for f,h in d['source_pins_after'].items():assert H(R/f)==h,f
assert d['regenerator_changed_paths']==['tests/fixtures/corpus-fingerprint.json'] and d['target_absent'] and not (R/'target').exists()
cli=d['candidate_cli'];f=Path(cli['path']);assert H(f)==cli['sha256']=='8bdb868ab58e4c9196970438c71de529996c66ba826c84fe216794d328fdfc9a' and f.stat().st_size==cli['bytes']==61314560 and (f.stat().st_mode&0o777)==0o555 and d['same_cli_as_runtime_candidate_v2'] and H('/private/tmp/jarde-hoisted-iterator-candidate-cli-root-v2')==H(f)
lib=Path(d['commands'][2]['streams']['stderr']['path']).read_text();api=Path(d['commands'][3]['streams']['stderr']['path']).read_text()
for marker in ['[iterator-owner-stop] usage=1 stop=Budget { dimension: AnalysisSteps, written: 0, limit: 1, at: Some(35) }','[iterator-phi-stop] usage=1 stop=Budget { dimension: AnalysisSteps, written: 0, limit: 1, at: Some(35) }','source_loads=[11]','block: CanonicalBlockId { bci: 18, path: [] }, slot: Local(3)']:assert marker in lib,marker
match=re.search(r'\[hoisted-iterator-api-stop\] limit=(\d+) usage=(\d+) stop=Budget \{ dimension: AnalysisSteps, written: (\d+), limit: (\d+), at: Some\((\d+)\) \}',api);assert match
limit,usage,written,limit2,bci=map(int,match.groups());assert limit==usage==limit2 and written>0 and bci==6
out=R/'openspec/changes/recover-proved-hoisted-iterator-foreach/results/regressions-root-v3';assert not out.exists();out.mkdir();shutil.copytree(P,out/'actual');shutil.copytree(I,out/'invocation');shutil.copyfile('/private/tmp/jarde-hoisted-iterator-regressions-root-v3.py',out/'runner.py');shutil.copyfile(__file__,out/'acceptor.py')
result={'status':'root_accepted_permanent_regressions_only','receipt_sha256':H(P/'execution.json'),'source_files':4732,'target_absent':True,'named_outcomes':named,'actual_helper_stops':[dict(gate=g,dimension='AnalysisSteps',usage=1,limit=1,at=35,written=0) for g in ['declaration','phi']],'actual_api_stop':dict(dimension='AnalysisSteps',limit=limit,usage=usage,at=bci,written_before_refusal=written,published_text_bytes=0,published_source_map_segments=0),'candidate_cli':cli,'same_cli_as_runtime_candidate_v2':True,'peak_target_bytes':max(c['peak_target_bytes'] for c in d['commands'])}
(out/'acceptance.json').write_text(json.dumps(result,indent=2)+'\n');(out/'README.md').write_text('实际永久回归根验收：2026-10-11 20:01:39–20:03:23 UTC，8cmd/16raw；Java lib365/0、API2/0、reader census1/0、p5常规5/0/1和regenerate1/0。guard/defaultstack/final clean，4732输入闭包前后/live同。canonical fixture带真实mixed/live Phi与alias额外Load11，三个owner/split/Phi旧fixture测试通过；v3别名假设静态拒绝未实跑，应用v4准确负例。\n\n两个生产helper实际AnalysisSteps usage1/limit1/BCI35、written0；公开API近complete预算实际limit8230/usage8230/BCI6、written2273，但该method没有发布text/source-map/content/news，pre-cancel计数零。公开Stop是实际后期发射/验证计费停止，不冒称该公开停止发生在新的owner helper。\n\n新CLI与已双JDK完整类对照的v2字节相同，source closure因永久tests/corpus变为4732。尚不代表fresh全workspace/Clippy/strict/本片自身CI。\n')
print(json.dumps({k:v for k,v in result.items() if k!='named_outcomes'}))
