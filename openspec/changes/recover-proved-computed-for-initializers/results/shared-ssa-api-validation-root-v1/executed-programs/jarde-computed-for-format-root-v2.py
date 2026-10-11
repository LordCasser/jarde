from pathlib import Path
import importlib.util,sys,os,json
sys.dont_write_bytecode=True
root=Path('/Users/lordcasser/workspace/projects/jarde');out=Path('/private/tmp/jarde-computed-for-format-root-v2');out.mkdir()
p=root/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py';s=importlib.util.spec_from_file_location('g',p);g=importlib.util.module_from_spec(s);s.loader.exec_module(g);g.OUT=out
p=root/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py';s=importlib.util.spec_from_file_location('sc',p);sc=importlib.util.module_from_spec(s);s.loader.exec_module(sc);g.target_bytes=lambda:sc.target_bytes(root);g.expected_test_summaries=lambda i,raw:None
env=os.environ.copy()
for k in g.STRIPPED_ENV_KEYS:env.pop(k,None)
env.update(g.ENV_VALUES)
r=g.run_command(100,['cargo','fmt','--all'],env);(out/'execution.json').write_text(json.dumps(r,indent=2)+'\n');assert r['exit_code']==0 and r['guard_stop'] is None
