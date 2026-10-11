import hashlib, importlib.util, json, pathlib, subprocess
O=pathlib.Path('/private/tmp/jarde-cf07-cf13-current-runtime-root-v2')
P=pathlib.Path('/private/tmp/jarde-cf07-cf13-current-cli-patrol-root-v1')
R=pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
A=pathlib.Path('/private/tmp/jarde-cf07-cf13-current-runtime-independent-root-v1');A.mkdir(exist_ok=False)
H=lambda p:hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
verifier=O/'luna-audit/verify.py'
p=subprocess.run(['python3',str(verifier)],capture_output=True,timeout=30)
(A/'stdout.raw').write_bytes(p.stdout);(A/'stderr.raw').write_bytes(p.stderr)
assert p.returncode==0
ex=json.loads((O/'execution.json').read_text());patrol=json.loads((P/'execution.json').read_text())
manifest=json.loads((R/'openspec/changes/preserve-proved-if-arm-join-origins/results/cf07-candidate-root-v1/manifest.json').read_text())
expected={};generated=[]
for idx,(pkg,cls,runner) in enumerate([('cf07','LoopCases','Runner'),('cf13','SwitchLoopExits','SwitchLoopExitsRunner')]):
    original=pathlib.Path(patrol['commands'][idx]['argv'][3])
    runner_path=next(pathlib.Path(x['path']) for x in ex['inputs'] if pathlib.Path(x['path']).name==runner+'.java')
    for profile,leg in manifest['jdk_legs'].items():
        for side in ('original','jarde'):
            base=O/(pkg+'-'+profile+'-'+side);src=base/'src';classes=base/'classes'
            assert H(src/(runner+'.java'))==H(runner_path)
            args=[leg['tools']['javac']['path']]+(['-source','8','-target','8'] if profile=='javac8' else ['--release','8'])+['-d',str(classes)]
            if side=='original':
                assert H(classes/pkg/(cls+'.class'))==H(original)
                args+=['-cp',str(classes),str(src/(runner+'.java'))]
            else:
                assert H(src/(cls+'.java'))==H(P/(pkg+'.java'))
                args+=[str(src/(cls+'.java')),str(src/(runner+'.java'))]
            prefix=pkg+'-'+profile+'-'+side
            expected[prefix+'-compile']=args
            expected[prefix+'-runtime']=[leg['tools']['java']['path'],'-Xverify:all','-cp',str(classes),pkg+'.'+runner]
            for path in classes.rglob('*.class'):generated.append({'path':str(path),'bytes':path.stat().st_size,'sha256':H(path)})
assert len(expected)==16 and len({c['label'] for c in ex['commands']})==16
assert {c['label']:c['argv'] for c in ex['commands']}==expected
record={'status':'passed_two_current_cli_cases_only','verifier':{'path':str(verifier),'sha256':H(verifier),'exit_code':p.returncode},'runner':{'path':'/private/tmp/jarde-cf07-cf13-current-runtime-root-v2.py','sha256':H('/private/tmp/jarde-cf07-cf13-current-runtime-root-v2.py')},'execution_sha256':H(O/'execution.json'),'patrol_sha256':H(P/'execution.json'),'commands':16,'paired_runtime_legs':4,'generated_classes':generated,'scope':'No fresh JADX execution, no whole CF07/CF13 completion; precise argv/input/source/raw validation.'}
(A/'verification.json').write_text(json.dumps(record,indent=2)+'\n');print(record['status'])
