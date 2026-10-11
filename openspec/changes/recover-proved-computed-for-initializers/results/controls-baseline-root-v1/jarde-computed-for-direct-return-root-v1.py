import hashlib,json,os,pathlib,subprocess,time
R=pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
D=pathlib.Path('/private/tmp/jarde-computed-for-controls-independent-luna-v1/v2')
O=pathlib.Path('/private/tmp/jarde-computed-for-direct-return-root-v1');O.mkdir(exist_ok=False)
H=lambda p:hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
M=json.loads((R/'openspec/changes/preserve-proved-if-arm-join-origins/results/cf07-candidate-root-v1/manifest.json').read_text())
CLI=pathlib.Path('/private/tmp/jarde-arithmetic-stack-candidate-cli-root-v2')
assert H(CLI)=='b081ca53080d0a41f00bf8f5b924499a2c781f413746fc9facb7862ef8b9cfe0'
record={'status':'running','scope':'minimal call-init original legality and current CLI baseline only','runner_sha256':H(__file__),'input_pins':[],'commands':[],'profiles':[]}
def pin(p):record['input_pins'].append({'path':str(p),'sha256':H(p)})
def save():(O/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
def run(label,argv,env):
    t=time.monotonic();a=list(map(str,argv));p=subprocess.run(a,capture_output=True,timeout=60,env=env)
    for stream,data in [('stdout',p.stdout),('stderr',p.stderr)]: (O/(label+'.'+stream+'.raw')).write_bytes(data)
    record['commands'].append({'label':label,'argv':a,'exit_code':p.returncode,'duration_seconds':time.monotonic()-t,'stdout_sha256':hashlib.sha256(p.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(p.stderr).hexdigest()});save();return p
for f in [D/'ComputedForDirectReturn.java',D/'ComputedForDirectReturnRunner.java',CLI]:pin(f)
env=os.environ.copy()
for key in ('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH'):env.pop(key,None)
expected=b'hit:2:1\nbounded-hit:4:1\nmiss:-1:1\nempty-range:-1:1\n'
try:
    for profile,leg in M['jdk_legs'].items():
        for tool in ('javac','java','javap'):pin(leg['tools'][tool]['path']);assert H(leg['tools'][tool]['path'])==leg['tools'][tool]['sha256']
        base=O/profile;base.mkdir();classes=base/'classes';classes.mkdir()
        flags=['-source','8','-target','8'] if profile=='javac8' else ['--release','8']
        p=run(profile+'-compile',[leg['tools']['javac']['path'],*flags,'-g:none','-proc:none','-d',classes,D/'ComputedForDirectReturn.java',D/'ComputedForDirectReturnRunner.java'],env);assert p.returncode==0
        p=run(profile+'-javap',[leg['tools']['javap']['path'],'-v','-c','-p','-classpath',classes,'ComputedForDirectReturn'],env);assert p.returncode==0
        p=run(profile+'-runtime',[leg['tools']['java']['path'],'-Xverify:all','-cp',classes,'ComputedForDirectReturnRunner'],env);assert p.returncode==0 and p.stderr==b'' and p.stdout==expected
        for evidence in ('default','all'):
            argv=[CLI,'class-source','--input',classes/'ComputedForDirectReturn.class','--class','ComputedForDirectReturn','--policy','single-class','--release','8','--format','json']
            if evidence=='all':argv+=['--evidence','all']
            p=run(profile+'-'+evidence+'-render',argv,env);assert p.returncode==0 and not p.stderr
            doc=json.loads(p.stdout);source=base/(evidence+'.java');source.write_text(doc['text'])
            methods=[{'name':bytes(m['item']['identity']['name']).decode(),'quality':m.get('outcome',{}).get('report',{}).get('quality'),'has_for':'for (' in m['text']} for m in doc['methods']]
            record['profiles'].append({'profile':profile,'evidence':evidence,'input_class_sha256':H(classes/'ComputedForDirectReturn.class'),'source_sha256':H(source),'methods':methods})
    assert all(H(x['path'])==x['sha256'] for x in record['input_pins']);record['status']='passed_original_legality_current_cli_baseline_only'
except Exception as error:record['status']='failed';record['error']=repr(error);raise
finally:save()
print(json.dumps({'status':record['status'],'commands':len(record['commands']),'profiles':record['profiles']}))
