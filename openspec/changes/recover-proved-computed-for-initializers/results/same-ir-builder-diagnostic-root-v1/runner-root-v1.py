import hashlib, importlib.util, json, os, pathlib, re, subprocess, sys
sys.dont_write_bytecode=True
R=pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
O=pathlib.Path('/private/tmp/jarde-computed-for-builder-diagnostic-root-v1');O.mkdir(exist_ok=False)
D=pathlib.Path('/private/tmp/jarde-computed-for-builder-diagnostic-luna-v2')
H=lambda p:hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
spec=importlib.util.spec_from_file_location('focused_diag', '/private/tmp/jarde-arithmetic-stack-focused-and-cli-root-v2.py');focused=importlib.util.module_from_spec(spec);spec.loader.exec_module(focused)
assert H(focused.GUARD)==focused.GUARD_SHA256 and H(focused.SCAN)==focused.SCAN_SHA256
g=focused.load_module('guard_diag',focused.GUARD);scan=focused.load_module('scan_diag',focused.SCAN)
g.ROOT=R;g.RESULTS=O.parent;g.OUT=O;g.target_bytes=lambda:scan.target_bytes(R)
g.command_stream=lambda path:{'path':str(path),'bytes':pathlib.Path(path).stat().st_size,'sha256':H(path)}
g.expected_test_summaries=lambda index,stdout:None
env=os.environ.copy();env.update(g.ENV_VALUES);env['PROPTEST_RNG_SEED']='5350648285461741569'
for key in g.STRIPPED_ENV_KEYS:env.pop(key,None)
region=R/'crates/jarde-java/src/region.rs';before=region.read_bytes()
build=R/'crates/jarde-java/src/build.rs';build_before=build.read_bytes()
assert before==(D/'base-region.rs').read_bytes() and build_before==(D/'base-build.rs').read_bytes() and not (R/'target').exists()
paths=focused.compound_paths(g);pins=g.source_pins(paths)
record={'status':'running','scope':'temporary same-real-IR SSA/CFG and Builder init/materialization diagnostic; no product implementation','runner_sha256':H(__file__),'base_build_sha256':H(build),'candidate_build_sha256':H(D/'candidate-build.rs'),'base_region_sha256':H(region),'candidate_region_sha256':H(D/'candidate-region.rs'),'patch_sha256':H(D/'combined.patch'),'source_pins_before':pins,'guards':{'free_bytes':g.FREE_LIMIT,'target_bytes':g.TARGET_LIMIT},'commands':[]}
def save(): (O/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
save();applied=False
try:
    for i,argv in enumerate([['git','apply','--check',str(D/'combined.patch')],['git','apply',str(D/'combined.patch')]]):
        p=subprocess.run(argv,cwd=R,capture_output=True,timeout=30)
        (O/('apply-'+str(i)+'.stdout.raw')).write_bytes(p.stdout);(O/('apply-'+str(i)+'.stderr.raw')).write_bytes(p.stderr)
        record['commands'].append({'argv':argv,'exit_code':p.returncode,'stdout_sha256':hashlib.sha256(p.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(p.stderr).hexdigest()});save();assert p.returncode==0
        if i==1:applied=True
    assert H(region)==record['candidate_region_sha256'] and H(build)==record['candidate_build_sha256']
    record['diagnostic_source_pins']=g.source_pins(paths);save()
    row=g.run_command(0,['cargo','test','-p','jarde-java','--lib','--locked','region::tests::cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir','--','--exact','--nocapture'],env)
    record['commands'].append(row);save()
    assert row['exit_code']==0 and row['guard_stop'] is None
    raw=(O/'0.stdout.raw').read_bytes();stderr=(O/'0.stderr.raw').read_bytes()
    assert b'test result: ok. 1 passed; 0 failed; 0 ignored;' in raw
    markers=['CF07_INIT_INSN','CF07_INIT_VALUE','CF07_RELEVANT_PHI','CF07_SLOT4_ACCESS','CF07_NATURAL_LOOP','CF07_RETURN_LEAVES','CF07_UPDATE_LATCH','CF07_UPDATE_EDGE','CF07_BUILDER_PROGRAM','CF07_BUILDER_INIT_STMT','CF07_BUILDER_ALL_STMTS','CF07_BUILDER_FRAME','CF07_BUILDER_MATERIALIZATION']
    record['diagnostic_marker_counts']={m:stderr.count(m.encode()) for m in markers};assert all(record['diagnostic_marker_counts'].values())
    record['status']='passed_same_IR_SSA_and_Builder_diagnostic_only'
except Exception as error:
    record['status']='failed';record['error']=repr(error);raise
finally:
    if applied:
        assert H(region)==record['candidate_region_sha256'],'unexpected concurrent region change; do not overwrite'
        assert H(build)==record['candidate_build_sha256'],'unexpected concurrent build change; do not overwrite'
        region.write_bytes(before);build.write_bytes(build_before)
    record['source_pins_after_restore']=g.source_pins(paths);record['restored']=record['source_pins_after_restore']==pins
    record['clean_command']=g.run_command(1,['cargo','clean'],env)
    record['target_absent']=not (R/'target').exists();save()
assert record['restored'] and record['target_absent'] and record['clean_command']['exit_code']==0
print(json.dumps({'status':record['status'],'restored':record['restored'],'target_absent':record['target_absent'],'markers':record['diagnostic_marker_counts']}))
