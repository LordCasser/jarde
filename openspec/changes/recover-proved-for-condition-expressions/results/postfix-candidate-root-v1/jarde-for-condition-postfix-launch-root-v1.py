from pathlib import Path
import json,hashlib,subprocess,sys,shutil
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde');H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
receipt_path=Path('/private/tmp/jarde-for-condition-focused-root-v3/execution.json');receipt=json.loads(receipt_path.read_text());assert receipt['status']=='passed_candidate_build_only' and receipt['focused_passed'] and receipt['target_absent'] and receipt['source_pins_before']==receipt['source_pins_after']
expected=receipt['source_pins_after']
def unchanged():assert len(expected)==4718 and all(H(R/k)==v for k,v in expected.items()) and not (R/'target').exists()
unchanged()
cli=receipt['candidate_cli'];assert H(cli['path'])==cli['sha256'] and (Path(cli['path']).stat().st_mode&0o777)==0o555
old=json.loads(Path('/private/tmp/jarde-cf11-testnestedloops2-baseline-root-v1/execution.json').read_text())['collector_config'];old.update(cli=cli['path'],cli_sha256=cli['sha256'],build_receipt=str(receipt_path),build_receipt_sha256=H(receipt_path))
prep=Path('/private/tmp/jarde-for-condition-collector-root-v3');review=json.loads(Path('/private/tmp/jarde-for-condition-collector-root-review-v3.json').read_text());assert all(H(prep/k)==v for k,v in review['files'].items())
checker=Path('/private/tmp/jarde-for-condition-checker-root-v5/check_controls.py');assert H(checker)=='d338d6ae9b4bec67f9b2fa76db0f71dc04ea7d75c82f7155805bb8ba6975c51b'
fixed=Path('/private/tmp/jarde-for-condition-fixed-inputs-root-v1');fixed.mkdir();base=Path('/private/tmp/jarde-cf11-testnestedloops2-baseline-root-v1/inputs')
entries=[]
for name,cls in [('UpstreamNestedLoops2.java','cf11.UpstreamNestedLoops2'),('Runner.java','cf11.Runner')]:
 shutil.copyfile(base/name,fixed/name);(fixed/name).chmod(0o444);entries.append({'class':cls,'source':name,'bytes':(fixed/name).stat().st_size,'sha256':H(fixed/name)})
(fixed/'manifest.json').write_text(json.dumps({'package':'cf11','source_level':'Java 8','target_classes':[entries[0]],'runner':entries[1]},indent=2)+'\n');(fixed/'manifest.json').chmod(0o444)
for label,inputs in [('controls','/private/tmp/jarde-for-condition-controls-inputs-root-v1'),('numeric','/private/tmp/jarde-for-condition-numeric-inputs-root-v1'),('fixed','/private/tmp/jarde-for-condition-fixed-inputs-root-v1')]:
 out=Path('/private/tmp/jarde-for-condition-'+label+'-postfix-root-v1');args=[sys.executable,str(prep/'collect_controls.py'),'--inputs-dir',inputs,'--inputs-manifest',str(Path(inputs)/'manifest.json'),'--out',str(out)]
 for key in ['cli','cli_sha256','build_receipt','build_receipt_sha256','jdk_manifest','jdk_manifest_sha256','jadx','jadx_sha256','jadx_jar','jadx_jar_sha256','guard','guard_sha256','scanner','scanner_sha256','classfile_parser','classfile_parser_sha256']:args+=['--'+key.replace('_','-'),old[key]]
 print(json.dumps({'stage':label,'argv':args}),flush=True);p=subprocess.run(args,cwd=R);assert p.returncode==0;unchanged()
 args=[sys.executable,str(checker),'--execution',str(out/'execution.json'),'--out',str(out/'observations-root-v1.json')];print(json.dumps({'stage':label+'-check','argv':args}),flush=True);p=subprocess.run(args,cwd=R);assert p.returncode==0;unchanged()
print(json.dumps({'status':'three_postfix_captures_verified_observations_only','source_count':4718,'source_unchanged':True,'target_absent':True}),flush=True)
