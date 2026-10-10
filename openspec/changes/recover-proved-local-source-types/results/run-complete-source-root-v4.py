from pathlib import Path
import subprocess,json,hashlib,sys
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
R=ROOT/'openspec/changes/recover-proved-local-source-types/results'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
meta=R/'candidate-cli-typed-root-v1.json'
m=json.loads(meta.read_bytes());cli=Path(m['cli_path']);build=R/'validation-build-root-v2/execution.json'
assert sha(cli)==m['cli_sha256'] and sha(build)==m['build_result_sha256']
common=['--cli',str(cli),'--cli-sha256',sha(cli),'--metadata',str(meta),'--metadata-sha256',sha(meta),'--source-base','5c2c06f1ec8c3ff0560f2c2d89059ee7d6d06b02']
replay=common+['--validation-execution',str(build),'--validation-sha256',sha(build),'--pretyped-execution','/private/tmp/jarde-cf12-post-pop-baseline-root-v4/execution.json','--pretyped-execution-sha256','93dc97cf213358e94d56bfe52846db79ffe765a9fd187c5e5bad3ed56433bc08','--pretyped-acceptance','/private/tmp/jarde-cf12-post-pop-baseline-root-v4/acceptance.json','--pretyped-acceptance-sha256','bc9be073af6fd98817d1bae6e541f630efea714cc2856a3981ff2bde76c7df06','--jdk8-home','/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home','--jdk23-home','/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home','--jadx','/opt/homebrew/Cellar/jadx/1.5.6/bin/jadx','--jadx-sha256','64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7']
verify_replay=[x for x in replay]
idx=verify_replay.index('--jadx');verify_replay=verify_replay[:idx]
D=R;O=R/'complete-source-root-v4';I=R/'int-overload-root-v1';P=R/'private-int-overload-luna-v2'
commands=[(['python','-B',str(D/'prepare-local-source-types-replay-root-v2.py'),*replay,'--out',str(O)],R/'complete-source-invocation-root-v4'),(['uv','run','--offline','--with','blake3','python','-B',str(D/'verify-local-source-types-replay-root-v1.py'),*verify_replay,'--execution',str(O/'execution.json')],R/'complete-source-verifier-invocation-root-v4'),(['python','-B',str(P/'collect.py'),*common,'--build-execution',str(build),'--build-sha256',sha(build),'--out',str(I)],R/'int-overload-invocation-root-v1'),(['uv','run','--offline','--with','blake3','python','-B',str(P/'verify.py'),*common,'--build-execution',str(build),'--build-sha256',sha(build),'--execution',str(I/'execution.json'),'--acceptance',str(I/'acceptance.json')],R/'int-overload-verifier-invocation-root-v1')]
for argv,evidence in commands:
    code=subprocess.run(['python','-B','/private/tmp/jarde-record-invocation.py',str(evidence),*argv],cwd=ROOT).returncode
    if code:sys.exit(code)

observed=(R/'complete-source-verifier-invocation-root-v4/stdout.raw').read_bytes()
a=json.loads(observed);assert a['status']=='verified-typed-replay-observations-only' and a['cf12_complete'] is False
with (O/'acceptance.json').open('xb') as f:f.write(observed)
