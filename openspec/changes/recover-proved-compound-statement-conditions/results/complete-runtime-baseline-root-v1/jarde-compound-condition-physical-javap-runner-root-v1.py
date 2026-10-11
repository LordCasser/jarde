import importlib.util, pathlib, json, os, hashlib
src=pathlib.Path('/private/tmp/jarde-compound-condition-runtime-compare-root-v3.py')
spec=importlib.util.spec_from_file_location('compound_runtime_guard',src);g=importlib.util.module_from_spec(spec);spec.loader.exec_module(g)
out=pathlib.Path('/private/tmp/jarde-compound-condition-physical-javap-root-v1');out.mkdir(exist_ok=False)
plan=json.loads(g.PLAN.read_text());tool=plan['shared_pins']['jdk23_tools']['javap'];g.verify_pin(tool)
cls=pathlib.Path('/private/tmp/jarde-compound-condition-old-candidate-runtime-root-v2/original-bytecode/CompoundStatementConditions.class')
g.OUT=out;g.RECORD={'schema':'compound-condition-physical-javap-root-v1','pre_pins':[g.pin(src),g.pin(cls),tool],'commands':[]}
env=os.environ.copy()
for k in g.STRIPPED_ENV:env.pop(k,None)
r=g.run(out,'physical-javap-v-p-c',[tool['path'],'-v','-p','-c',cls],env)
assert r['exit_code']==0 and r['guard_stop'] is None
for x in g.RECORD['pre_pins']:g.verify_pin(x)
g.RECORD['post_pin_check']='passed';g.write_record();print(g.RECORD['pre_pins'])
