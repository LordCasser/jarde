from pathlib import Path
import json,hashlib
E=Path('/private/tmp/jarde-computed-shared-stack-compare-root-v4/execution.json');d=json.loads(E.read_text());v=json.loads((E.parent/'verification.json').read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
assert d['status']=='observed_complete_pending_independent_checker' and len(d['commands'])==20
assert v['limits_and_pins']=='verified' and v['original']['status']==v['jadx']['status']=='matched'
assert len(v['jarde']['issues'])==4 and all(s.endswith('generated class compile failed') for s in v['jarde']['issues'])
rows=[];docs={}
for leg in d['legs']:
 physical=set(next(x['physical_bcis'] for x in v['original']['shared_bytecode'] if x['jdk']==leg['jdk']))
 for r in leg['renders']:
  c=next(c for c in d['commands'] if c['index']==r['render_command']);raw=Path(c['streams']['stdout']['path']).read_bytes();assert sha(raw)==r['report_sha256'];doc=json.loads(raw);docs[(leg['jdk'],r['profile'])]=doc
  m=next(m for m in doc['methods'] if bytes(m['item']['identity']['name'])==b'find' and bytes(m['item']['identity']['descriptor'])==b'([III)I');rep=m['outcome']['report']
  assert rep['quality']=='fallback' and rep['content']=='explanation_only'
  assert 'for (' not in rep['text'] and 'the copy at BCI 5 has no proved local assignment' in rep['text'] and '@bytecode' in rep['text']
  seen=set();buf=rep['text'].encode()
  for seg in rep['source_map']['segments']:
   a,b=seg['start'],seg['end'];assert 0<=a<b<=len(buf);buf[:a].decode();buf[:b].decode()
   for o in [seg['origin']['primary'],*seg['origin']['derived']]:
    assert o['method']==m['item']['identity'] and o['bci'] in physical;seen.add(o['bci'])
  cr=next(c for c in d['commands'] if c['index']==r['compile_command']);assert cr['exit_code']==r['compile_exit']==1 and r['runtime_command'] is None
  err=Path(cr['streams']['stderr']['path']).read_bytes();assert b'missing return statement' in err or '缺少返回语句'.encode() in err
  rows.append({'jdk':leg['jdk'],'profile':r['profile'],'quality':rep['quality'],'content':rep['content'],'refuses_for':True,'complete_source_compile_exit':1,'runtime_claimed':False,'mapped_physical_bcis':sorted(seen),'unmapped_physical_bcis':sorted(physical-seen),'utf8_and_all_existing_origin_identities_valid':True})
for leg in d['legs']:
 a=docs[(leg['jdk'],'default')];b=docs[(leg['jdk'],'all')];assert a['text']==b['text']
 assert len(a['methods'])==len(b['methods'])
 for x,y in zip(a['methods'],b['methods']):
  assert x['item']['identity']==y['item']['identity'] and x['text']==y['text'];rx=x['outcome'].get('report');ry=y['outcome'].get('report')
  if rx is not None:
   assert ry is not None and all(rx[k]==ry[k] for k in ('text','quality','content','source_map'))
result={'status':'accepted_reliable_refusal_only','execution_sha256':sha(E.read_bytes()),'command_count':20,'original_and_jadx_runtime_matched':True,'jarde_complete_source_runtime_accepted':False,'jarde_refusal_reports':rows,'verifier_sha256':sha(Path(__file__).read_bytes()),'general_checker_sha256':sha(Path('/private/tmp/jarde-computed-shared-stack-compare-luna-v3/verify_reports.py').read_bytes())}
Path('/private/tmp/jarde-computed-shared-stack-refusal-acceptance-root-v2.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':result['status'],'reports':len(rows)}))
