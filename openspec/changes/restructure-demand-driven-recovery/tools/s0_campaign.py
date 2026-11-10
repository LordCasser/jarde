#!/usr/bin/env python3
"""Predeclare and execute a new, indivisible S0 resource campaign. No target execution."""
import argparse
from collections import defaultdict
import gzip
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import signal
import statistics
import subprocess
import time

CHANGE = Path(__file__).resolve().parents[1]
ROOT = CHANGE.parents[2]
OLD = CHANGE / 'results/s0-baseline-v1'
OUT = Path(os.environ.get('JARDE_S0_OUT', str(CHANGE / 'results/s0-observed-v3')))
BINARY = Path(os.environ.get('JARDE_S0_BINARY', '/private/tmp/jarde-s0-observed-bin-20261020/s0_resources'))

def write(p, v):
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(v, ensure_ascii=False, indent=2)+'\n')

def pin(p):
    return dict(path=str(p), bytes=p.stat().st_size, sha256=hashlib.sha256(p.read_bytes()).hexdigest())

def config(label):
    manifest=json.loads((OLD/'manifest.json').read_text())
    if label=='fixture':
        d=OLD/'matrix/E2-dev-javac8-none'
        report=json.loads(gzip.open(d/'recover-jarde.stdout.gz','rt').read())
        method=next(m['item']['identity'] for m in report['methods'] if bytes(m['item']['identity']['name'])==b'run')
        artifact=d/'input.jar'
        roots=[dict(kind='container',origin=method['owner']['location']['entry']['origin'],prefix=[])]
    else:
        artifact=Path(next(a['path'] for a in manifest['artifacts'] if a['label']==label))
        roots=json.loads((OLD/'inventory'/f'{label}.roots.json').read_text())
        with gzip.open(OLD/'bulk'/label/'records.jsonl.gz','rt') as f:
            method=next(x['method'] for line in f if (x:=json.loads(line))['kind']=='method'
                        and x['delivery']['state']=='recovered' and bytes(x['method']['name']) not in [b'<init>',b'<clinit>'])
    env=dict(snapshot=method['owner']['location']['entry']['origin']['snapshot'],
        scope=dict(kind='artifact_tree',root_container='root'),policy=dict(kind='explicit_classpath',roots=roots),
        profile=manifest['runtime_profile'],loader='app')
    # The schema spelling of the CLI is not guessed: EnvironmentPolicy uses explicit_classpath.
    limits=json.loads((OLD/'bulk/bcprov/header.json').read_text())['limits']['total']
    limits.update(elapsed_millis=120000,nested_depth=8,dependency_depth=32)
    return dict(label=label,input=str(artifact),input_pin=pin(artifact),environment=env,method=method,
        limits=limits,method_limits=json.loads((OLD/'bulk/bcprov/header.json').read_text())['limits']['method'],workers=1,fault='normal')

