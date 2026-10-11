import ast, hashlib, json, os, pathlib, shutil, subprocess, time
R=pathlib.Path('/Users/lordcasser/workspace/projects/jarde');D=pathlib.Path('/private/tmp/jarde-computed-for-controls-luna-v1')
O=pathlib.Path('/private/tmp/jarde-computed-for-controls-root-v1');O.mkdir(exist_ok=False)
H=lambda p:hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
M=json.loads((R/'openspec/changes/preserve-proved-if-arm-join-origins/results/cf07-candidate-root-v1/manifest.json').read_text())
CLI=pathlib.Path('/private/tmp/jarde-arithmetic-stack-candidate-cli-root-v2');assert H(CLI)=='b081ca53080d0a41f00bf8f5b924499a2c781f413746fc9facb7862ef8b9cfe0'
record={'status':'running','scope':'computed-for new control baseline only; no new product recovery','runner_sha256':H(__file__),'input_pins':[],'commands':[],'profiles':[]}
def pin(p):record['input_pins'].append({'path':str(p),'sha256':H(p)})
def save():(O/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
def run(label,argv,env=None):
    t=time.monotonic();p=subprocess.run(list(map(str,argv)),capture_output=True,timeout=60,env=env)
    for stream,data in [('stdout',p.stdout),('stderr',p.stderr)]: (O/(label+'.'+stream+'.raw')).write_bytes(data)
    row={'label':label,'argv':list(map(str,argv)),'exit_code':p.returncode,'duration_seconds':time.monotonic()-t,'stdout_sha256':hashlib.sha256(p.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(p.stderr).hexdigest()}
    record['commands'].append(row);save();return p
for path in [D/'ComputedForControls.java',D/'ComputedForControlsRunner.java',D/'patch_pending_stack.py',CLI]:pin(path)
env=os.environ.copy()
for key in ('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH'):env.pop(key,None)
try:
    for profile,leg in M['jdk_legs'].items():
        for tool in ('javac','java','javap'):pin(leg['tools'][tool]['path']);assert H(leg['tools'][tool]['path'])==leg['tools'][tool]['sha256']
        base=O/profile;base.mkdir();original=base/'original';original.mkdir();pending=base/'pending'
        flags=['-source','8','-target','8'] if profile=='javac8' else ['--release','8']
        p=run(profile+'-compile',[leg['tools']['javac']['path'],*flags,'-g:none','-proc:none','-d',original,D/'ComputedForControls.java',D/'ComputedForControlsRunner.java'],env);assert p.returncode==0
        shutil.copytree(original,pending)
        p=run(profile+'-patch',['python3',D/'patch_pending_stack.py',original/'ComputedForControls.class',pending/'ComputedForControls.class']);assert p.returncode==0
        patches=ast.literal_eval(p.stdout.decode())['patched_methods'];assert len(patches)==2
        before=(original/'ComputedForControls.class').read_bytes();after=(pending/'ComputedForControls.class').read_bytes();assert len(before)==len(after) and before!=after
        outputs={}
        for side,classes in [('original',original),('pending',pending)]:
            p=run(profile+'-'+side+'-javap',[leg['tools']['javap']['path'],'-v','-c','-p','-classpath',classes,'ComputedForControls'],env);assert p.returncode==0
            p=run(profile+'-'+side+'-runtime',[leg['tools']['java']['path'],'-Xverify:all','-cp',classes,'ComputedForControlsRunner'],env);assert p.returncode==0 and p.stderr==b'';outputs[side]=p.stdout
            for evidence in ('default','all'):
                prefix=profile+'-'+side+'-'+evidence
                argv=[CLI,'class-source','--input',classes/'ComputedForControls.class','--class','ComputedForControls','--policy','single-class','--release','8','--format','json']
                if evidence=='all':argv+=['--evidence','all']
                p=run(prefix+'-render',argv);assert p.returncode==0 and not p.stderr
                doc=json.loads(p.stdout);source=base/(side+'-'+evidence+'.java');source.write_text(doc['text'])
                methods=[{'name':m['item']['name']['escaped'],'quality':m.get('outcome',{}).get('report',{}).get('quality'),'has_for':'for (' in m.get('outcome',{}).get('report',{}).get('text','')} for m in doc['methods']]
                record['profiles'].append({'profile':profile,'side':side,'evidence':evidence,'input_class_sha256':H(classes/'ComputedForControls.class'),'source_sha256':H(source),'methods':methods})
        assert outputs['original']==outputs['pending']
        (base/'expected-runtime.stdout.raw').write_bytes(outputs['original'])
        record.setdefault('runtime_pairs',[]).append({'profile':profile,'equal':True,'stdout':outputs['original'].decode(),'patches':patches,'original_class_sha256':H(original/'ComputedForControls.class'),'pending_class_sha256':H(pending/'ComputedForControls.class')})
    assert all(H(x['path'])==x['sha256'] for x in record['input_pins'])
    record['status']='passed_original_and_pending_legality_and_observation_only'
except Exception as error:
    record['status']='failed';record['error']=repr(error);raise
finally:save()
print(json.dumps({'status':record['status'],'commands':len(record['commands']),'profiles':len(record['profiles']),'runtime_pairs':len(record['runtime_pairs'])}))
