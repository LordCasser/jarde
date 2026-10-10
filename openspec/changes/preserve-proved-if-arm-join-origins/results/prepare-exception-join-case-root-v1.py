#!/usr/bin/env python3
"""Root actual javac fixture collection under the pinned 5 GiB / 1 GiB guard."""
from pathlib import Path
import datetime, hashlib, importlib.util, json, os
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
RESULTS=ROOT/'openspec/changes/preserve-proved-if-arm-join-origins/results'
OUT=RESULTS/'exception-join-case-root-v1'
TEMPLATE=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
TEMPLATE_SHA='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SOURCE='''package ifjoin;
public final class ExceptionIfJoin {
    static void mayThrow() {}
    public static int withException(int n) {
        int total = 0;
        int i = 0;
        try {
            while (i < n) {
                if ((i & 1) == 0) {
                    mayThrow();
                    total += 2;
                } else {
                    total++;
                }
                i++;
            }
        } catch (RuntimeException e) {
            return -1;
        }
        return total;
    }
}
'''
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    assert sha(TEMPLATE)==TEMPLATE_SHA
    assert not OUT.exists()
    spec=importlib.util.spec_from_file_location('loop_guard',TEMPLATE)
    runner=importlib.util.module_from_spec(spec);spec.loader.exec_module(runner)
    OUT.mkdir();runner.OUT=OUT
    env=os.environ.copy()
    for k in runner.STRIPPED_ENV_KEYS: env.pop(k,None)
    env.update(runner.ENV_VALUES);jdk=runner.configure_jdk23(env)
    src=OUT/'src/ifjoin/ExceptionIfJoin.java';src.parent.mkdir(parents=True);src.write_text(SOURCE)
    for d in ('empty-classpath','empty-sourcepath','classes'): (OUT/d).mkdir()
    argv=[jdk['tools']['javac']['path'],'--release','8','-g:none','-proc:none','-classpath',str(OUT/'empty-classpath'),'-sourcepath',str(OUT/'empty-sourcepath'),'-d',str(OUT/'classes'),str(src)]
    rows=[runner.run_command(20,argv,env)]
    assert rows[-1]['exit_code']==0 and rows[-1]['guard_stop'] is None,rows[-1]
    target=OUT/'classes/ifjoin/ExceptionIfJoin.class'
    rows.append(runner.run_command(21,[jdk['tools']['javap']['path'],'-c','-p','-s',str(target)],env))
    assert rows[-1]['exit_code']==0 and rows[-1]['guard_stop'] is None,rows[-1]
    record={'schema':'preserve-proved-if-arm-exception-fixture-root-v1','status':'compiled-and-javap-captured','utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'runner':{'path':str(Path(__file__).resolve()),'sha256':sha(Path(__file__))},'guard_template':{'path':str(TEMPLATE),'sha256':TEMPLATE_SHA},'jdk':jdk,'commands':rows,'source':{'path':str(src),'sha256':sha(src)},'class':{'path':str(target),'sha256':sha(target),'bytes':target.stat().st_size},'target_code_executed':False}
    (OUT/'execution.json').write_text(json.dumps(record,indent=2)+'\n')
    print(json.dumps(record));print((OUT/'21.stdout.raw').read_text())
if __name__=='__main__': main()
