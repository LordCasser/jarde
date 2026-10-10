#!/usr/bin/env python3
"""Run actual internal exception-edge and public Stop/boundary regressions serially."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,os
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS=ROOT/'openspec/changes/preserve-proved-if-arm-join-origins/results'
OUT=RESULTS/'boundary-tests-root-v2'
TEMPLATE=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
SHA='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
TEST='build::tests::exception_if_join_transfer_rejects_a_canonical_exception_edge'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--build-sha256',required=True);args=parser.parse_args()
    assert sha(TEMPLATE)==SHA and sha(ROOT/'crates/jarde-java/src/build.rs')==args.build_sha256
    assert not OUT.exists();OUT.mkdir()
    spec=importlib.util.spec_from_file_location('loop_guard',TEMPLATE);runner=importlib.util.module_from_spec(spec);spec.loader.exec_module(runner);runner.OUT=OUT
    env=os.environ.copy()
    for k in runner.STRIPPED_ENV_KEYS:env.pop(k,None)
    env.update(runner.ENV_VALUES);jdk=runner.configure_jdk23(env)
    paths=['crates/jarde-java/src/build.rs','crates/jarde-java/src/region.rs','crates/jarde-java/tests/p3_loop_exit_gateways.rs','openspec/changes/preserve-proved-if-arm-join-origins/results/exception-join-case-root-v1/classes/ifjoin/ExceptionIfJoin.class']
    pins={p:sha(ROOT/p) for p in paths}
    expected={2:[(1,0,0)],3:[(12,0,0)]}
    def summaries(index,raw):
        actual=[tuple(map(int,x)) for x in runner.SUMMARY_RE.findall(raw.decode())]
        names=[TEST] if index==2 else ['cf07_nonempty_if_join_goto_is_derived_from_complete_if','cf07_if_join_origin_rejects_nonjoin_and_nontransfer_terminals','cf07_if_join_proof_budget_and_cancellation_publish_no_partial_source']
        ok=actual==expected[index] and all(f'test {name} ... ok' in raw.decode() for name in names)
        return {'expected':expected[index],'actual':actual,'required_test_names':names,'ok':ok}
    runner.expected_test_summaries=summaries
    commands=[(2,['cargo','test','-p','jarde-java','--lib','--locked',TEST,'--','--exact','--nocapture']),(3,['cargo','test','-p','jarde-java','--test','p3_loop_exit_gateways','--locked','--','--nocapture'])]
    rows=[];ok=True
    for i,argv in commands:
        row=runner.run_command(i,argv,env);rows.append(row)
        ok=ok and row['exit_code']==0 and row['guard_stop'] is None and row['test_summary_check']['ok']
        if not ok:break
    after={p:sha(ROOT/p) for p in paths};ok=ok and pins==after
    record={'schema':'preserve-proved-if-arm-boundary-tests-root-v2','status':'passed' if ok else 'failed','runner':{'path':str(Path(__file__).resolve()),'sha256':sha(Path(__file__))},'template':{'path':str(TEMPLATE),'sha256':SHA},'jdk':jdk,'guards':{'minimum_free_bytes':5*1024**3,'maximum_target_bytes':1024**3,'poll_interval_seconds':1,'process_group_stop':True},'source_pins_before':pins,'source_pins_after':after,'commands':rows,'root_private_patch_corrections':['Visitor cannot retain a borrowed Region beyond the callback lifetime; clone the actual recovered Region for this test only.','Discard unused helper return tuple; all proof assertions are in observer.','Public IR-edge Stop must point to actual new helper BCI20.']}
    (OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps({'status':record['status'],'commands':len(rows)}));return 0 if ok else 1
if __name__=='__main__':raise SystemExit(main())
