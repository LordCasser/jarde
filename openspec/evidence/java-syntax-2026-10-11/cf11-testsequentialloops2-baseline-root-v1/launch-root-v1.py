from pathlib import Path
import json,hashlib,subprocess,sys,shutil
sys.dont_write_bytecode=True
R=Path('/Users/lordcasser/workspace/projects/jarde');P=Path('/private/tmp/jarde-sequential-loops2-collector-prep-luna-v1');I=Path('/private/tmp/jarde-sequential-loops2-inputs-root-v1');I.mkdir()
H=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
for row in json.loads((P/'pins.json').read_text())['files'].values():assert H(row['path'])==row['sha256']
manifest=json.loads((P/'inputs_manifest.json').read_text());src=Path('/private/tmp/jarde-cf11-sequential-loops2-input-luna-v2')
up=json.loads((src/'pins.json').read_text());data=Path(up['upstream_source']).read_bytes();assert hashlib.sha256(data).hexdigest()==up['upstream_source_sha256']
for key in ['field','method']:
 q=up[key];part=data[q['byte_offset_start']:q['byte_offset_end_exclusive']];assert hashlib.sha256(part).hexdigest()==q['sha256'] and part in (src/'UpstreamSequentialLoops2.java').read_bytes()
for x in manifest['target_classes']+[manifest['runner']]:
 assert H(src/x['source'])==x['sha256'] and (src/x['source']).stat().st_size==x['bytes'];shutil.copyfile(src/x['source'],I/x['source']);(I/x['source']).chmod(0o444)
shutil.copyfile(P/'inputs_manifest.json',I/'manifest.json');(I/'manifest.json').chmod(0o444)
old=json.loads(Path('/private/tmp/jarde-cf11-testnestedloops2-baseline-root-v1/execution.json').read_text())['collector_config']
files=[p for base in ['src','crates','tests'] for p in (R/base).rglob('*') if p.is_file() and (p.suffix=='.rs' or 'fixtures' in p.parts)]
before={str(p.relative_to(R)):H(p) for p in files};assert not (R/'target').exists()
args=[sys.executable,str(P/'collect_sequential_loops2.py'),'--inputs-dir',str(I),'--inputs-manifest',str(I/'manifest.json'),'--out','/private/tmp/jarde-cf11-sequential-loops2-baseline-root-v1']
for key in ['cli','cli_sha256','build_receipt','build_receipt_sha256','jdk_manifest','jdk_manifest_sha256','jadx','jadx_sha256','jadx_jar','jadx_jar_sha256','guard','guard_sha256','scanner','scanner_sha256','classfile_parser','classfile_parser_sha256']:args+=['--'+key.replace('_','-'),old[key]]
print(json.dumps({'scope':'historical accepted v10 CLI baseline; current uncommitted ForHeader candidate is not this CLI','argv':args}),flush=True)
p=subprocess.run(args,cwd=R);assert {str(x.relative_to(R)):H(x) for x in files}==before and not (R/'target').exists()
print(json.dumps({'collector_exit':p.returncode,'current_live_inputs_unchanged':True,'target_absent':True}),flush=True);sys.exit(p.returncode)
