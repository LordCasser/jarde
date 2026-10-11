#!/usr/bin/env python3
"""Compile the one frozen JadxAssertions source for JDK 8 replay; preparation only."""
import argparse, datetime, hashlib, json, os, shutil, signal, stat, subprocess, sys, time, zipfile
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
PLAN = Path('/private/tmp/jarde-cf12-ten-fixture-replay-plan-luna-v1.json')
SOURCE = Path('/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/api/utils/assertj/JadxAssertions.java')
SOURCE_SHA256 = '8b7d50aca9ec2ac8dd861444bd1854fa6d43adac4d8d9d3c12195c6544bc4d5e'
MIN_FREE, MAX_TARGET, MAX_OUT, TIMEOUT = 5*1024**3, 1024**3, 1024**3, 60

def digest(p):
    h=hashlib.sha256()
    with Path(p).open('rb') as f:
        for b in iter(lambda:f.read(1024*1024),b''): h.update(b)
    return h.hexdigest()
def pin(p):
    p=Path(p); return {'path':str(p),'bytes':p.stat().st_size,'sha256':digest(p)}
def tree_size(p):
    total=0
    if Path(p).exists():
        for x in Path(p).rglob('*'):
            try: st=x.stat()
            except FileNotFoundError: continue
            if stat.S_ISREG(st.st_mode): total+=st.st_size
    return total
def verify(row):
    p=Path(row['path'])
    if not p.is_file() or p.stat().st_size!=row['bytes'] or digest(p)!=row['sha256']:
        raise RuntimeError('pin mismatch: '+str(p))
def main():
    ap=argparse.ArgumentParser(); ap.add_argument('--out',required=True,type=Path); a=ap.parse_args()
    out=a.out.resolve()
    if out.exists(): raise SystemExit('output already exists: '+str(out))
    out.mkdir(parents=True)
    manifest=json.loads(PLAN.read_text()); shared=manifest['shared_pins']
    helpers=shared['helper_class_files']; jars=shared['jadx_product_jars']+shared['test_sdk_jars']
    jdk=shared['jdk23_tools']; javac=Path(jdk['javac']['path']).resolve()
    pins=[pin(Path(__file__)),pin(PLAN),pin(SOURCE),jdk['javac'],*helpers,*jars]
    if pin(javac)['sha256']!=jdk['javac']['sha256']: raise RuntimeError('resolved javac differs from frozen pin')
    for x in pins: verify(x)
    if len(helpers)!=21 or len(jars)!=63: raise RuntimeError('frozen classpath inventory count mismatch')
    # Reject target/observer classes hidden inside any classpath JAR.
    for row in jars:
        with zipfile.ZipFile(row['path']) as z: names=z.namelist()
        bad=[n for n in names if n.endswith('.class') and (n.startswith('jadx/tests/integration/switches/') or n.startswith('cf12capture/'))]
        if bad: raise RuntimeError('forbidden target/observer class in jar '+row['path'])
    if shutil.disk_usage(ROOT).free<MIN_FREE or tree_size(ROOT/'target')>MAX_TARGET: raise RuntimeError('initial resource guard failed')
    cls=out/'classes'; cls.mkdir(); empty=out/'empty-sourcepath'; empty.mkdir(); raw=out/'raw'; raw.mkdir()
    # helper inventory entries are .../helpers/<package>/<class>.class.
    helper_root=Path(helpers[0]['path']).parents[1]
    cp=os.pathsep.join([str(helper_root),*[x['path'] for x in jars]])
    argv=[str(javac),'-encoding','UTF-8','--release','8','-proc:none','-classpath',cp,'-sourcepath',str(empty),'-d',str(cls),str(SOURCE)]
    stdout=raw/'0001-overlay-compile.stdout.raw'; stderr=raw/'0001-overlay-compile.stderr.raw'
    env=os.environ.copy()
    for k in ('JAVA_HOME','JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH'): env.pop(k,None)
    rec={'index':1,'label':'jdk23-compile-single-frozen-helper-release8','argv':argv,'cwd':str(out),'guard_stop':None,'started_at':datetime.datetime.now(datetime.timezone.utc).isoformat()}
    started=time.monotonic()
    with stdout.open('wb') as so,stderr.open('wb') as se:
        p=subprocess.Popen(argv,cwd=out,env=env,stdout=so,stderr=se,start_new_session=True)
        while p.poll() is None:
            reason=('free_floor' if shutil.disk_usage(ROOT).free<MIN_FREE else 'target_limit' if tree_size(ROOT/'target')>MAX_TARGET else 'output_limit' if tree_size(out)>MAX_OUT else 'timeout' if time.monotonic()-started>TIMEOUT else None)
            if reason:
                rec['guard_stop']=reason
                try: os.killpg(p.pid,signal.SIGTERM)
                except ProcessLookupError: pass
                try: p.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    try: os.killpg(p.pid,signal.SIGKILL)
                    except ProcessLookupError: pass
                    p.wait()
                break
            time.sleep(1)
        rec['exit_code']=p.wait()
    rec['duration_seconds']=time.monotonic()-started
    rec['streams']={'stdout':pin(stdout),'stderr':pin(stderr)}
    classes=sorted(str(x.relative_to(cls)) for x in cls.rglob('*.class'))
    overlays=[pin(x) for x in cls.rglob('*.class')]
    for x in pins: verify(x)
    usable=rec['exit_code']==0 and rec['guard_stop'] is None and classes==['jadx/tests/api/utils/assertj/JadxAssertions.class']
    report={'schema':'cf12-jdk8-single-helper-overlay-build-luna-v1','status':'observed_pending_root_acceptance','overlay_usable':usable,'class_count':len(classes),'class_entries':classes,'overlay_classes':overlays,'source':pin(SOURCE),'javac':pin(javac),'pre_run_pins':pins,'post_run_pin_check':'passed','command':rec,'resource_guard':{'minimum_free_bytes':MIN_FREE,'maximum_target_bytes':MAX_TARGET,'maximum_output_bytes':MAX_OUT,'poll_seconds':1,'timeout_seconds':TIMEOUT,'process_group_termination':['SIGTERM','SIGKILL']}}
    (out/'execution.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps({'overlay_usable':usable,'class_entries':classes,'execution':str(out/'execution.json')},ensure_ascii=False))
if __name__=='__main__': main()
