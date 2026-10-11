#!/usr/bin/env python3
"""Build/freeze/clean the final CLI under the existing disk guard; run no tests."""
from __future__ import annotations
import argparse, datetime, hashlib, importlib.util, json, os, re, shutil, stat, sys
from pathlib import Path

sys.dont_write_bytecode=True
ROOT=Path('/Users/lordcasser/workspace/projects/jarde').resolve()
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCAN=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCAN_SHA='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
INCLUDE_RE=re.compile(r'(?:include_bytes|include_str)!\s*\(\s*"([^"]+)"')

def sha(path:Path)->str:
    h=hashlib.sha256()
    with Path(path).open('rb') as f:
        for b in iter(lambda:f.read(1024*1024),b''):h.update(b)
    return h.hexdigest()
def write(out:Path,record:dict):
    tmp=out/'.execution.json.tmp';tmp.write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8');tmp.replace(out/'execution.json')
def load_module(name,path):
    spec=importlib.util.spec_from_file_location(name,path)
    if spec is None or spec.loader is None: raise RuntimeError('cannot load pinned helper: '+str(path))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

def product_paths()->set[str]:
    paths={ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'.github/workflows/ci.yml',ROOT/'tests/fixtures/corpus-fingerprint.json'}
    for base in ('src','crates','tests'):paths.update((ROOT/base).rglob('*.rs'))
    paths.update((ROOT/'crates').glob('*/Cargo.toml'))
    for asset_root in (ROOT/'tests/fixtures',ROOT/'openspec/changes/recover-class-scope-constructor-parameters/evidence/frozen'):
        paths.update(p for p in asset_root.rglob('*') if p.is_file())
    for source in list(paths):
        if source.suffix=='.rs':
            for rel in INCLUDE_RE.findall(source.read_text(encoding='utf-8')):
                child=(source.parent/rel).resolve()
                if not child.is_relative_to(ROOT):raise RuntimeError(f'literal include escapes workspace: {source} -> {rel}')
                if child.is_file():paths.add(child)
    return {p.relative_to(ROOT).as_posix() for p in paths}
def pin_sources(paths:set[str])->dict[str,str]:
    pins={}
    for rel in sorted(paths):
        path=ROOT/rel
        if not path.is_file():raise FileNotFoundError('required source pin missing: '+rel)
        pins[rel]=sha(path)
    return pins

