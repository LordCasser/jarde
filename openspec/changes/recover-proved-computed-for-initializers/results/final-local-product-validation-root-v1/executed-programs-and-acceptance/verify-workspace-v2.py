#!/usr/bin/env python3
"""Pure read-only verifier for the fresh full-workspace batch evidence."""
from __future__ import annotations
# v2 allows valid zero-test Cargo targets only when the captured artifact and
# metadata-derived exact target inventory still bind every result summary.
import argparse, hashlib, json, re
from pathlib import Path

ROOT = Path('/Users/lordcasser/workspace/projects/jarde')
RUNNER = Path('/private/tmp/jarde-compound-workspace-batches-root-v1.py')
RUNNER_SHA = '2ac64ccdf7ed6c8f90f37b8ae34693eb16206cba493ca4a3e261de993a08235b'
GUARD = ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCAN = ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCAN_SHA = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
MIN_FREE = 5*1024**3
MAX_TARGET = 1024**3
SEED = '5350648285461741569'
OVERRIDES = {'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always','PROPTEST_RNG_SEED':SEED,'CARGO_PROFILE_TEST_STRIP':'symbols'}
STRIPPED = ['JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH']
SUMMARY = re.compile(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;')
INCLUDE = re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')

def sha(b: bytes) -> str: return hashlib.sha256(b).hexdigest()
def file_sha(p: Path) -> str: return sha(p.read_bytes())
def need(ok, msg):
    if not ok: raise AssertionError(msg)
def load(p: Path): return json.loads(p.read_text(encoding='utf-8'))
def expected_source_paths() -> set[str]:
    paths={ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'.github/workflows/ci.yml',ROOT/'tests/fixtures/corpus-fingerprint.json'}
    for base in ('src','crates','tests'):
        paths.update((ROOT/base).rglob('*.rs'))
    paths.update((ROOT/'crates').glob('*/Cargo.toml'))
    for asset_root in (ROOT/'tests/fixtures',ROOT/'openspec/changes/recover-class-scope-constructor-parameters/evidence/frozen'):
        paths.update(p for p in asset_root.rglob('*') if p.is_file())
    for source in list(paths):
        if source.suffix=='.rs':
            for rel in INCLUDE.findall(source.read_text(encoding='utf-8')):
                child=(source.parent/rel).resolve()
                need(child.is_relative_to(ROOT.resolve()),f'literal include escapes repository: {source} -> {rel}')
                if child.is_file(): paths.add(child)
    return {p.relative_to(ROOT).as_posix() for p in paths}
def stream_bytes(base: Path, row: dict, label: str) -> bytes:
    ref=row
    p=Path(ref['path'])
    if not p.is_absolute(): p=base/p
    b=p.read_bytes()
    need(len(b)==ref['bytes'], f'{label}: byte count mismatch')
    need(sha(b)==ref['sha256'], f'{label}: SHA-256 mismatch')
    return b

def reconstruct(md: dict):
    members=set(md['workspace_members']); commands=[]; inventory=[]
    for pkg in md['packages']:
        if pkg['id'] not in members: continue
        base=['cargo','test','-p',pkg['name'],'--all-features','--locked','--message-format','json']
        unit=[t for t in pkg['targets'] if t['kind'][0] in ('lib','bin','example','bench')]
        if unit:
            flags=[]
            for kind,flag in [('lib','--lib'),('bin','--bins'),('example','--examples'),('bench','--benches')]:
                if any(t['kind'][0]==kind for t in unit): flags.append(flag)
            commands.append(base+flags)
            inventory.extend((pkg['name'],t['name'],t['kind'][0]) for t in unit)
        tests=[t for t in pkg['targets'] if t['kind'][0]=='test']
        for start in range(0,len(tests),4):
            batch=tests[start:start+4]
            commands.append(base+sum((['--test',t['name']] for t in batch),[]))
            inventory.extend((pkg['name'],t['name'],'test') for t in batch)
    all_targets=[(p['name'],t['name'],t['kind'][0]) for p in md['packages'] if p['id'] in members for t in p['targets']]
    need(sorted(inventory)==sorted(all_targets) and len(inventory)==len(set(inventory)), 'metadata target inventory is not covered exactly once')
    return commands, all_targets

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--evidence',required=True,type=Path)
    ap.add_argument('--out',required=True,type=Path)
    a=ap.parse_args(); ev=a.evidence.resolve(); out=a.out.resolve(); d=load(ev); base=ev.parent
    need(d.get('schema')=='compound-full-workspace-batched-root-v1','execution schema mismatch')
    need(d.get('status')=='passed',f"run is incomplete/failed: {d.get('status')} {d.get('error')}")
    need(file_sha(RUNNER)==RUNNER_SHA and d['runner']=={'path':str(RUNNER),'sha256':RUNNER_SHA,'derived_from':'openspec/changes/recover-proved-conditional-switch-fallthrough/results/full-workspace-root-v1/jarde-run-conditional-workspace-batches-root-v5.py'},'runner identity mismatch')
    need(file_sha(GUARD)==GUARD_SHA and d['guard_template']=={'path':str(GUARD),'sha256':GUARD_SHA},'guard template identity mismatch')
    need(file_sha(SCAN)==SCAN_SHA and d['target_size_scan_adapter']=={'path':str(SCAN),'sha256':SCAN_SHA},'target scanner identity mismatch')
    need(d.get('seed')==SEED and d.get('no_carried_execution') is True,'seed/no-carried-execution declaration mismatch')
    need(d.get('environment_overrides')==OVERRIDES,'environment override set mismatch')
    need(d.get('guard_limits')=={'free_bytes':MIN_FREE,'target_bytes':MAX_TARGET},'guard limits mismatch')
    need(d.get('pins_unchanged') is True and not d.get('pin_changes') and not d.get('pin_errors'),'source pins changed or unreadable')
    before=d.get('source_pins_before_format'); pins=d.get('source_pins')
    need(isinstance(before,dict) and pins==before and len(pins)>0,'source pin closure changed or empty')
    expected_paths=expected_source_paths()
    need(set(pins)==expected_paths,'source pin path set differs from reconstructed Cargo/CI/corpus/Rust/fixture/include closure')
    for rel,digest in pins.items():
        p=ROOT/rel
        need(p.is_file() and file_sha(p)==digest, f'source pin mismatch: {rel}')
    need(d.get('guard_limits')=={'free_bytes':MIN_FREE,'target_bytes':MAX_TARGET},'guard bounds mismatch')
    fmt=d.get('format_command'); meta=d.get('metadata_command'); clean=d.get('clean_command')
    need(fmt and fmt['argv']==['cargo','fmt','--all','--','--check'] and fmt['exit_code']==0 and fmt['guard_stop'] is None,'format command did not pass exactly')
    need(meta and meta['argv']==['cargo','metadata','--no-deps','--format-version','1','--locked'] and meta['exit_code']==0 and meta['guard_stop'] is None,'metadata command did not pass exactly')
    for label,row in [('format',fmt),('metadata',meta)]:
        need(row.get('env_overrides')==OVERRIDES and row.get('peak_target_bytes',MAX_TARGET+1)<=MAX_TARGET and row.get('free_bytes_after',0)>=MIN_FREE,f'{label}: environment or disk guard mismatch')
        for key in ('stdout','stderr'): stream_bytes(base,row['streams'][key],f'{label} {key}')
    mdbytes=stream_bytes(base,meta['streams']['stdout'],'metadata stdout')
    metadata_path=base/'metadata.json'
    need(metadata_path.is_file() and metadata_path.read_bytes()==mdbytes,'metadata.json does not match captured metadata stdout')
    md=json.loads(mdbytes)
    expected,inventory=reconstruct(md)
    need(d.get('target_inventory')==[list(x) for x in inventory], 'recorded target inventory differs from captured Cargo metadata')
    need(d.get('expected_batch_count')==95 and len(expected)==95,'fresh run is not the expected 95-batch workspace run')
    rows=d.get('commands',[])
    need(len(rows)==95,'workspace evidence must contain all 95 batches')
    need([r.get('index') for r in rows]==list(range(2,97)),'batch indices are not contiguous from raw stream 2')
    mdid={p['name']:p['id'] for p in md['packages']}
    expected_ids={(mdid[p],t,k) for p,t,k in inventory}
    actual_ids=[]; summary_count=0; totals=[0,0,0]; max_target=0; min_free=None
    for row,argv in zip(rows,expected):
        i=row['index']; need(row.get('argv')==argv,f'batch {i}: argv differs from Cargo metadata-derived partition')
        need(row.get('cwd')==str(ROOT),f'batch {i}: cwd mismatch')
        need(row.get('exit_code')==0 and row.get('guard_stop') is None,f'batch {i}: failed or tripped disk guard')
        need(row.get('env_overrides')==OVERRIDES,f'batch {i}: environment mismatch')
        peak=row.get('peak_target_bytes'); free=row.get('free_bytes_after')
        need(isinstance(peak,int) and peak<=MAX_TARGET,f'batch {i}: target cap exceeded')
        need(isinstance(free,int) and free>=MIN_FREE,f'batch {i}: free-space floor violated')
        max_target=max(max_target,peak); min_free=free if min_free is None else min(min_free,free)
        raw=stream_bytes(base,row['streams']['stdout'],f'batch {i} stdout')
        stream_bytes(base,row['streams']['stderr'],f'batch {i} stderr')
        text=raw.decode('utf-8',errors='replace'); counts=[tuple(map(int,x)) for x in SUMMARY.findall(text)]
        need(bool(counts) and all(f==0 for p,f,ig in counts),f'batch {i}: missing or failed test summaries')
        need(row.get('observed_counts')==[list(c) for c in counts],f'batch {i}: stored test counts differ from raw stdout')
        artifacts=[]
        for line in text.splitlines():
            if not line.startswith('{'): continue
            try: artifact=json.loads(line)
            except json.JSONDecodeError: continue
            ex=artifact.get('executable')
            if artifact.get('reason')=='compiler-artifact' and artifact.get('profile',{}).get('test') and ex:
                artifacts.append((artifact['package_id'],artifact['target']['name'],artifact['target']['kind'][0]))
        need(row.get('executed_targets')==[list(x) for x in artifacts],f'batch {i}: recorded executed targets differ from Cargo JSON artifacts')
        need(len(artifacts)==len(set(artifacts))==len(counts),f'batch {i}: test summary/artifact cardinality mismatch')
        actual_ids.extend(artifacts); summary_count+=len(counts)
        for c in counts:
            for j,v in enumerate(c): totals[j]+=v
    need(set(actual_ids)==expected_ids and len(actual_ids)==len(expected_ids),'executed targets do not cover metadata targets exactly once')
    need(summary_count==d.get('workspace_summary_count'),'workspace summary count mismatch')
    need(totals==d.get('workspace_totals') and totals[1]==0,'workspace totals differ from raw batch summaries or contain failures')
    removed=d.get('removed_completed_test_executables',[])
    need(len(removed)==len(actual_ids),'completed test-executable cleanup inventory is incomplete')
    removed_paths=[]
    for row in removed:
        p=Path(row['path']).resolve()
        need(p.is_relative_to(ROOT/'target') and not p.exists() and not p.is_symlink(),'completed test executable remains or escapes target')
        need(row['bytes']>0 and re.fullmatch(r'[0-9a-f]{64}',row['sha256']) is not None,'removed executable identity malformed')
        removed_paths.append(str(p))
    need(len(removed_paths)==len(set(removed_paths)),'duplicate removed executable path')
    need(clean and clean.get('index')==97 and clean['argv']==['cargo','clean'] and clean['exit_code']==0 and clean['guard_stop'] is None,'cargo clean did not pass exactly')
    need(clean.get('env_overrides')==OVERRIDES and clean.get('peak_target_bytes',MAX_TARGET+1)<=MAX_TARGET and clean.get('free_bytes_after',0)>=MIN_FREE,'clean: environment or disk guard mismatch')
    for key in ('stdout','stderr'): stream_bytes(base,clean['streams'][key],f'clean {key}')
    target=ROOT/'target'; target_bytes=sum(p.stat().st_size for p in target.rglob('*') if p.is_file()) if target.exists() else 0
    need(d.get('target_absent_after_clean') is True and not target.exists() and target_bytes==0,'target was not absent after clean')
    report={'status':'accepted','evidence':str(ev),'schema':d['schema'],'batch_count':len(rows),'workspace_summary_count':summary_count,'workspace_totals':totals,'target_inventory_count':len(inventory),'removed_test_executables':len(removed),'source_pin_count':len(pins),'reconstructed_source_pin_count':len(expected_paths),'source_pins_unchanged':True,'source_pins_complete':True,'zero_test_summaries':sum(1 for r in rows for c in r.get('observed_counts',[]) if c[0]==0 and c[2]==0),'min_free_bytes':min_free,'max_target_bytes':max_target,'cargo_clean_succeeded':True,'target_absent':True,'runner_sha256':RUNNER_SHA,'execution_sha256':file_sha(ev)}
    out.parent.mkdir(parents=True,exist_ok=True); out.write_text(json.dumps(report,indent=2,sort_keys=True)+'\n',encoding='utf-8')
    print(json.dumps(report,indent=2,sort_keys=True))

if __name__=='__main__':
    try: main()
    except Exception as exc:
        print(f'NOT ACCEPTED: {type(exc).__name__}: {exc}')
        raise
