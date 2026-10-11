import json,hashlib,importlib.util,re,shutil,sys
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde'); P=Path('/private/tmp/jarde-hoisted-iterator-focused-root-v2'); I=Path('/private/tmp/jarde-cf11-invocations-root-v1/hoisted-iterator-focused-root-v2')
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
def streams(rows):
 out=[]
 for row in rows:
  for key,pin in row['streams'].items():
   f=Path(pin['path']);assert f.is_file() and not f.is_symlink() and f.stat().st_size==pin['bytes'] and H(f)==pin['sha256'];out.append(pin)
 return out
d=json.loads((P/'execution.json').read_bytes());i=json.loads((I/'execution.json').read_bytes());assert H(P/'execution.json')=='69206e83ccb235ce5318b161c8398fc70e3da6ddb40b377217415af0a9e90092'
assert d['schema']=='hoisted-iterator-focused-root-v2' and d['status']=='passed_focused_candidate_only' and i['exit_code']==0
assert d['rust_min_stack'] is None and d['actual_env_overrides']['CARGO_BUILD_JOBS']=='1' and d['actual_env_overrides']['CARGO_INCREMENTAL']=='0'
assert d['runner_sha256_before']==d['runner_sha256_after']==H('/private/tmp/jarde-hoisted-iterator-focused-root-v2.py')
assert len(d['commands'])==8 and len(streams(d['commands']))==16;streams([i])
expected=[['cargo','fmt','--all'],['cargo','test','--test','p3_hoisted_iterator_foreach','--locked','--','--nocapture'],['cargo','test','--test','p3_iterable_foreach','--locked','--','--nocapture'],['cargo','test','--test','p3_array_foreach','--locked','--','--nocapture'],['cargo','test','--test','p3_wrapped_array_foreach','--locked','--','--nocapture'],['cargo','test','-p','jarde-java','--lib','--locked'],['cargo','build','-p','jarde-cli','--bin','jarde-cli','--locked'],['cargo','clean']]
counts=[2,5,6,4,360];named=[]
for n,c in enumerate(d['commands']):
 assert c['index']==n and c['argv']==expected[n] and c['cwd']==str(R) and c['exit_code']==0 and c['guard_stop'] is None and c['peak_target_bytes']<1024**3 and c['free_bytes_after']>=5*1024**3
 if 1<=n<=5:
  raw=Path(c['streams']['stdout']['path']).read_text();outcomes=re.findall(r'^test (.+) \.\.\. (ok|FAILED|ignored)$',raw,re.M);assert len(outcomes)==counts[n-1] and all(s=='ok' for _,s in outcomes) and len({name for name,_ in outcomes})==len(outcomes)
  assert c['observed_test_summary']==dict(passed=counts[n-1],failed=0,ignored=0);named.append({'command_index':n,'named_outcomes':outcomes})
assert d['target_absent'] and not (R/'target').exists() and not d['formatter_changed_paths']
assert d['source_pins_before']==d['source_pins_after'] and len(d['source_pins_after'])==4729
for f,h in d['source_pins_after'].items():assert H(R/f)==h,f
cli=d['candidate_cli'];assert cli==dict(path='/private/tmp/jarde-hoisted-iterator-candidate-cli-root-v2',bytes=61314560,sha256='8bdb868ab58e4c9196970438c71de529996c66ba826c84fe216794d328fdfc9a',mode='0o555')
f=Path(cli['path']);assert f.stat().st_size==cli['bytes'] and H(f)==cli['sha256'] and (f.stat().st_mode&0o777)==0o555
out=R/'openspec/changes/recover-proved-hoisted-iterator-foreach/results/focused-root-v2';assert not out.exists();out.mkdir()
shutil.copytree(P,out/'actual');shutil.copytree(I,out/'invocation');shutil.copyfile('/private/tmp/jarde-hoisted-iterator-focused-root-v2.py',out/'runner.py');shutil.copyfile(__file__,out/'acceptor.py')
accept={'status':'root_accepted_focused_candidate_only','receipt_sha256':H(P/'execution.json'),'invocation_sha256':H(I/'execution.json'),'source_count':4729,'commands':8,'raw_streams':16,'named_outcomes':named,'target_absent':True,'candidate_cli':cli,'peak_target_bytes':max(c['peak_target_bytes'] for c in d['commands'])}
(out/'acceptance.json').write_text(json.dumps(accept,indent=2)+'\n');(out/'README.md').write_text('实际 focused candidate 验收：8 命令/16 原始流；新增 API 2、既有 foreach 5/6/4、Java lib 360 均按 named outcome 核对。默认 Rust 栈；5 GiB free/target 1 GiB 守卫未触发、最终 cargo clean。4729 输入完整闭包在构建前后及验收时相同。尚不代表 whole-class 对照、全 workspace、Clippy 或自身 CI 验收。首轮借用编译错误完整保留于 first-focused-root-v1。\n')
print(json.dumps({k:v for k,v in accept.items() if k!='named_outcomes'}))
