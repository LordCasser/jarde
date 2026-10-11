import hashlib, json, pathlib, re, shutil, subprocess, time
R = pathlib.Path('/Users/lordcasser/workspace/projects/jarde')
P = pathlib.Path('/private/tmp/jarde-cf07-cf13-current-cli-patrol-root-v1')
O = pathlib.Path('/private/tmp/jarde-cf07-cf13-current-runtime-root-v2')
O.mkdir(exist_ok=False)
H = lambda p: hashlib.sha256(pathlib.Path(p).read_bytes()).hexdigest()
manifest = json.loads((R/'openspec/changes/preserve-proved-if-arm-join-origins/results/cf07-candidate-root-v1/manifest.json').read_text())
patrol = json.loads((P/'execution.json').read_text())
inputs = []
commands = []
def pin(p):
    p = pathlib.Path(p); inputs.append({'path':str(p), 'sha256':H(p)})
def run(label, argv):
    t=time.monotonic(); p=subprocess.run(list(map(str,argv)),capture_output=True,timeout=45)
    for name,data in [('stdout',p.stdout),('stderr',p.stderr)]:
        (O/(label+'.'+name+'.raw')).write_bytes(data)
    commands.append({'label':label,'argv':list(map(str,argv)),'exit_code':p.returncode,'duration_seconds':time.monotonic()-t,'stdout_sha256':hashlib.sha256(p.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(p.stderr).hexdigest()})
    assert p.returncode == 0, label
    return p.stdout
pin(patrol['commands'][0]['argv'][0])
assert inputs[-1]['sha256']==patrol['cli_sha256']
cases=[('cf07','LoopCases','Runner',R/'openspec/changes/preserve-proved-if-arm-join-origins/results/cf07-candidate-root-v1/original-sources/Runner.java'),('cf13','SwitchLoopExits','SwitchLoopExitsRunner',R/'openspec/evidence/java-syntax-2026-09-27/cf13-switch-exits/input/SwitchLoopExitsRunner.java')]
results=[]
try:
    for idx,(pkg,cls,runner,runner_path) in enumerate(cases):
        original=pathlib.Path(patrol['commands'][idx]['argv'][3]);pin(original);pin(runner_path);pin(P/(pkg+'.stdout.raw'));pin(P/(pkg+'.java'))
        report=json.loads((P/(pkg+'.stdout.raw')).read_text())
        assert report['text']==(P/(pkg+'.java')).read_text()
        if pkg=='cf07':
            method=next(m for m in report['methods'] if m['item']['name']['escaped']=='counted')['outcome']['report']
            assert method['quality']=='structured'
            segments=method['source_map']['segments']
            joins=[s for s in segments if any(a['bci']==20 for a in s['origin']['derived'])]
            assert len(joins)==1 and joins[0]['origin']['primary']['bci']==14
            join=joins[0];span=method['text'].encode()[join['start']:join['end']].decode()
            assert span.lstrip().startswith('if (local3 == 7)') and '} else {' in span and span.rstrip().endswith('}')
            (O/'counted-join.json').write_text(json.dumps({'segment':join,'exact_slice':span},indent=2)+'\n')
        else:
            method=next(m for m in report['methods'] if m['item']['name']['escaped']=='walk')['outcome']['report']
            assert method['quality']=='structured' and 'continue;' in method['text'] and 'switch (' in method['text']
        for profile,leg in manifest['jdk_legs'].items():
            for tool in ('javac','java','javap'):
                pin(leg['tools'][tool]['path']);assert inputs[-1]['sha256']==leg['tools'][tool]['sha256']
            outputs={}
            for side in ('original','jarde'):
                base=O/(pkg+'-'+profile+'-'+side);src=base/'src';classes=base/'classes';src.mkdir(parents=True);classes.mkdir()
                runner_copy=src/(runner+'.java');shutil.copyfile(runner_path,runner_copy)
                argv=[leg['tools']['javac']['path']]+(['-source','8','-target','8'] if profile=='javac8' else ['--release','8'])+['-d',str(classes)]
                if side=='original':
                    dst=classes/pkg/(cls+'.class');dst.parent.mkdir();shutil.copyfile(original,dst)
                    argv+=['-cp',str(classes),str(runner_copy)]
                else:
                    source=src/(cls+'.java');shutil.copyfile(P/(pkg+'.java'),source);argv += [str(source),str(runner_copy)]
                run(pkg+'-'+profile+'-'+side+'-compile',argv)
                outputs[side]=run(pkg+'-'+profile+'-'+side+'-runtime',[leg['tools']['java']['path'],'-Xverify:all','-cp',str(classes),pkg+'.'+runner])
            assert outputs['original']==outputs['jarde']
            if pkg=='cf13':assert outputs['original']==b'38\n'
            results.append({'case':pkg,'profile':profile,'equal':True,'stdout':outputs['original'].decode()})
    assert all(H(x['path'])==x['sha256'] for x in inputs)
    status='passed_current_cli_two_cases_only'
finally:
    (O/'execution.json').write_text(json.dumps({'product_commit':patrol['product_commit'],'cli_sha256':patrol['cli_sha256'],'status':locals().get('status','failed'),'inputs':inputs,'commands':commands,'results':results,'scope':'Current frozen CLI source-map and complete-class runtime patrol; no fresh JADX execution, no whole-unit completion.'},indent=2)+'\n')
print(json.dumps({'status':status,'commands':len(commands),'runtime_legs':len(results)}))
