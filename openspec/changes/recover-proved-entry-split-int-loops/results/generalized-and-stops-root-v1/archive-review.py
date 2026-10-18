from pathlib import Path
import json,tarfile,hashlib,stat,shutil
base=Path(__file__).parent
inv=json.loads((base/'capture.tar.gz.inventory.json').read_text())
script=Path('/private/tmp/jarde-entry-split-durable-pack-root-v2-20261019/pack.py')
ns={'__file__':str(script),'__name__':'frozen_packer_constants'}
exec(compile(script.read_text(),str(script),'exec'),ns)
roots={prefix:root for root,prefix in ns['ROOTS']}
with tarfile.open(base/'capture.tar.gz','r:gz') as t:
 assert len(t.getmembers())==inv['member_count']+1
 for item in inv['members']:
  name=item['path'];prefix=max((p for p in roots if name.startswith(p+'/')),key=len);original=roots[prefix]/name[len(prefix)+1:];data=t.extractfile(name).read()
  assert data==original.read_bytes() and hashlib.sha256(data).hexdigest()==item['sha256'] and len(data)==item['bytes']
  assert t.getmember(name).mode==stat.S_IMODE(original.stat().st_mode)==item['mode']
 assert json.loads(t.extractfile('ARCHIVE_INVENTORY.json').read())==inv
review={'status':'root_reopened_all_archive_member_bytes_and_modes','payload_members':inv['member_count'],'archive_sha256':hashlib.sha256((base/'capture.tar.gz').read_bytes()).hexdigest(),'notes':['Root excludes previously archived five-fixture gate and focused reviews to avoid duplicate payload.','Root stop review and all actual failures preserved; no CLI/Rust binary included.','Initial archive verifier exec missed __file__; fixed verifier context only, no collection/test rerun.']}
dest=Path('/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-entry-split-int-loops/results/generalized-and-stops-root-v1');dest.mkdir()
shutil.copy2(base/'capture.tar.gz',dest/'capture.tar.gz');shutil.copy2(base/'capture.tar.gz.inventory.json',dest/'inventory.json');(dest/'archive-root-review.json').write_text(json.dumps(review,ensure_ascii=False,indent=2)+'\n');shutil.copy2(Path(__file__),dest/'archive-review.py');print(json.dumps(review,ensure_ascii=False))
