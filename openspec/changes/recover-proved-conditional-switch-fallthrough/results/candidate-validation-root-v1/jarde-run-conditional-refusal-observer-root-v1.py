from pathlib import Path
import hashlib,importlib.util,json,os,sys
root=Path('/Users/lordcasser/workspace/projects/jarde');out=Path('/private/tmp/jarde-conditional-refusal-observer-root-v1');out.mkdir()
p=root/'crates/jarde-java/src/region.rs';original=p.read_bytes();(out/'region.before.rs').write_bytes(original)
s=original.decode();a=s.index("fn prove_switch_fallthroughs<'edge");b=s.index('\n/// The postfix position',a)
helper=s[a:b];helper=helper.replace('return Ok(None)', '{ eprintln!("ROOT_SWITCH_PROOF_REFUSAL line={} caller={}", line!(), switch_bci); return Ok(None) }')
p.write_text(s[:a]+helper+s[b:]);(out/'region.observer.rs').write_bytes(p.read_bytes())
guard=root/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py';spec=importlib.util.spec_from_file_location('guard',guard);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
g=m.run_command.__globals__;g.update(ROOT=root,OUT=out,expected_test_summaries=lambda i,raw:None,command_stream=lambda p:{'path':str(p),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()})
env=os.environ.copy();env.update(m.ENV_VALUES);env['CARGO_PROFILE_TEST_STRIP']='symbols'
e={'schema':'conditional-refusal-observation-root-v1','product_accepted':False,'commands':[],'before_sha256':hashlib.sha256(original).hexdigest()}
try:
 row=m.run_command(0,['cargo','test','-p','jarde-java','--lib','--locked','cf12_switch_certificate','--','--nocapture'],env);e['commands'].append(row)
 e['status']='observed' if row['guard_stop'] is None else 'guard-stopped'
finally:
 p.write_bytes(original);e['source_restored']=p.read_bytes()==original
 (out/'execution.json').write_text(json.dumps(e,indent=2)+'\n')
print(json.dumps({k:e.get(k) for k in ['status','source_restored']}))
