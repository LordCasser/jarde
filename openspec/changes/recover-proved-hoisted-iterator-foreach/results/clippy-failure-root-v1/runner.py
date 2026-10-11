#!/usr/bin/env python3
"""Run guarded final fmt, CI clippy, strict OpenSpec, and byte-identical CLI build."""
from __future__ import annotations
import argparse, datetime, hashlib, importlib.util, json, os, re, runpy, shutil, sys
from pathlib import Path
sys.dont_write_bytecode = True
ROOT = Path('/Users/lordcasser/workspace/projects/jarde').resolve()
PREP = Path('/private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1')
DEFAULT_OUT = PREP/'final-build'
WORKSPACE_RUNNER = PREP/'workspace_runner.py'
WORKSPACE_RUNNER_SHA256 = '711fbd12ba12f8e559340f5dc8bcb3edd6a5dd6b075c9cf54677353565bc506e'
GUARD = ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA256 = '51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCANNER = ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCANNER_SHA256 = 'd954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
RUNTIME_CLI = Path('/private/tmp/jarde-hoisted-iterator-candidate-cli-root-v2')
RUNTIME_CLI_SHA256 = '8bdb868ab58e4c9196970438c71de529996c66ba826c84fe216794d328fdfc9a'
FREE_LIMIT, TARGET_LIMIT = 5*1024**3, 1024**3
SEED = '5350648285461741569'
STRIPPED_ENV_KEYS = ('JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH')


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def load(path: Path):
    return json.loads(path.read_text(encoding='utf-8'))
def need(ok, msg):
    if not ok: raise ValueError(msg)
def load_module(name, path):
    spec=importlib.util.spec_from_file_location(name,path)
    need(spec is not None and spec.loader is not None,'cannot import pinned helper '+str(path))
    module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module); return module

