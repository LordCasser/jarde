import hashlib, json, os, pathlib, subprocess, tarfile, time, sys
root=pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
gates=root/'openspec/changes/recover-same-class-generic-call-consumers/results/local-gates'
version=sys.argv[1]
mode=sys.argv[2]
def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def write(path, value):
    with path.open('x') as f: json.dump(value, f, indent=2); f.write('\n')
env_extra={'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','RUST_TEST_THREADS':'1'}
env=os.environ.copy(); env.update(env_extra)
snapshot_name='source-snapshot-'+version+'.json'
snapshot_path=gates/snapshot_name
if mode=='preflight':
    prepare_label='fmt-'+version+'-prepare'
    with (gates/(prepare_label+'.log')).open('xb') as out:
        result=subprocess.run(['cargo','fmt','--all'], cwd=root, env=env, stdout=out, stderr=subprocess.STDOUT)
    write(gates/(prepare_label+'-result.json'), {'argv':['cargo','fmt','--all'],'exit':result.returncode,'log_sha256':digest(gates/(prepare_label+'.log'))})
    if result.returncode: sys.exit(result.returncode)
    previous=json.loads((gates/'source-snapshot-v33.json').read_text())
    files=list(previous['files'])+['tests/generic_method_projection.rs','tests/generic_throws_projection.rs']
    archive=gates/('source-snapshot-'+version+'.tar.gz')
    if archive.exists(): raise RuntimeError('refuse overwrite snapshot')
    with tarfile.open(archive, 'w:gz') as tar:
        for file in files: tar.add(root/file, arcname=file)
    head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
    write(snapshot_path, {'archive':archive.name,'archive_sha256':digest(archive),'files':{file:digest(root/file) for file in files},'temporary_trace':False,'git_head':head})
snapshot=json.loads(snapshot_path.read_text())
def unchanged(): return all(digest(root/p)==h for p,h in snapshot['files'].items())
if not unchanged(): raise RuntimeError('source snapshot mismatch')
if mode=='preflight':
    clippy=json.loads((gates/'clippy-v33-command.json').read_text())['argv']
    commands=[('fmt',['cargo','fmt','--all','--','--check'],{}),('diff',['git','diff','--check'],{}),('regressions',['cargo','test','--test','preserve_local_scope_refusals','--test','generic_method_projection','--test','generic_throws_projection','--test','ordinary_generic_projection','--test','generic_constructor_projection','--test','same_class_generic_binding','--test','same_class_generic_call_consumers','--test','class_scope_constructor_projection','--locked'],{}),('initializer-regression',['cargo','test','-p','jarde-java','--test','class_initializer_candidates','--locked'],{}),('graph',['cargo','test','--lib','generic_call_components','--locked'],{}),('clippy',clippy,{}),('candidate-cli-v8-build',['cargo','build','-p','jarde-cli','--locked'],{})]
else:
    commands=[('seed-'+seed,['cargo','test','--workspace','--all-targets','--all-features','--locked','--no-fail-fast'],{'PROPTEST_RNG_SEED':seed}) for seed in ['5350648285461741569','5350648285461741570']]
    commands += [('ignored-p3',['cargo','test','--test','p3_execution_comparison','--locked','--','--ignored'],{}),('ignored-constructor',['cargo','test','-p','jarde-cli','--test','json_cli','--locked','--','--ignored','--exact','functional_constructor_arguments_replay_the_complete_class_on_both_jdks'],{}),('ignored-bound-receiver',['cargo','test','--test','recover_proved_nonnull_bound_receivers','--locked','--','--ignored','--exact','every_stripped_anchor_answers_what_its_class_answers'],{}),('openspec-strict',['openspec','validate','--all','--strict','--no-interactive'],{})]
index=[]
index_path=gates/(('preflight-gates-' if mode=='preflight' else 'final-gates-')+version+'-index.json')
if index_path.exists(): raise RuntimeError('refuse overwrite gate index')
for name,argv,extra in commands:
    label=name+'-'+version
    spec={'label':label,'argv':argv,'cwd':str(root),'environment':dict(env_extra,**extra),'source_snapshot':snapshot_name,'started_unix_ns':time.time_ns()}
    write(gates/(label+'-command.json'),spec)
    actual_env=env.copy();actual_env.update(extra)
    print('RUN '+label,flush=True)
    with (gates/(label+'.log')).open('xb') as out:
        result=subprocess.run(argv,cwd=root,env=actual_env,stdout=out,stderr=subprocess.STDOUT)
    spec.update({'exit':result.returncode,'finished_unix_ns':time.time_ns(),'log_sha256':digest(gates/(label+'.log')),'source_unchanged':unchanged()})
    write(gates/(label+'-result.json'),spec)
    with (gates/(label+'.exit')).open('x') as out: out.write(str(result.returncode)+'\n')
    index.append(spec)
    index_path.write_text(json.dumps(index,indent=2)+'\n')
    print('DONE '+label+' exit='+str(result.returncode)+' unchanged='+str(spec['source_unchanged']),flush=True)
    if result.returncode or not spec['source_unchanged']: sys.exit(result.returncode or 2)
print('ALL PASSED',flush=True)
