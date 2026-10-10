#!/usr/bin/env python3
"""Root-run template for focused gateway tests on the applied If-origin product."""
from __future__ import annotations
import argparse, datetime, hashlib, importlib.util, json, os, shutil
from pathlib import Path
ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS = ROOT / 'openspec/changes/preserve-proved-if-arm-join-origins/results'
OUT = RESULTS / 'focused-gateway-root-v2'
TEMPLATE = ROOT / 'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
TEMPLATE_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
REGION_SOURCE = ROOT / 'crates/jarde-java/src/build.rs'
TEST_NAME = 'cf07_nonempty_if_join_goto_is_derived_from_complete_if'
REQUIRED_NAMES = ('cf07_nonempty_if_join_goto_is_derived_from_complete_if', 'cf07_if_join_origin_rejects_nonjoin_and_nontransfer_terminals', 'cf07_if_join_proof_budget_and_cancellation_publish_no_partial_source')
ARGV = ['cargo','test','-p','jarde-java','--test','p3_loop_exit_gateways','--locked','--','--nocapture']
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
    stripped = []
    for key in runner.STRIPPED_ENV_KEYS:
        if key in env:
            stripped.append(key); env.pop(key)
    env.update(runner.ENV_VALUES); jdk=runner.configure_jdk23(env)
    source_before=sha_file(REGION_SOURCE)
    started=datetime.datetime.now(datetime.timezone.utc).isoformat()
    def one_test_summary(index, raw):
        text = raw.decode('utf-8', errors='strict')
        actual = runner.SUMMARY_RE.findall(text)
        named = [line for line in text.splitlines() if line.strip() == f'test {TEST_NAME} ... ok']
        return {'expected': [[12,0,0]], 'actual': [[int(value) for value in item] for item in actual], 'required': True, 'ok': actual == [('12','0','0')] and len(named) == 1, 'required_test_names': [f'{TEST_NAME}'], 'required_test_names_present': len(named) == 1}
    runner.expected_test_summaries = one_test_summary
    pins = {name: sha_file(ROOT / name) for name in ['crates/jarde-java/src/build.rs','crates/jarde-java/tests/p3_loop_exit_gateways.rs','crates/jarde-java/src/region.rs']}
    row=runner.run_command(3,ARGV,env)
    source_after=sha_file(REGION_SOURCE)
    stdout=(OUT/'3.stdout.raw').read_text(encoding='utf-8',errors='replace')
    summaries=runner.SUMMARY_RE.findall(stdout)
    exact=[line for line in stdout.splitlines() if line.strip()==f'test {TEST_NAME} ... ok']
    passed=(row['exit_code']==0 and row['guard_stop'] is None and summaries==[('12','0','0')]
            and all(f'test {name} ... ok' in stdout for name in REQUIRED_NAMES) and all(sha_file(ROOT / name)==value for name,value in pins.items()) and row['test_summary_check']['ok'] and exact==[f'test {TEST_NAME} ... ok'] and source_before==source_after)
    record={'schema':'preserve-proved-if-arm-join-origins-focused-gateway-root-v2',
      'status':'diagnostic-passed' if passed else 'diagnostic-failed','test':TEST_NAME,'argv':ARGV,'cwd':str(ROOT),
      'started_at':started,'diagnostic_runner':{'path':str(Path(__file__).resolve()),'sha256':sha_file(Path(__file__))},'guarded_runner_template':{'path':str(TEMPLATE),'sha256':TEMPLATE_SHA256},'jdk23':jdk,
      'environment_overrides':runner.ENV_VALUES,'stripped_java_environment_keys':stripped,
      'guards':{'minimum_free_bytes':5*1024**3,'maximum_target_bytes':1024**3,'poll_interval_seconds':1,'process_group_stop':True},
      'source':{'path':str(REGION_SOURCE),'sha256_before':source_before,'sha256_after':source_after,
                'unchanged_during_test':source_before==source_after,'note':'applied product build.rs; full candidate freeze not yet performed'},
      'product_source_pins_before':pins,'required_test_names':list(REQUIRED_NAMES),'preflight':{'free_bytes':free_before,'target_bytes':target_bytes_before},'command':row,
      'exact_test_line':exact,'summary_matches':summaries,'free_bytes_after':shutil.disk_usage(ROOT).free}
    with (OUT/'execution.json').open('x',encoding='utf-8') as stream: json.dump(record,stream,ensure_ascii=False,indent=2); stream.write('\n')
    return 0 if passed else 1
if __name__=='__main__': raise SystemExit(main())
