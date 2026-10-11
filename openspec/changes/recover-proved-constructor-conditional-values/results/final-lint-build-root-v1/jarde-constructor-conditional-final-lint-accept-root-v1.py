import sys, json, importlib.util, hashlib, stat
from pathlib import Path
sys.dont_write_bytecode=True
p=Path('/private/tmp/jarde-constructor-conditional-own-ci-verifier-root-v1.py')
assert hashlib.sha256(p.read_bytes()).hexdigest()=='bdd1945f2a6341bdc98da6c764ee95c22c109be36fc0eac1ef7a7cf39ef2bb1d'
s=importlib.util.spec_from_file_location('reviewed_ci',p);v=importlib.util.module_from_spec(s);s.loader.exec_module(v)
r=json.loads(v.EXPECTED_BUILD_RECORD.read_bytes());f=json.loads(v.FOCUSED_RECEIPT.read_bytes());w=json.loads((v.EXPECTED_WORKSPACE/'execution.json').read_bytes())
assert r['status']=='passed_candidate_build_only' and r['rust_min_stack'] is None
assert r['source_pins_before']==r['source_pins_after']==w['source_pins_before']==w['source_pins_after']==f['source_pins_before']==f['source_pins_after']==v.pin_sources(v.source_paths())
assert len(r['source_pins_before'])==4727
assert r['runner_sha256_before']==r['runner_sha256_after']==v.BUILD_RUNNER_SHA256==v.sha_file(v.BUILD_RUNNER)
assert r['accepted_focused_receipt']=={'path':str(v.FOCUSED_RECEIPT),'sha256':v.sha_file(v.FOCUSED_RECEIPT)}
assert r['focused_passed'] and r['final_lint_strict_byte_identity_passed'] and r['candidate_byte_identity'] and r['target_absent'] and not (v.ROOT/'target').exists()
t=(v.ROOT/'.github/workflows/ci.yml').read_text();start=t.index('          cargo clippy');end=t.index('\n\n',start)
args=[['cargo','fmt','--all','--','--check'],t[start:end].split(),['openspec','validate','--all','--strict','--no-interactive'],['cargo','build','-p','jarde-cli','--bin','jarde-cli','--locked'],['cargo','clean']]
assert len(r['commands'])==5
for i,(row,arg) in enumerate(zip(r['commands'],args)):v.check_command(row,i,arg,v.BASE_ENV,v.EXPECTED_BUILD_RECORD.parent)
assert b'Totals: 353 passed, 0 failed (353 items)' in v.verify_stream(r['commands'][2]['streams']['stdout'],v.EXPECTED_BUILD_RECORD.parent)
c=r['candidate_cli'];path=Path(c['path']);assert path==v.EXPECTED_FINAL_CLI and path.stat().st_size==c['bytes']==61286928 and stat.S_IMODE(path.stat().st_mode)==0o555
assert v.sha_file(path)==c['sha256']==f['candidate_cli']['sha256']=='a71c31e80da7c6550aa572e5368f0ecad12ac862066c5d40535f5818c2263241'
result={'status':'accepted-local-final-checks','commands':5,'source_inputs':4727,'strict':[353,0],'build_receipt_sha256':v.sha_file(v.EXPECTED_BUILD_RECORD),'cli':c,'target_absent':True,'peak_target_bytes':max(x['peak_target_bytes'] for x in r['commands']),'own_ci_pending':True}
out=v.EXPECTED_BUILD_RECORD.parent/'root-acceptance.json';assert not out.exists();out.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
