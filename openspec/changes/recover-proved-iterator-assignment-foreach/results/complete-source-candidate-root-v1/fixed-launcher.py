import sys,hashlib,json,subprocess,importlib.util
from pathlib import Path
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde')
O=Path('/private/tmp/jarde-iterator-assignment-fixed-candidate-root-v1')
P=Path('/private/tmp/jarde-cf11-testnestedloops-inputs-root-v1')
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert not O.exists() and not (R/'target').exists()
assert H(P/'manifest.json')=='1227d9a427465af6249753fbf7139c710de2fe15138b1710b389602616e3cf59'
manifest=json.loads((P/'manifest.json').read_text())
for e in manifest['target_classes']+[manifest['runner']]:
 f=P/e['source'];assert f.stat().st_size==e['bytes'] and H(f)==e['sha256']
closure=Path('/private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/workspace_runner.py')
assert H(closure)=='711fbd12ba12f8e559340f5dc8bcb3edd6a5dd6b075c9cf54677353565bc506e'
spec=importlib.util.spec_from_file_location('closure',closure);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
final=Path('/private/tmp/jarde-iterator-assignment-regressions-root-v1/execution.json')
build=json.loads(final.read_text());assert build['status']=='passed_candidate_build_only';before=m.pin_sources(m.source_paths() | set((R/'examples').rglob('*.rs')));assert len(before)==4744 and before==build['source_pins_before']==build['source_pins_after']
collector=Path('/private/tmp/jarde-for-condition-collector-root-v3/collect_controls.py')
checker=Path('/private/tmp/jarde-for-condition-checker-root-v7/check_controls.py')
assert H(collector)=='f944a892f4920feaff9631822bddf4dd9e49a0e8349ec4e09e4a16046257a7ac'
assert H(checker)=='a05acce687bf0e22bfab9eb8ae00520d53a006b740732f4b0041182ce72a464a'
config_file=Path('/private/tmp/jarde-hoisted-iterator-current-baseline-root-v1/execution.json')
assert H(config_file)=='4f3322219d09505fe9b81f5753d1c4d0996145581b0e81538d40f39b0d73d22a'
cfg=json.loads(config_file.read_text())['collector_config']
keys=['cli','cli_sha256','build_receipt','build_receipt_sha256','receipt_cli_object','receipt_status_key','receipt_accepted_value','receipt_mode_key','jdk_manifest','jdk_manifest_sha256','jadx','jadx_sha256','jadx_jar','jadx_jar_sha256','guard','guard_sha256','scanner','scanner_sha256','classfile_parser','classfile_parser_sha256']
cfg={k:cfg[k] for k in keys}
cfg.update(inputs_dir=str(P),inputs_manifest=str(P/'manifest.json'),out=str(O/'capture'),cli=build['candidate_cli']['path'],cli_sha256=build['candidate_cli']['sha256'],build_receipt=str(final),build_receipt_sha256=H(final))
argv=[sys.executable,str(collector)]
for k,v in cfg.items():argv.extend(['--'+k.replace('_','-'),str(v)])
O.mkdir()
x={'schema':'iterator-assignment-candidate-launch-v1','status':'running','source_pins_before':before,'source_closure_helper':{'path':str(closure),'sha256':H(closure)},'collector_config':cfg,'collector_argv':argv,'runner':{'path':str(Path(__file__).resolve()),'sha256':H(__file__)}}
def save():(O/'execution.json').write_text(json.dumps(x,indent=2)+'\n')
save()
try:
 c=subprocess.run(argv,cwd=R);x['collector_exit_code']=c.returncode;save()
 assert c.returncode==0
 a=[sys.executable,str(checker),'--execution',str(O/'capture/execution.json'),'--out',str(O/'checker.json')]
 x['checker_argv']=a;c=subprocess.run(a,cwd=R);x['checker_exit_code']=c.returncode;save();assert c.returncode==0
 x['status']='checked_observations_only'
finally:
 x.update(source_pins_after=m.pin_sources(m.source_paths() | set((R/'examples').rglob('*.rs'))),target_absent=not (R/'target').exists());save()
assert before==x['source_pins_after'] and x['target_absent']
print(json.dumps({'status':x['status'],'sources':len(before),'target_absent':x['target_absent']}))
