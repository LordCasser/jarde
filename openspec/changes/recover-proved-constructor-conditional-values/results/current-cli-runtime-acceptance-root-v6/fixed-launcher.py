#!/usr/bin/env python3
import importlib.util,json,hashlib,subprocess,sys
from pathlib import Path
BASE=Path('/private/tmp/jarde-cf11-testnestedloops-baseline-launch-root-v1.py')
assert hashlib.sha256(BASE.read_bytes()).hexdigest()=='16bb2b1286585a30a735440b5e08a0100318006fea390792d4948a61b17dc74d'
spec=importlib.util.spec_from_file_location('baseline_tools',BASE); m=importlib.util.module_from_spec(spec); spec.loader.exec_module(m)
m.CLI=Path('/private/tmp/jarde-constructor-conditional-candidate-cli-root-v6');m.CLI_SHA256='a71c31e80da7c6550aa572e5368f0ecad12ac862066c5d40535f5818c2263241'
m.BUILD_RECEIPT=Path('/private/tmp/jarde-constructor-conditional-focused-build-root-v6/execution.json');m.BUILD_RECEIPT_SHA256='4e3260ecfd829cc3ab139552918cc5fc2e119f259338aa3a400b77e37c91ad67'
m.CHECKER=Path('/private/tmp/jarde-for-condition-checker-root-v7/check_controls.py');m.CHECKER_SHA256='a05acce687bf0e22bfab9eb8ae00520d53a006b740732f4b0041182ce72a464a'
INPUTS=Path('/private/tmp/jarde-constructor-conditional-fixed-inputs-root-v1'); OUT=Path('/private/tmp/jarde-constructor-conditional-fixed-candidate-root-v6'); RECEIPT=Path('/private/tmp/jarde-constructor-conditional-fixed-candidate-launch-receipt-root-v6.json')
assert not OUT.exists() and not RECEIPT.exists() and not (m.ROOT/'target').exists()
frozen=INPUTS/'frozen-pins.json'; assert m.sha256(frozen)=='3518626c614b6991687f128a1df8ae35977ce9be9adc06a1b1145e428872990a'
f=m.load_json(frozen)
for row in f['files']+[f['preparation']]:
 q=Path(row['path']); assert m.sha256(q)==row['sha256'] and (q.stat().st_mode & 0o777)==0o444
 if 'bytes' in row: assert q.stat().st_size==row['bytes']
for path,sha,label in [(m.COLLECTOR,m.COLLECTOR_SHA256,'collector'),(m.CHECKER,m.CHECKER_SHA256,'checker'),(m.CLI,m.CLI_SHA256,'cli'),(m.BUILD_RECEIPT,m.BUILD_RECEIPT_SHA256,'build'),(m.BASELINE_CONFIG_RECEIPT,m.BASELINE_CONFIG_SHA256,'old config')]: m.require_pin(path,sha,label)
build=m.load_json(m.BUILD_RECEIPT); source_spec=importlib.util.spec_from_file_location('new_closure',m.SOURCE_CLOSURE_HELPER); source_module=importlib.util.module_from_spec(source_spec);source_spec.loader.exec_module(source_module);before=source_module.pin_sources(source_module.source_paths())
assert before==build['source_pins_before']==build['source_pins_after']
cfg=dict(m.load_json(m.BASELINE_CONFIG_RECEIPT)['collector_config'])
cfg.update(inputs_dir=str(INPUTS),inputs_manifest=str(INPUTS/'manifest.json'),out=str(OUT),cli=str(m.CLI),cli_sha256=m.CLI_SHA256,build_receipt=str(m.BUILD_RECEIPT),build_receipt_sha256=m.BUILD_RECEIPT_SHA256,receipt_cli_object='candidate_cli',receipt_status_key='status',receipt_accepted_value='passed_candidate_build_only',receipt_mode_key='mode')
argv=[sys.executable,str(m.COLLECTOR)]
for name in ('inputs_dir','inputs_manifest','out','cli','cli_sha256','build_receipt','build_receipt_sha256','receipt_cli_object','receipt_status_key','receipt_accepted_value','receipt_mode_key','jdk_manifest','jdk_manifest_sha256','jadx','jadx_sha256','jadx_jar','jadx_jar_sha256','guard','guard_sha256','scanner','scanner_sha256','classfile_parser','classfile_parser_sha256'): argv+=['--'+name.replace('_','-'),str(cfg[name])]
a=subprocess.run(argv,cwd=m.ROOT); checkargv=[sys.executable,str(m.CHECKER),'--execution',str(OUT/'execution.json'),'--out',str(OUT/'checker-root-v7.json')]; b=subprocess.run(checkargv,cwd=m.ROOT) if (OUT/'execution.json').exists() else None
after=source_module.pin_sources(source_module.source_paths()); absent=not (m.ROOT/'target').exists()
r={'schema':'conditional-constructor-candidate-launch-root-v6','collector_argv':argv,'collector_exit_code':a.returncode,'checker_argv':checkargv if b else None,'checker_exit_code':b.returncode if b else None,'source_pins_before':before,'source_pins_after':after,'source_pins_unchanged':before==after,'target_absent_after':absent,'frozen_inputs_sha256':m.sha256(frozen),'status':'complete' if a.returncode==0 and b and b.returncode==0 and before==after and absent else 'observed_failure'}
RECEIPT.write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:r[k] for k in ('status','collector_exit_code','checker_exit_code','source_pins_unchanged','target_absent_after')}));sys.exit(0 if r['status']=='complete' else 1)
