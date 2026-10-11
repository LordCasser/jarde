from pathlib import Path
import json,hashlib,shutil
OUT=Path('/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-nested-loop-side-exits/results/own-product-ci-clean-root-v1')
ci=json.loads(Path('/private/tmp/jarde-cf11-own-product-ci-acceptance-root-v1.json').read_text())
assert ci['status']=='accepted' and ci['ci']['jobs']==4 and ci['ci']['steps']==52
rows=[]
def cp(src,rel):
 dst=OUT/rel
 assert not dst.exists(),dst
 dst.parent.mkdir(parents=True,exist_ok=True)
 shutil.copyfile(src,dst)
 raw=src.read_bytes()
 assert dst.read_bytes()==raw
 rows.append({'source':str(src),'archived':str(rel),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
for src in map(Path,('/private/tmp/jarde-cf11-own-product-ci-acceptance-root-v1.json','/private/tmp/jarde-cf11-product-local-binding-root-v1.json','/private/tmp/jarde-cf11-product-local-binding-root-v1.py','/private/tmp/jarde-cf11-local-acceptance-root-v3.json','/private/tmp/jarde-cf11-local-acceptance-root-v3.py','/private/tmp/jarde-cf11-own-ci-prep-luna-v5/verify-cf11-own-ci.py','/private/tmp/jarde-conditional-ci-public-labels-root-v1.py','/private/tmp/jarde-cf11-invocations-root-v1/record.py')):
 cp(src,Path('acceptance')/src.name)
for label in ('own-product-ci-capture','own-product-ci-verifier','own-product-evidence-archive','product-local-binding','product-clean-audit','local-acceptance'):
 base=Path('/private/tmp/jarde-cf11-invocations-root-v1')/label
 for p in sorted(base.rglob('*')):
  if p.is_file():cp(p,Path('invocations')/label/p.relative_to(base))
base=Path('/private/tmp/jarde-cf11-staged-whitespace-root-v1')
for p in sorted(base.rglob('*')):
 if p.is_file():cp(p,Path('staged-whitespace-product')/p.relative_to(base))
cp(Path(__file__),Path('acceptance')/Path(__file__).name)
(OUT/'supplement-inventory.json').write_text(json.dumps({'status':'archived_accepted_actual_receipts','files':rows},indent=2)+'\n')
with (OUT/'README.md').open('a') as f:
 f.write('\nRoot独立验收已实际执行：acceptance/jarde-cf11-own-product-ci-acceptance-root-v1.json 为 accepted，精确产品 b155877ee9a3b1894b053e70ffbf1cbf0a9512fa / CI 38144212651，4 jobs/52 steps，双seed各360目标/3455-0-97。全部named outcomes与fresh workspace相同；两个GitHub脱敏public labels只绑定唯一同status本地结果。完整4707 inputs与产品Git/live/冻结CLI一致，38项累计required tests已核。原始调用exit/stdout/stderr见invocations。clean-audit是产品提交后实际干净checkpoint，不冒称后续文档HEAD已经审计；关闭文档推送后将独立执行outside-repo final audit。\n')
print(json.dumps({'supplement_files':len(rows),'ci_acceptance_sha256':hashlib.sha256(Path('/private/tmp/jarde-cf11-own-product-ci-acceptance-root-v1.json').read_bytes()).hexdigest()}))
