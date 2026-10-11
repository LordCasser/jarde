from pathlib import Path
import hashlib,json,shutil
R=Path('/Users/lordcasser/workspace/projects/jarde')
O=R/'openspec/changes/recover-proved-nested-loop-side-exits/results/controls-baseline-root-v1'
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
receipt=Path('/private/tmp/jarde-cf11-side-exits-controls-acceptance-root-v2.json');accepted=json.loads(receipt.read_text());assert accepted['status']=='accepted_scoped_baseline_observation'
assert H(accepted['execution']['path'])==accepted['execution']['sha256']
assert H('/private/tmp/jarde-cf11-side-exits-controls-verifier-root-v2.py')==accepted['verifier']['sha256']
O.mkdir(exist_ok=False);inventory=[]
def cp(src,rel):
 dst=O/rel;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dst);assert dst.read_bytes()==src.read_bytes();inventory.append({'source':str(src),'archived':str(rel),'bytes':dst.stat().st_size,'sha256':H(dst)})
for version in [1,2,3]:
 capture=Path(f'/private/tmp/jarde-cf11-side-exits-controls-baseline-root-v{version}')
 x=json.loads((capture/'execution.json').read_text())
 assert len(x['commands'])==(18 if version==1 else 25)
 for command in x['commands']:
  assert command['guard_stop'] is None and command['peak_target_bytes']<=1024**3 and command['free_bytes_after']>=5*1024**3
  for stream in command['streams'].values():assert H(stream['path'])==stream['sha256'] and Path(stream['path']).stat().st_size==stream['bytes']
 for p in sorted(capture.rglob('*')):
  if p.is_file():cp(p,Path('captures')/f'v{version}'/p.relative_to(capture))
 source=Path(f'/private/tmp/jarde-cf11-side-exits-controls-root-v{version}')
 for p in sorted(source.iterdir()):
  if p.is_file():cp(p,Path('programs')/f'collector-v{version}'/p.name)
 for suffix in ['stdout','stderr']:
  p=Path(str(capture)+'.'+suffix);cp(p,Path('invocations')/p.name)
for version in [1,2]:
 for suffix in ['py']:
  p=Path(f'/private/tmp/jarde-cf11-side-exits-controls-verifier-root-v{version}.{suffix}');cp(p,Path('programs')/p.name)
 for suffix in ['stdout','stderr']:
  p=Path(f'/private/tmp/jarde-cf11-side-exits-controls-acceptance-root-v{version}.{suffix}');cp(p,Path('invocations')/p.name)
cp(receipt,Path('verification')/receipt.name)
cp(Path(__file__),Path('programs')/Path(__file__).name)
(O/'archive-inventory.json').write_text(json.dumps(inventory,indent=2)+'\n')
(O/'README.md').write_text('''# Controlled nested-loop exit baseline

Only capture v3 is accepted for task1.2 by the actual frozen root verifier v2 (exit0).
25 commands/50 streams: two JDK originals and four fresh JADX complete-source runs
match all ten threshold/bound/window rows. Old frozen Jarde CLI generates four
fallback complete classes; their javac exits1, no generated class or runtime.
Default/all text, exact method identities, maps and all physical BCI origins agree.
The constructor/testFor/runCounts instruction counts are 3/40/47 per JDK;
classfile decoder and javap, method indices/flags/Code envelope and UTF8 spans
were independently checked. BLAKE3 report identities were not independently hashed.

v1 is a preserved collector failure at mkdir, after18 actual commands; v2 is a
complete observed different variant with tmp++ inside its condition (normal6).
v3 restores upstream continuation placement, tmp++ after the condition (normal12).
It tests reachable earlyreturn with ticks13/1/4 and normal20, zero-J/zero-K ticks0.
Neither v1 nor v2 is feature acceptance or task1.2 acceptance.

All raw commands, outputs, source/class/JAR inputs and frozen collector versions
are retained. Guard 5GiB free/target1GiB polls every second; no Cargo executed.
The guard env_overrides is template metadata, not measured Cargo environment.
This baseline uses the previously accepted computed-for CLI, not a new product.
CF11 and the 71-unit ledger remain open; implementation and post-fix verification
are outstanding.
''')
print(json.dumps({'status':'accepted_archive','files':len(inventory),'bytes':sum(x['bytes'] for x in inventory)}))