def main() -> int:
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--regression-receipt',type=Path,required=True)
    ap.add_argument('--regression-receipt-sha256',required=True)
    ap.add_argument('--workspace-receipt',type=Path,required=True)
    ap.add_argument('--workspace-receipt-sha256',required=True)
    ap.add_argument('--workspace-acceptance',type=Path,required=True)
    ap.add_argument('--workspace-acceptance-sha256',required=True)
    ap.add_argument('--out',type=Path,default=DEFAULT_OUT)
    args=ap.parse_args()
    out=args.out.resolve()
    need(not out.exists(),'refuse to overwrite existing final-build receipt directory')
    need(sha(GUARD)==GUARD_SHA256 and sha(SCANNER)==SCANNER_SHA256,'guard/scanner pin changed')
    need(sha(WORKSPACE_RUNNER)==WORKSPACE_RUNNER_SHA256,'workspace source-closure helper runner pin changed')
    need(sha(RUNTIME_CLI)==RUNTIME_CLI_SHA256 and RUNTIME_CLI.stat().st_size==61314560 and RUNTIME_CLI.stat().st_mode&0o777==0o555,'runtime-tested CLI pin/mode changed')
    regression_path=args.regression_receipt.resolve(strict=True)
    workspace_path=args.workspace_receipt.resolve(strict=True)
    acceptance_path=args.workspace_acceptance.resolve(strict=True)
    for p,h,label in ((regression_path,args.regression_receipt_sha256,'regression'),(workspace_path,args.workspace_receipt_sha256,'workspace'),(acceptance_path,args.workspace_acceptance_sha256,'workspace acceptance')):
        need(not p.is_symlink() and sha(p)==h,label+' receipt pin changed')
    regression,workspace,acceptance=map(load,(regression_path,workspace_path,acceptance_path))
    need(regression.get('schema')=='hoisted-iterator-regressions-root-v3' and regression.get('status')=='passed_regressions_candidate_only','regression receipt is not the completed candidate-only run')
    need(workspace.get('schema')=='hoisted-iterator-full-workspace-batched-root-v2' and workspace.get('status')=='passed','full workspace receipt did not pass')
    need(acceptance.get('schema')=='hoisted-iterator-full-workspace-local-acceptance-v1' and acceptance.get('status')=='accepted_local_workspace_only','workspace acceptance did not pass')
    need(acceptance.get('evidence',{}).get('execution_sha256')==args.workspace_receipt_sha256,'workspace acceptance does not bind this receipt')
    need(regression.get('target_absent') is True and workspace.get('cleanup',{}).get('target_absent_after') is True and not (ROOT/'target').exists(),'prior validation did not leave target clean')
    closure=load_module('hoisted_iterator_workspace_source_closure',WORKSPACE_RUNNER)
    source_paths=closure.source_paths(); before=closure.pin_sources(source_paths)
    need(before==regression.get('source_pins_before')==regression.get('source_pins_after')==workspace.get('source_pins_before')==workspace.get('source_pins_after'),'current source bytes differ from regression/workspace receipts')
    need(acceptance.get('contract',{}).get('source_pin_count')==len(before),'acceptance source closure count differs')
    cli=regression.get('candidate_cli',{})
    need(cli.get('sha256')==RUNTIME_CLI_SHA256 and cli.get('bytes')==RUNTIME_CLI.stat().st_size and cli.get('mode')=='0o555' and Path(cli.get('path','')).read_bytes()==RUNTIME_CLI.read_bytes(),'regression CLI does not match runtime-tested candidate')
    need(sha(Path(cli['path']))==RUNTIME_CLI_SHA256,'regression candidate CLI changed')
    if (ROOT/'target').exists(): raise SystemExit('fresh-target preflight failed')
    free_before=shutil.disk_usage(ROOT).free
    need(free_before>=FREE_LIMIT,f'5 GiB free-space preflight failed: {free_before}')
    guard=load_module('hoisted_iterator_validation_guard',GUARD)
    scanner=runpy.run_path(str(SCANNER),run_name='hoisted_iterator_target_scan')
    guard.ROOT=ROOT; guard.OUT=out; guard.target_bytes=lambda:scanner['target_bytes'](ROOT)
    guard.FREE_LIMIT=FREE_LIMIT; guard.TARGET_LIMIT=TARGET_LIMIT
    guard.command_stream=lambda p: {"path":str(Path(p).resolve()),"bytes":Path(p).stat().st_size,"sha256":sha(Path(p))}
    env=os.environ.copy(); stripped=[]
    for key in STRIPPED_ENV_KEYS:
        if key in env: env.pop(key); stripped.append(key)
    env.pop('RUST_MIN_STACK',None)
    env.update(guard.ENV_VALUES)
    env['PROPTEST_RNG_SEED']=SEED; env['CARGO_PROFILE_TEST_STRIP']='symbols'
    guard.ENV_VALUES={**guard.ENV_VALUES,'PROPTEST_RNG_SEED':SEED,'CARGO_PROFILE_TEST_STRIP':'symbols'}
    out.mkdir()
    runner_sha=sha(Path(__file__).resolve())
    record={'schema':'hoisted-iterator-final-lint-build-root-v2','status':'running','scope':'final same-source CI fmt/clippy/global strict/build plus byte identity with the previously runtime-tested CLI; local validation only',
            'started_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'regression_receipt':{'path':str(regression_path),'sha256':args.regression_receipt_sha256,'schema':regression['schema'],'status':regression['status']},
            'workspace_receipt':{'path':str(workspace_path),'sha256':args.workspace_receipt_sha256,'schema':workspace['schema'],'status':workspace['status']},
            'workspace_acceptance':{'path':str(acceptance_path),'sha256':args.workspace_acceptance_sha256,'schema':acceptance['schema'],'status':acceptance['status']},
            'runtime_tested_cli':{'path':str(RUNTIME_CLI),'bytes':RUNTIME_CLI.stat().st_size,'sha256':RUNTIME_CLI_SHA256,'mode':'0o555'},
            'source_pins_before':before,'source_pins_after':None,'source_pin_count':len(before),
            'stripped_environment_keys':stripped,'environment_overrides':guard.ENV_VALUES,'rust_min_stack':env.get('RUST_MIN_STACK'),
            'fresh_target_preflight':{'target_absent':True,'free_bytes':free_before},
            'guard_limits':{'minimum_free_bytes':FREE_LIMIT,'maximum_target_bytes':TARGET_LIMIT,'poll_interval_seconds':1},
            'references':[{'path':str(WORKSPACE_RUNNER),'sha256':WORKSPACE_RUNNER_SHA256},{'path':str(GUARD),'sha256':GUARD_SHA256},{'path':str(SCANNER),'sha256':SCANNER_SHA256}],
            'commands':[],'candidate_cli':None,'candidate_byte_identity':None,'target_absent':None,
            'runner':{'path':str(Path(__file__).resolve()),'sha256_before':runner_sha,'sha256_after':None}}
    def save():
        tmp=out/'execution.json.tmp';tmp.write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');tmp.replace(out/'execution.json')
    def assert_source(stage):
        current=closure.pin_sources(closure.source_paths())
        if current!=before: raise RuntimeError('source closure changed at '+stage)
    def run(argv):
        assert_source('before command '+str(len(record['commands'])))
        row=guard.run_command(len(record['commands']),argv,env);record['commands'].append(row);save()
        if row['exit_code']!=0 or row['guard_stop'] is not None: raise RuntimeError('command failed/stopped: '+repr(argv))
        assert_source('after command '+str(row['index']));return row
    cleanup_ok=False
    try:
        ci=(ROOT/'.github/workflows/ci.yml').read_text(encoding='utf-8')
        start=ci.index('          cargo clippy');end=ci.index('\n\n',start)
        lint=ci[start:end].split()
        need(lint[:7]==['cargo','clippy','--workspace','--all-targets','--all-features','--locked','--'] and lint[-2:]==['-D','warnings'],'CI clippy command changed from the pinned workflow contract')
        run(['cargo','fmt','--all','--','--check'])
        run(lint)
        # Keep global strict validation invocation exact and let its actual stdout establish current counts.
        strict=run(['openspec','validate','--all','--strict','--no-interactive'])
        record['global_strict_observation']={'stdout_sha256':strict['streams']['stdout']['sha256'],'stderr_sha256':strict['streams']['stderr']['sha256'],'exit_code':strict['exit_code']}
        run(['cargo','build','-p','jarde-cli','--bin','jarde-cli','--locked'])
        built=ROOT/'target/debug/jarde-cli'
        need(built.is_file() and not built.is_symlink(),'built CLI executable missing or symlink')
        built_bytes=built.read_bytes();baseline_bytes=RUNTIME_CLI.read_bytes()
        candidate={'path':str(RUNTIME_CLI),'bytes':len(built_bytes),'sha256':hashlib.sha256(built_bytes).hexdigest(),'mode':oct(RUNTIME_CLI.stat().st_mode&0o777)}
        byte_identity=built_bytes==baseline_bytes and candidate['sha256']==RUNTIME_CLI_SHA256 and len(built_bytes)==61314560
        record['candidate_cli']=candidate;record['candidate_byte_identity']=byte_identity
        if not byte_identity: raise RuntimeError('freshly built CLI is not byte-identical to runtime-tested candidate')
        record['status']='passed_candidate_build_only';record['final_lint_strict_byte_identity_passed']=True;save()
    except BaseException as exc:
        record['status']='failed';record['error']=f'{type(exc).__name__}: {exc}';save()
    finally:
        try:
            clean=guard.run_command(len(record['commands']),['cargo','clean'],env);record['commands'].append(clean)
            record['target_absent']=not (ROOT/'target').exists()
            record['source_pins_after']=closure.pin_sources(closure.source_paths())
            record['runner']['sha256_after']=sha(Path(__file__).resolve())
            cleanup_ok=(clean['exit_code']==0 and clean['guard_stop'] is None and record['target_absent'] and record['source_pins_after']==before and record['runner']['sha256_after']==runner_sha)
            record['cleanup']={'status':'passed' if cleanup_ok else 'failed','command':clean,'target_absent_after':record['target_absent'],'source_pins_unchanged':record['source_pins_after']==before}
            if not cleanup_ok: record['status']='failed';record.setdefault('error','final guarded cleanup/source postcondition failed')
        except BaseException as exc:
            record['status']='failed';record['cleanup']={'status':'failed-before-completion','error':f'{type(exc).__name__}: {exc}'}
            record['target_absent']=not (ROOT/'target').exists();record['source_pins_after']=closure.pin_sources(closure.source_paths());record['runner']['sha256_after']=sha(Path(__file__).resolve())
            record.setdefault('error','final cleanup failed')
        record['finished_at']=datetime.datetime.now(datetime.timezone.utc).isoformat();save()
    print(json.dumps({'status':record['status'],'commands':len(record['commands']),'candidate_byte_identity':record.get('candidate_byte_identity'),'source_pin_count':record.get('source_pin_count'),'target_absent':record.get('target_absent'),'cleanup_status':record.get('cleanup',{}).get('status')}))
    return 0 if record['status']=='passed_candidate_build_only' and cleanup_ok else 1
if __name__=='__main__': raise SystemExit(main())