def plan():
    assert not (OUT/'resource-plan.json').exists(), 'a declared campaign is immutable; use a new version'
    configs=[]
    for label in ['fixture','bcprov','s2-009']:
        base=config(label)
        for shape in ['single','class','query','bulk']:
            for cache in (['disabled','cold','hot','tiny'] if label=='fixture' else ['cold']):
                for evidence in (['essential','all'] if label=='fixture' else ['essential']):
                    configs.append(dict(base,shape=shape,cache=cache,evidence=evidence))
    base=config('fixture')
    for fault in ['sink','budget','elapsed','panic','slow','refused']:
        configs.append(dict(base,shape='bulk',cache='cold',evidence='all',workers=2,fault=fault))
    configs.append(dict(config('bcprov'),shape='bulk',cache='cold',evidence='all',workers=2,fault='token'))
    for i,c in enumerate(configs): c['id']=f'{i:02}-{c["label"]}-{c["shape"]}-{c["cache"]}-{c["evidence"]}-{c["fault"]}'
    data=dict(schema='jarde-s0-resource-plan-v3',created_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()),
        engine_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
        measurement_source=pin(CHANGE/'tools/s0_resources.rs'),observation_sources=[pin(p) for p in sorted(ROOT.glob('crates/*/src/observation.rs'))],observation_manifest=pin(OUT/'observation-source-manifest.json'),configs=configs,repeats=10,
        ordering='round-robin, rotate first configuration by round; one fresh process per configuration per round; no mixing old interrupted samples',
        thresholds=dict(time='maximum + max(0.20*median, 3*MAD, 1000us)',
            rss='maximum + max(0.20*median, 3*MAD, 1MiB)',token='same time formula on set-to-return microseconds',
            usage='maximum by dimension and exact request shape; elapsed excluded; billed dimensions are not actual CPU/I/O counters'),
        hard_guards=dict(seconds=180,process_group_rss_bytes=1<<30,free_disk_bytes=5<<30,poll_seconds=.5),
        init='cache disabled: no handle; cold: new roomy cache; hot: identical operation warmup; tiny: 1 entry/4096 bytes; OS page cache not flushed',
        request_scope='real-artifact method/class use fixed physical method owner; query is six first-page requests of two items; bulk is full declared artifact tree',
        evidence='fixture essential/all paired; real core baseline essential; failure probes all; no implicit runtime headers',
        limitations=['Artifact-file read calls/returned bytes are reported at open; request entry wrapped-reader output bytes count Deflate output or Stored bytes; neither means physical disk I/O. Slice parsing makes no stream calls.',
            'Facts/PreparedClass/ContainerFactsHandle and Java Expr/Stmt/declared class candidate payload retention use owned-object counts, not heap bytes or all Arc references. Covered types/lifetimes are pinned; other local records and Region trees are not a heap census.',
            'Pipeline completion includes stopped/partial payload; per-phase completions mean table publication; report execution remains separate.',
            'Token set occurs 1ms after a real Prepared signal on concurrent bcprov bulk; no artificial class delay; report late or missing trigger as invalid.',
            'Fault probes cover their declared shapes only; elapsed probe expires before dispatch; sink stop is independent of token.'],
        censored_observation='WAR bulk emits atomic snapshots every 250ms; last captured peaks are lower bounds before termination, not completed-request peaks. RSS guard remains a failed product baseline.',
        timing='sample.micros is adapter operation window including budget/setup and return-summary; final stdout serialization excluded; open and process wall separate',
        host=dict(platform=platform.platform(),machine=platform.machine(),cpu=os.cpu_count(),load=os.getloadavg()))
    write(OUT/'resource-plan.json',data)
    print(json.dumps(dict(configs=len(configs),processes=len(configs)*10,plan=pin(OUT/'resource-plan.json')),indent=2))

def run_one(c, repeat):
    dest=OUT/'resources'/f'{repeat:02}-{c["id"]}'
    dest.parent.mkdir(parents=True,exist_ok=True)
    env=dict(os.environ)
    for k in ['RUST_MIN_STACK','CLASSPATH','JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS']:env.pop(k,None)
    env['JARDE_S0_CONFIG']=json.dumps(c)
    cmd=[str(BINARY),'--exact','measure_s0','--ignored','--nocapture','--test-threads=1']
    timer=dest.with_suffix('.time')
    began=time.perf_counter();peak=0;guard=None
    with dest.with_suffix('.stdout').open('wb') as out,dest.with_suffix('.stderr').open('wb') as err:
        child=subprocess.Popen(['/usr/bin/time','-l','-o',str(timer)]+cmd,cwd=ROOT,env=env,stdout=out,stderr=err,start_new_session=True)
        try:
            while child.poll() is None:
                ps=subprocess.check_output(['ps','-axo','pgid=,rss='],text=True)
                current=sum(int(r) for line in ps.splitlines() for group,r in [line.split()] if int(group)==child.pid)*1024
                peak=max(peak,current)
                if current>1<<30:guard='rss'
                elif shutil.disk_usage(OUT).free<5<<30:guard='free_disk'
                elif time.perf_counter()-began>180:guard='timeout'
                if guard:
                    os.killpg(child.pid,signal.SIGTERM);break
                try:child.wait(timeout=.5)
                except subprocess.TimeoutExpired:pass
            try:rc=child.wait(timeout=5)
            except subprocess.TimeoutExpired:os.killpg(child.pid,signal.SIGKILL);rc=child.wait()
        finally:
            if child.poll() is None:os.killpg(child.pid,signal.SIGKILL);child.wait()
    timing=timer.read_text() if timer.exists() else ''
    rss=re.search(r'(\d+)\s+maximum resident set size',timing)
    real=re.search(r'([0-9.]+)\s+real',timing)
    samples=[json.loads(s[s.index('{"config":'):]) for s in dest.with_suffix('.stdout').read_text().splitlines() if '{"config":' in s]
    observations=[json.loads(s.split('S0_OBSERVATION ',1)[1]) for s in dest.with_suffix('.stdout').read_text().splitlines() if 'S0_OBSERVATION ' in s]
    row=dict(observation_prefix=observations,config_id=c['id'],repeat=repeat,command=cmd,cwd=str(ROOT),exit_code=rc,guard=guard,
        wall_seconds=float(real[1]) if real else None,wrapper_seconds=time.perf_counter()-began,rss_process_peak_bytes=int(rss[1]) if rss else None,
        sampled_process_group_rss_peak_bytes=peak,load=os.getloadavg(),samples=samples)
    write(dest.with_suffix('.meta.json'),row)
    return row

