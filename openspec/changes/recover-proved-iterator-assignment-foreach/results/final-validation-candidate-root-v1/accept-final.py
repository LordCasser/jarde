#!/usr/bin/env python3
"""Independently verify the final local lint/strict/build and cleanup receipt."""
from __future__ import annotations
import argparse, hashlib, json, os, re, sys, importlib.util
from pathlib import Path
sys.dont_write_bytecode=True
ROOT=Path('/Users/lordcasser/workspace/projects/jarde').resolve()
PREP=Path('/private/tmp/jarde-iterator-assignment-full-validation-prep-luna-v2')
WORKSPACE_RUNNER=PREP/'workspace_runner.py'
WORKSPACE_RUNNER_SHA256='63e52b55fe5d20d5f2ece98ac80676ccdb2de8dfd33d7199b688e075fb1e9673'
WORKSPACE_ACCEPTOR=PREP/'accept_workspace.py'
WORKSPACE_ACCEPTOR_SHA256='06d84b99fcd99e97040b35f9e5397061b41c1f0fede3443256cfc7c656ac8f78'
FINAL_RUNNER=Path('/private/tmp/jarde-iterator-assignment-final-validation-prep-luna-v2/final-build-runner.py')
FINAL_RUNNER_SHA256='70c30e97eb233b7007714c3eda0247c8da381d51940d5db82fd455d1d57a81b0'
GUARD=ROOT/'openspec/changes/preserve-proved-loop-latch-origins/results/run-validation-build-root-v9.py'
GUARD_SHA256='51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33'
SCANNER=ROOT/'openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py'
SCANNER_SHA256='d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7'
CLI=Path('/private/tmp/jarde-iterator-assignment-candidate-cli-root-v1')
CLI_SHA256='526e2e5a6db8a798a92d491123264bbb4d7105c0c1c3c2c9495c1b316eb89dd5'
FREE_LIMIT,TARGET_LIMIT=5*1024**3,1024**3
ENV={'CARGO_BUILD_JOBS':'1','CARGO_INCREMENTAL':'0','CARGO_PROFILE_DEV_DEBUG':'0','CARGO_PROFILE_TEST_DEBUG':'0','RUST_TEST_THREADS':'1','CARGO_TERM_COLOR':'always','PROPTEST_RNG_SEED':'5350648285461741569','CARGO_PROFILE_TEST_STRIP':'symbols'}
ALLOWED_STRIPPED={'JAVA_TOOL_OPTIONS','_JAVA_OPTIONS','JDK_JAVA_OPTIONS','CLASSPATH'}
def sha(raw):return hashlib.sha256(raw).hexdigest()
def pin(path):
 raw=Path(path).read_bytes();return {'path':str(Path(path).resolve()),'bytes':len(raw),'sha256':sha(raw)}
def load(path):return json.loads(Path(path).read_text(encoding='utf-8'))
def need(ok,msg):
 if not ok: raise ValueError(msg)
def module(path):
 spec=importlib.util.spec_from_file_location('hoisted_workspace_closure',path);need(spec and spec.loader,'cannot load workspace source helper');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m