def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--source-base',required=True,help='exact 40-character source base recorded for provenance')
    ap.add_argument('--out',required=True,type=Path,help='new evidence directory')
    ap.add_argument('--cli',required=True,type=Path,help='new frozen CLI destination')
    args=ap.parse_args();out=args.out.resolve();cli=args.cli.resolve()
    if not re.fullmatch(r'[0-9a-f]{40}',args.source_base):raise SystemExit('--source-base must be lowercase 40-hex')
    if out.exists() or cli.exists():raise SystemExit('refusing to overwrite evidence directory or frozen CLI')
    target=ROOT/'target'
    if target.exists():raise SystemExit('expected the fresh workspace cargo clean to leave target absent before final CLI build')
    if sha(GUARD)!=GUARD_SHA or sha(SCAN)!=SCAN_SHA:raise SystemExit('pinned guard/scanner SHA mismatch')
    paths=product_paths();before=pin_sources(paths)
    guard=load_module('final_cli_guard_v9',GUARD);scanner=load_module('final_cli_target_scanner',SCAN)
    guard.ROOT=ROOT;guard.OUT=out;guard.target_bytes=lambda:scanner.target_bytes(ROOT)
    guard.expected_test_summaries=lambda index,raw:None
    guard.command_stream=lambda p:{'path':str(p),'bytes':Path(p).stat().st_size,'sha256':sha(Path(p))}
    out.mkdir(parents=True,exist_ok=False)
    env=os.environ.copy();stripped=[]
    for key in guard.STRIPPED_ENV_KEYS:
        if key in env:env.pop(key);stripped.append(key)
    env.update(guard.ENV_VALUES)
    runner=Path(__file__).resolve();runner_sha=sha(runner)
    record={'schema':'jarde-final-cli-build-clean-root-v1','status':'running','source_base':args.source_base,
      'output_directory':str(out),'frozen_cli_destination':str(cli),'runner':{'path':str(runner),'sha256':runner_sha},
      'guard_template':{'path':str(GUARD),'sha256':GUARD_SHA},'target_size_scan_adapter':{'path':str(SCAN),'sha256':SCAN_SHA},
      'environment_overrides':guard.ENV_VALUES,'stripped_environment_keys':stripped,
      'guard_limits':{'free_bytes':guard.FREE_LIMIT,'target_bytes':guard.TARGET_LIMIT},
      'source_paths_before':sorted(paths),'source_pins_before':{'full_product_source_and_include_closure':before},
      'commands':[],'pre_build_target_absent':True,'product_source_pin_count':len(before)}
    write(out,record);failure=None
    try:
        row=guard.run_command(0,['cargo','build','-p','jarde-cli','--locked'],env);row['phase']='cli-build';record['commands'].append(row);write(out,record)
        if row['exit_code']!=0 or row['guard_stop'] is not None:raise RuntimeError('final CLI build failed or hit resource guard')
        if pin_sources(paths)!=before:raise RuntimeError('full product source/include pins changed during CLI build')
        built=ROOT/'target/debug/jarde-cli'
        if not built.is_file():raise FileNotFoundError('successful build did not produce target/debug/jarde-cli')
        if cli.exists():raise FileExistsError('refusing to overwrite frozen CLI')
        tmp=cli.with_name('.'+cli.name+'.tmp-'+str(os.getpid()))
        if tmp.exists():raise FileExistsError('refusing to overwrite temporary frozen CLI')
        cli.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(built,tmp);os.chmod(tmp,0o555);os.replace(tmp,cli)
        frozen={'path':str(cli),'size_bytes':cli.stat().st_size,'sha256':sha(cli),'mode':oct(stat.S_IMODE(cli.stat().st_mode))}
        record['freeze']={'built_binary':{'path':str(built),'size_bytes':built.stat().st_size,'sha256':sha(built)},
                          'frozen_cli':frozen,'source_base':args.source_base,
                          'product_source_pins':before}
        if frozen['mode']!='0o555' or frozen['size_bytes']!=record['freeze']['built_binary']['size_bytes'] or frozen['sha256']!=record['freeze']['built_binary']['sha256']:
            raise RuntimeError('frozen CLI differs from build output or is not mode 0555')
        record['status']='cli-frozen-before-clean';write(out,record)
    except Exception as exc:
        failure=f'{type(exc).__name__}: {exc}';record.update(status='failed',failure=failure);write(out,record)
    finally:
        try:
            clean=guard.run_command(1,['cargo','clean'],env);clean['phase']='cargo-clean';record['commands'].append(clean)
            record['target_absent_after_clean']=not target.exists()
            after_paths=product_paths();after=pin_sources(after_paths)
            record['source_paths_after']=sorted(after_paths)
            record['source_pins_after']={'full_product_source_and_include_closure':after}
            if clean['exit_code']!=0 or clean['guard_stop'] is not None or not record['target_absent_after_clean']:
                record.update(status='failed',cleanup_error='cargo clean failed, hit guard, or left target present')
            if after_paths!=paths or after!=before:
                record.update(status='failed',cleanup_error='full product source/include pin closure changed during build')
            if record.get('status')=='cli-frozen-before-clean':record['status']='validation-passed-cli-frozen-target-cleaned'
        except Exception as exc:
            record.update(status='failed',cleanup_error=f'{type(exc).__name__}: {exc}')
        record['failure']=failure;record['completed_at_utc']=datetime.datetime.now(datetime.timezone.utc).isoformat();write(out,record)
    print(json.dumps({'status':record['status'],'source_base':args.source_base,'product_source_pin_count':len(before),
                      'frozen_cli':record.get('freeze',{}).get('frozen_cli'),'target_absent_after_clean':record.get('target_absent_after_clean')}))
    return 0 if record['status']=='validation-passed-cli-frozen-target-cleaned' else 1

if __name__=='__main__':raise SystemExit(main())