def run():
    p=json.loads((OUT/'resource-plan.json').read_text())
    assert not (OUT/'resource-raw.json').exists(), 'never resume an interrupted batch by filling samples'
    write(OUT/'resource-binary.json',pin(BINARY))
    rows=[]
    for repeat in range(p['repeats']):
        for index in range(len(p['configs'])):
            c=p['configs'][(repeat+index)%len(p['configs'])]
            r=run_one(c,repeat);rows.append(r);write(OUT/'resource-raw.json',rows)
            print(repeat,c['id'],r['exit_code'],round(r['wall_seconds'] if r['wall_seconds'] is not None else r['wrapper_seconds'],3),flush=True)
        print('round-complete',repeat+1,flush=True)
    summarize()

def stats(values, floor):
    m=statistics.median(values);mad=statistics.median(abs(x-m) for x in values)
    return dict(min=min(values),median=m,max=max(values),mad=mad,upper=max(values)+max(.2*m,3*mad,floor))

def summarize():
    p=json.loads((OUT/'resource-plan.json').read_text());rows=json.loads((OUT/'resource-raw.json').read_text());groups=defaultdict(list)
    for r in rows:groups[r['config_id']].append(r)
    result=[]
    for c in p['configs']:
        a=groups[c['id']];samples=[r['samples'][0] for r in a if r['exit_code']==0 and len(r['samples'])==1 and not r['guard']]
        guarded=[r for r in a if r['guard'] is not None and r['exit_code'] in [-15,-9]]
        row=dict(id=c['id'],attempts=len(a),valid=len(samples),guarded_failures=[{k:r[k] for k in ['repeat','guard','exit_code','sampled_process_group_rss_peak_bytes','wrapper_seconds','observation_prefix']} for r in guarded],collection_complete=len(a)==10 and {r['repeat'] for r in a}==set(range(10)) and len(samples)+len(guarded)==10,complete=len(samples)==10 and {r['repeat'] for r in a}==set(range(10)))
        if samples:
            row['request_micros']=stats([s['sample']['micros'] for s in samples],1000)
            rss=[r['rss_process_peak_bytes'] for r in a if r['rss_process_peak_bytes'] is not None]
            row['rss_bytes']=stats(rss,1<<20) if rss else None
            row['digests']=sorted({s['sample']['semantic_text_digest'] for s in samples})
            keys=set().union(*(s['sample']['usage'].keys() for s in samples))
            row['usage_max']={k:max(s['sample']['usage'][k] for s in samples) for k in keys if k!='elapsed_millis'}
            row['counts']=[s['sample']['counts'] for s in samples]
            row['observed_work']=[s['sample']['observed_work'] for s in samples]
            row['retention']=[s['sample']['retention'] for s in samples]
            row['missing_prepared']=[s['sample']['details'].get('missing_prepared') for s in samples]
            token=[s['sample']['details'].get('token') for s in samples if s['sample']['details'].get('token')]
            if token:row['token']=dict(valid=sum(x['set_before_return'] for x in token),latency_micros=stats([x['return_latency_micros'] for x in token],1000),work_after_set=[x['work_after_set'] for x in token])
        result.append(row)
    write(OUT/'resource-summary.json',dict(plan=pin(OUT/'resource-plan.json'),processes=len(rows),expected=len(p['configs'])*10,collection_complete=all(r['collection_complete'] for r in result),complete=all(r['complete'] for r in result),configs=result))
    print('summary',len(rows),'complete',all(r['complete'] for r in result),flush=True)

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('phase',choices=['plan','run','summarize']);a=p.parse_args()
    {'plan':plan,'run':run,'summarize':summarize}[a.phase]()
