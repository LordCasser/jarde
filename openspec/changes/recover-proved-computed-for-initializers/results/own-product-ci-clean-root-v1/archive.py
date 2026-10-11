from pathlib import Path
import hashlib,json,shutil
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
OUT=ROOT/'openspec/changes/recover-proved-computed-for-initializers/results/own-product-ci-clean-root-v1'
accept=Path('/private/tmp/jarde-computed-for-own-ci-acceptance-root-v1.json')
a=json.loads(accept.read_text());assert a['status']=='accepted' and a['product_commit']=='8bef1b76acfc93e1b1510e8cb71d7c7a25b0fdc6' and a['run_id']==38131239404
OUT.mkdir();rows=[]
def cp(src,dst):
 dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(src,dst);raw=src.read_bytes();assert dst.read_bytes()==raw
 rows.append({'source':str(src),'archived':str(dst.relative_to(OUT)),'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
for label,source in [('capture','/private/tmp/jarde-computed-for-own-ci-capture-root-v1'),('product-clean-audit','/private/tmp/jarde-computed-for-product-clean-audit-root-v1')]:
 base=Path(source)
 for p in sorted(base.rglob('*')):
  if p.is_file():cp(p,OUT/label/p.relative_to(base))
for name in ['jarde-computed-for-own-ci-verifier-root-v1.py','jarde-computed-for-own-ci-acceptance-root-v1.json','jarde-computed-for-own-ci-acceptance-root-v1.stdout','jarde-computed-for-own-ci-acceptance-root-v1.stderr','jarde-computed-for-own-ci-acceptance-root-v1.invocation.json','jarde-computed-for-run-own-ci-verification-root-v1.py','jarde-computed-for-own-ci-preflight-root-v1.json','jarde-computed-for-own-ci-preflight-root-v1.stdout','jarde-computed-for-own-ci-preflight-root-v1.stderr','jarde-computed-for-clean-main-audit-root-v1.py']:
 cp(Path('/private/tmp')/name,OUT/'verification'/name)
cp(ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/capture-conditional-ci-root-v3.py',OUT/'verification/capture-conditional-ci-root-v3.py')
cp(Path(__file__),OUT/'archive.py')
(OUT/'archive-inventory.json').write_text(json.dumps(rows,indent=2)+'\n')
counts=[r['workspace_totals'] for r in a['ci']['seed_runs']]
(OUT/'README.md').write_text(f'''# 当前产品自身 CI 与干净主线\n\n实际产品 `8bef1b76acfc93e1b1510e8cb71d7c7a25b0fdc6` 已推送 main；自身 [CI 38131239404](https://github.com/LordCasser/jarde/actions/runs/38131239404) 由 root 采集完整 summary 和 stable/supply raw 日志并独立执行 verifier exit 0。4 jobs、{a['ci']['steps']} steps 成功，两固定 seed 各359 metadata targets，实际汇总 {counts}。所有 named outcomes 与 fresh 本地完整 workspace 逐一绑定；公开脱敏名沿既有唯一同状态绑定规则，原日志不修改。\n\n4700 动态 source/include/fixture 输入绑定本产品 Git blobs、live 文件及最终冻结 CLI 构建 closure；新 computed-for/shared-SSA/return-leaf 来源回归全部准确通过。CLI SHA `d1543cb69867d78f964873fe0e3d534f18923d3350d6dcf88b6a7e02738b56d1`、60821472 bytes、0555，已 clean。没有借旧产品 CI。\n\n实际产品提交后的 clean audit exit 0：HEAD=main=origin/main，真实远端与本地仅main，15 checkout 都干净且无 target/fuzz target，14辅助为 detached/main祖先、不占分支，目录并未删除。关闭文档提交后的最终主线审计另以新的实际 HEAD 运行，不冒称本目录预先核过未来提交。持续目标 active；整 CF07/71 单元未关闭，共享赋值子句完整恢复仍单列债务。\n\n本目录保留原 raw/receipt 的字节和路径，archive inventory 将私有原件绑定到仓库副本。\n''')
print(json.dumps({'files':len(rows),'bytes':sum(r['bytes'] for r in rows)}))
