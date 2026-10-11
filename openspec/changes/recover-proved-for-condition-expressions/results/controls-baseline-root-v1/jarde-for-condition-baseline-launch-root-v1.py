#!/usr/bin/env python3
import json,hashlib,os,subprocess,sys
from pathlib import Path
sys.dont_write_bytecode=True
ROOT=Path('/Users/lordcasser/workspace/projects/jarde')
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def closure(receipt):
    expected=receipt['source_pins_before']
    assert expected==receipt['source_pins_after']
    assert len(expected)==4707
    assert all(sha(ROOT/k)==v for k,v in expected.items())
    return len(expected)
old=json.loads(Path('/private/tmp/jarde-cf11-testnestedloops2-baseline-root-v1/execution.json').read_text())['collector_config']
receipt=json.loads(Path(old['build_receipt']).read_text())
head=subprocess.run(['git','rev-parse','HEAD'],cwd=ROOT,capture_output=True,check=True).stdout.decode().strip()
assert head=='593fbcc0b2318fc5cf125aca17a306c276f3e4bb'
assert not (ROOT/'target').exists()
assert sha('/private/tmp/jarde-for-condition-controls-inputs-root-v1/manifest.json')=='99ef0b881f4df678020b9bc99394a5cc2ce409b184e7c74a1eece3b112a7b5ad'
prep=Path('/private/tmp/jarde-for-condition-collector-luna-v2')
pins=json.loads((prep/'pins.json').read_text())
# Root reviewed pins passed separately: reject an unreviewed preparation.
reviewed=json.loads(Path('/private/tmp/jarde-for-condition-collector-root-review-v2.json').read_text())
for name,digest in reviewed['files'].items():assert sha(prep/name)==digest
closure(receipt)
args=[sys.executable,str(prep/'collect_controls.py'),'--inputs-dir','/private/tmp/jarde-for-condition-controls-inputs-root-v1','--inputs-manifest','/private/tmp/jarde-for-condition-controls-inputs-root-v1/manifest.json','--out','/private/tmp/jarde-for-condition-baseline-root-v1']
for key in ['cli','cli_sha256','build_receipt','build_receipt_sha256','jdk_manifest','jdk_manifest_sha256','jadx','jadx_sha256','jadx_jar','jadx_jar_sha256','guard','guard_sha256','scanner','scanner_sha256','classfile_parser','classfile_parser_sha256']:
    args+=['--'+key.replace('_','-'),old[key]]
print(json.dumps({'argv':args,'head':head,'source_count':4707}),flush=True)
r=subprocess.run(args,cwd=ROOT)
closure(receipt)
assert not (ROOT/'target').exists()
print(json.dumps({'collector_exit':r.returncode,'source_closure_restored':True,'target_absent':True}),flush=True)
sys.exit(r.returncode)
