#!/usr/bin/env python3
"""Root-run template for one temporary internal Region diagnostic test."""
from __future__ import annotations
import argparse, datetime, hashlib, importlib.util, json, os, shutil
from pathlib import Path
ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS = ROOT / 'openspec/changes/preserve-proved-for-latch-origins/results/If'
OUT = RESULTS / 'internal-diagnostic-root-v1'
TEMPLATE = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
TEMPLATE_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
REGION_SOURCE = ROOT / 'crates/jarde-java/src/region.rs'
TEST_NAME = 'diagnose_cf07_counted_if_join_from_real_class_ir'
ARGV = ['cargo','test','-p','jarde-java','--lib','--locked',f'region::tests::{TEST_NAME}','--','--exact','--nocapture']
def sha_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()
def require(ok: bool, message: str) -> None:
    if not ok: raise RuntimeError(message)
def main() -> int:
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--expected-patch-region-sha256',required=True)
    args=parser.parse_args()
    require(sha_file(TEMPLATE)==TEMPLATE_SHA256,'pinned v9 guarded runner SHA changed')
    require(REGION_SOURCE.is_file(),'region.rs missing')
    require(sha_file(REGION_SOURCE)==args.expected_patch_region_sha256,'temporary diagnostic patch SHA differs')
    require(not OUT.exists(),f'exclusive output already exists: {OUT}')
    free_before=shutil.disk_usage(ROOT).free
    target=ROOT/'target'
    target_bytes_before=sum(p.stat().st_size for p in target.rglob('*') if p.is_file()) if target.exists() else 0
    require(free_before>=5*1024**3,f'5 GiB free-space preflight failed: {free_before}')
    require(target_bytes_before<=1024**3,f'1 GiB target preflight failed: {target_bytes_before}')
    spec=importlib.util.spec_from_file_location('pinned_loop_latch_runner_v9',TEMPLATE)
    require(spec is not None and spec.loader is not None,'cannot import pinned v9 guarded runner')
    runner=importlib.util.module_from_spec(spec); spec.loader.exec_module(runner)
    require(runner.sha_file(TEMPLATE)==TEMPLATE_SHA256,'imported guard helper changed')
    OUT.mkdir(parents=True,exist_ok=False); runner.OUT=OUT
    env=os.environ.copy()
    for key in runner.STRIPPED_ENV_KEYS: env.pop(key,None)
    env.update(runner.ENV_VALUES); jdk=runner.configure_jdk23(env)
    source_before=sha_file(REGION_SOURCE)
    started=datetime.datetime.now(datetime.timezone.utc).isoformat()
    row=runner.run_command(2,ARGV,env)
    source_after=sha_file(REGION_SOURCE)
    stdout=(OUT/'2.stdout.raw').read_text(encoding='utf-8',errors='replace')
    summaries=runner.SUMMARY_RE.findall(stdout)
    exact=[line for line in stdout.splitlines() if line.strip()==f'test region::tests::{TEST_NAME} ... ok']
    passed=(row['exit_code']==0 and row['guard_stop'] is None and summaries==[('1','0','0')]
            and exact==[f'test region::tests::{TEST_NAME} ... ok'] and source_before==source_after)
    record={'schema':'preserve-proved-for-latch-origins-internal-diagnostic-root-v1',
      'status':'diagnostic-passed' if passed else 'diagnostic-failed','test':TEST_NAME,'argv':ARGV,'cwd':str(ROOT),
      'started_at':started,'guarded_runner_template':{'path':str(TEMPLATE),'sha256':TEMPLATE_SHA256},'jdk23':jdk,
      'environment_overrides':runner.ENV_VALUES,'stripped_java_environment_keys':list(runner.STRIPPED_ENV_KEYS),
      'guards':{'minimum_free_bytes':5*1024**3,'maximum_target_bytes':1024**3,'poll_interval_seconds':1,'process_group_stop':True},
      'source':{'path':str(REGION_SOURCE),'sha256_before':source_before,'sha256_after':source_after,
                'unchanged_during_test':source_before==source_after,'note':'temporary diagnostic source is not a product pin'},
      'preflight':{'free_bytes':free_before,'target_bytes':target_bytes_before},'command':row,
      'exact_test_line':exact,'summary_matches':summaries,'free_bytes_after':shutil.disk_usage(ROOT).free}
    with (OUT/'execution.json').open('x',encoding='utf-8') as stream: json.dump(record,stream,ensure_ascii=False,indent=2); stream.write('\n')
    return 0 if passed else 1
if __name__=='__main__': raise SystemExit(main())