def main():
 ap=argparse.ArgumentParser(description=__doc__)
 for name in ('final-receipt','regression-receipt','workspace-receipt','workspace-acceptance'):
  ap.add_argument('--'+name,type=Path,required=True)
 for name in ('final-receipt','regression-receipt','workspace-receipt','workspace-acceptance'):
  ap.add_argument('--'+name+'-sha256',required=True)
 args=ap.parse_args()
 final_path=args.final_receipt.resolve(strict=True);reg_path=args.regression_receipt.resolve(strict=True);ws_path=args.workspace_receipt.resolve(strict=True);wa_path=args.workspace_acceptance.resolve(strict=True)
 for p,h,label in ((final_path,args.final_receipt_sha256,'final build'),(reg_path,args.regression_receipt_sha256,'regression'),(ws_path,args.workspace_receipt_sha256,'workspace'),(wa_path,args.workspace_acceptance_sha256,'workspace acceptance')):
  need(not p.is_symlink() and sha(p.read_bytes())==h,label+' receipt pin differs')
 need(sha(WORKSPACE_RUNNER.read_bytes())==WORKSPACE_RUNNER_SHA256 and sha(WORKSPACE_ACCEPTOR.read_bytes())==WORKSPACE_ACCEPTOR_SHA256 and sha(FINAL_RUNNER.read_bytes())==FINAL_RUNNER_SHA256,'runner/acceptor pin differs')
 need(sha(GUARD.read_bytes())==GUARD_SHA256 and sha(SCANNER.read_bytes())==SCANNER_SHA256,'guard/scanner pin differs')
 need(pin(CLI)['sha256']==CLI_SHA256 and pin(CLI)['bytes']==61314560 and CLI.stat().st_mode&0o777==0o555,'runtime CLI identity differs')
 f,r,w,a=map(load,(final_path,reg_path,ws_path,wa_path))
 need(f.get('schema')=='iterator-assignment-final-lint-build-root-v2' and f.get('status')=='passed_candidate_build_only' and f.get('final_lint_strict_byte_identity_passed') is True,'final validation receipt not passed')
 need(r.get('schema')=='iterator-assignment-regressions-root-v1' and r.get('status')=='passed_candidate_build_only','regression receipt not passed')
 need(w.get('schema')=='iterator-assignment-full-workspace-root-v2' and w.get('status')=='passed','workspace receipt not passed')
 need(a.get('schema')=='iterator-assignment-full-workspace-local-acceptance-v2' and a.get('status')=='accepted_local_workspace_only' and a.get('evidence',{}).get('execution_sha256')==args.workspace_receipt_sha256 and a.get('contract',{}).get('cleanup_only_exception')=='exact cargo clean bypasses only free-space precheck and monitor abort; all other commands keep both guards','workspace acceptance not bound to the cleanup-only contract')
 need(f.get('regression_receipt')=={'path':str(reg_path),'sha256':args.regression_receipt_sha256,'schema':r['schema'],'status':r['status']},'final regression input binding differs')
 need(f.get('workspace_receipt')=={'path':str(ws_path),'sha256':args.workspace_receipt_sha256,'schema':w['schema'],'status':w['status']},'final workspace input binding differs')
 need(f.get('workspace_acceptance')=={'path':str(wa_path),'sha256':args.workspace_acceptance_sha256,'schema':a['schema'],'status':a['status']},'final workspace acceptance binding differs')
 closure=module(WORKSPACE_RUNNER);live=closure.pin_sources(closure.source_paths())
 need(len(live)==4744 and live==r['source_pins_before']==r['source_pins_after']==w['source_pins_before']==w['source_pins_after']==f['source_pins_before']==f['source_pins_after'],'live source closure differs across the three validations')
 need(f['source_pin_count']==len(live)==a['contract']['source_pin_count'],'source closure count differs')
 need(f.get('candidate_byte_identity') is True and f.get('candidate_cli')=={'path':str(CLI),'bytes':61314560,'sha256':CLI_SHA256,'mode':'0o555'},'fresh CLI did not exactly match the already focused-build candidate')
 need(r.get('candidate_cli',{}).get('sha256')==CLI_SHA256 and Path(r['candidate_cli']['path']).read_bytes()==CLI.read_bytes(),'regression candidate CLI differs')
 rows=f.get('commands',[])
 need(f.get('target_absent') is True and f.get('cleanup',{}).get('status')=='passed' and f.get('cleanup',{}).get('cleanup_only') is True and f.get('cleanup',{}).get('guarded') is False and f.get('cleanup',{}).get('command')==rows[-1] and not (ROOT/'target').exists(),'final cleanup-only clean/target postcondition failed')
 need(f.get('rust_min_stack') is None and f.get('environment_overrides')==ENV and set(f.get('stripped_environment_keys',[]))<=ALLOWED_STRIPPED,'final Rust stack/environment differs')
 need(f.get('guard_limits')=={'minimum_free_bytes':FREE_LIMIT,'maximum_target_bytes':TARGET_LIMIT,'poll_interval_seconds':1},'final resource guard settings differ')
 need(len(rows)==5 and [x.get('index') for x in rows]==list(range(5)),'final command count/order differs')
 ci=(ROOT/'.github/workflows/ci.yml').read_text(encoding='utf-8');start=ci.index('          cargo clippy');end=ci.index('\n\n',start);clippy=ci[start:end].split()
 expected=[['cargo','fmt','--all','--','--check'],clippy,['openspec','validate','--all','--strict','--no-interactive'],['cargo','build','-p','jarde-cli','--bin','jarde-cli','--locked'],['cargo','clean']]
 need(clippy[:7]==['cargo','clippy','--workspace','--all-targets','--all-features','--locked','--'] and clippy[-2:]==['-D','warnings'],'CI clippy command differs')
 raw_paths=set()
 for i,(row,argv) in enumerate(zip(rows,expected)):
  need(row.get('index')==i and row.get('argv')==argv and row.get('cwd')==str(ROOT),'final actual argv/cwd differs')
  need(row.get('exit_code')==0 and row.get('guard_stop') is None,'final command failed/stopped')
  if i==len(expected)-1:
   proof=row.get('cleanup_only',{})
   need(argv==['cargo','clean'] and proof.get('exact_argv')==['cargo','clean'] and
        proof.get('space_precheck_bypassed') is True and proof.get('space_monitor_abort_bypassed') is True and
        type(proof.get('observed_before',{}).get('target_bytes')) is int and
        type(proof.get('observed_before',{}).get('free_bytes')) is int and
        type(proof.get('observed_after',{}).get('target_bytes')) is int and
        type(proof.get('observed_after',{}).get('free_bytes')) is int and
        row.get('free_bytes_after')==proof.get('observed_after',{}).get('free_bytes') and
        row.get('peak_target_bytes',-1)>=max(proof.get('observed_before',{}).get('target_bytes',0),proof.get('observed_after',{}).get('target_bytes',0)),
        'final cleanup-only exception is not bound to exact cargo clean')
  else:
   need(row.get('cleanup_only') is None and row.get('peak_target_bytes',TARGET_LIMIT+1)<=TARGET_LIMIT and row.get('free_bytes_after',0)>=FREE_LIMIT,'final command resource guard differs')
  need(row.get('env_overrides')==ENV,'final command environment differs')
  need(set(row.get('streams',{}))=={'stdout','stderr'},'final command lacks raw streams')
  for stream,ref in row['streams'].items():
   p=Path(ref['path']);need(not p.is_symlink() and p.is_file(),'raw stream missing/symlink')
   raw=p.read_bytes();need(len(raw)==ref['bytes'] and sha(raw)==ref['sha256'],'raw stream pin differs')
   need(p.resolve().is_relative_to(final_path.parent.resolve()),'raw stream escaped final receipt directory')
   raw_paths.add(p.resolve())
 need(len(raw_paths)==10,'final raw stream inventory differs')
 strict=rows[2]
 need(f.get('global_strict_observation')=={'stdout_sha256':strict['streams']['stdout']['sha256'],'stderr_sha256':strict['streams']['stderr']['sha256'],'exit_code':0},'global strict observation does not bind actual command streams')
 refs={x.get('path'):x.get('sha256') for x in f.get('references',[])}
 need(refs.get(str(WORKSPACE_RUNNER))==WORKSPACE_RUNNER_SHA256 and refs.get(str(WORKSPACE_ACCEPTOR))==WORKSPACE_ACCEPTOR_SHA256,'final receipt does not pin v2 workspace runner and acceptor')
 need(f['runner'].get('path')==str(FINAL_RUNNER) and f['runner'].get('sha256_before')==FINAL_RUNNER_SHA256==f['runner'].get('sha256_after'),'final runner bytes changed during run')
 result={'schema':'iterator-assignment-final-validation-acceptance-v2','status':'accepted_local_full_validation_only','regression_receipt_sha256':args.regression_receipt_sha256,'workspace_receipt_sha256':args.workspace_receipt_sha256,'workspace_acceptance_sha256':args.workspace_acceptance_sha256,'final_receipt_sha256':args.final_receipt_sha256,'source_pin_count':len(live),'workspace_target_count':a['contract']['metadata_target_count'],'workspace_actual_totals':a['actual_workspace_totals'],'final_commands':[{'index':x['index'],'argv':x['argv'],'exit_code':x['exit_code'],'stdout_sha256':x['streams']['stdout']['sha256'],'stderr_sha256':x['streams']['stderr']['sha256']} for x in rows],'candidate_cli_byte_identical':True,'target_absent':True,'scope':'Local full-workspace and final-validation receipts only; not CI, release, or publication acceptance.'}
 print(json.dumps(result,ensure_ascii=False,indent=2));return 0
if __name__=='__main__':
 try:raise SystemExit(main())
 except Exception as exc:print('final validation acceptance failed: '+str(exc),file=sys.stderr);raise
