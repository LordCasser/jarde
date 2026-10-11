from pathlib import Path
import json,hashlib,subprocess,sys,shutil
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde');P=Path('/private/tmp/jarde-for-condition-collector-root-v3');src=Path('/private/tmp/jarde-for-condition-numeric-inputs-luna-v2');I=Path('/private/tmp/jarde-for-condition-numeric-inputs-root-v1');I.mkdir()
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
rows=json.loads((src/'pins.json').read_text())['sourcefiles']
for x in rows:
 assert H(src/x['source'])==x['sha256'] and (src/x['source']).stat().st_size==x['bytes'];shutil.copyfile(src/x['source'],I/x['source']);(I/x['source']).chmod(0o444)
manifest={'package':'cf11','source_level':'Java 8','target_classes':[x for x in rows if x['class']!='cf11.Runner'],'runner':next(x for x in rows if x['class']=='cf11.Runner')};(I/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n');(I/'manifest.json').chmod(0o444)
review=json.loads(Path('/private/tmp/jarde-for-condition-collector-root-review-v3.json').read_text());assert all(H(P/k)==v for k,v in review['files'].items())
old=json.loads(Path('/private/tmp/jarde-cf11-testnestedloops2-baseline-root-v1/execution.json').read_text())['collector_config']
files=[p for base in ['src','crates','tests'] for p in (R/base).rglob('*') if p.is_file() and (p.suffix=='.rs' or 'fixtures' in p.parts)];before={str(p.relative_to(R)):H(p) for p in files};assert not (R/'target').exists()
args=[sys.executable,str(P/'collect_controls.py'),'--inputs-dir',str(I),'--inputs-manifest',str(I/'manifest.json'),'--out','/private/tmp/jarde-for-condition-numeric-baseline-root-v1']
for key in ['cli','cli_sha256','build_receipt','build_receipt_sha256','jdk_manifest','jdk_manifest_sha256','jadx','jadx_sha256','jadx_jar','jadx_jar_sha256','guard','guard_sha256','scanner','scanner_sha256','classfile_parser','classfile_parser_sha256']:args+=['--'+key.replace('_','-'),old[key]]
print(json.dumps({'scope':'historical accepted v10 CLI NumericComparison condition baseline; not the uncommitted candidate','argv':args}),flush=True)
p=subprocess.run(args,cwd=R);assert {str(x.relative_to(R)):H(x) for x in files}==before and not (R/'target').exists()
print(json.dumps({'collector_exit':p.returncode,'current_live_inputs_unchanged':True,'target_absent':True}),flush=True);sys.exit(p.returncode)
