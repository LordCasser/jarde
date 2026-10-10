from pathlib import Path
import hashlib,json,subprocess
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
md=RESULTS/'candidate-cli-root-v1.json';build=RESULTS/'validation-build-root-v2/execution.json';cli=Path('/private/tmp/jarde-proved-conditional-switch-cli-v1')
b=json.loads(build.read_text());assert b['status']=='validation-passed-cli-frozen'
jdk=json.loads((ROOT/'openspec/changes/recover-nested-int-array-compound-updates/results/controls-v1/manifest.json').read_text())
homes={r['leg']:str(Path(r['jdk_tools']['javac']['path']).parent.parent) for r in jdk['legs']}
jadx=Path('/opt/homebrew/bin/jadx').resolve();out=RESULTS/'complete-source-root-v1'
args=['python','-B',str(RESULTS/'prepare-conditional-replay.py'),'--cli',str(cli),'--cli-sha256',sha(cli),'--metadata',str(md),'--metadata-sha256',sha(md),'--validation-execution',str(build),'--validation-sha256',sha(build),'--source-base','2d70da515896c25ce022b8c28f4935ff2e105026','--typed-execution',str(ROOT/'openspec/changes/recover-proved-local-source-types/results/complete-source-root-v4/execution.json'),'--typed-execution-sha256','27afad91dc4e5c426d90f6ff4253b4244541490161752b759d1f5ada19847ed5','--typed-acceptance',str(ROOT/'openspec/changes/recover-proved-local-source-types/results/complete-source-root-v4/acceptance.json'),'--typed-acceptance-sha256','d9e336b8b6eb33b76c0d866eb83040540586eba9df5ba629311f2371f577812f','--jdk8-home',homes['javac8'],'--jdk23-home',homes['javac23'],'--jadx',str(jadx),'--jadx-sha256',sha(jadx),'--out',str(out)]
raise SystemExit(subprocess.call(['python','-B','/private/tmp/jarde-record-invocation.py','/private/tmp/jarde-conditional-complete-replay-invocation-root-v1',*args],cwd=ROOT))
